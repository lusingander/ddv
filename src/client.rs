use std::{
    collections::{BTreeSet, HashMap},
    str::FromStr,
};

use aws_config::{default_provider, meta::region::RegionProviderChain, BehaviorVersion, Region};
use aws_sdk_dynamodb::types::{
    AttributeDefinition as AwsAttributeDefinition, AttributeValue as AwsAttributeValue,
    GlobalSecondaryIndexDescription as AwsGlobalSecondaryIndexDescription,
    KeySchemaElement as AwsKeySchemaElement, KeyType as AwsKeyType,
    LocalSecondaryIndexDescription as AwsLocalSecondaryIndexDescription,
    Projection as AwsProjection, ProjectionType as AwsProjectionType,
    ProvisionedThroughputDescription as AwsProvisionedThroughputDescription,
    ScalarAttributeType as AwsScalarAttributeType, TableDescription as AwsTableDescription,
    TableStatus as AwsTableStatus,
};
use aws_smithy_types::{Blob, DateTime as AwsDateTime};
use chrono::{DateTime, Local, TimeZone as _};
use rust_decimal::Decimal;

use crate::{
    data::{
        Attribute, AttributeDefinition, GlobalSecondaryIndexDescription, Item, KeySchemaElement,
        KeySchemaType, KeyType, LocalSecondaryIndexDescription, Projection, ProjectionType,
        ProvisionedThroughput, ScalarAttributeType, Table, TableDescription, TableStatus,
    },
    error::{AppError, AppResult},
};

pub struct Client {
    client: aws_sdk_dynamodb::Client,
}

impl Client {
    pub async fn new(
        region: Option<String>,
        endpoint_url: Option<String>,
        profile: Option<String>,
        default_region_fallback: String,
    ) -> Client {
        let mut region_builder = default_provider::region::Builder::default();
        if let Some(profile) = &profile {
            region_builder = region_builder.profile_name(profile);
        }
        let region_provider = RegionProviderChain::first_try(region.map(Region::new))
            .or_else(region_builder.build())
            .or_else(Region::new(default_region_fallback));

        let mut config_loader =
            aws_config::defaults(BehaviorVersion::latest()).region(region_provider);
        if let Some(endpoint_url) = &endpoint_url {
            config_loader = config_loader.endpoint_url(endpoint_url);
        }
        if let Some(profile) = &profile {
            config_loader = config_loader.profile_name(profile);
        }
        let sdk_config = config_loader.load().await;

        let config_builder = aws_sdk_dynamodb::config::Builder::from(&sdk_config);
        let config = config_builder.build();

        let client = aws_sdk_dynamodb::Client::from_conf(config);
        Client { client }
    }

    pub async fn list_all_tables(&self) -> AppResult<Vec<Table>> {
        let mut last_evaluated_table_name = None;
        let mut tables = Vec::new();
        loop {
            let mut req = self.client.list_tables();
            if let Some(table_name) = last_evaluated_table_name {
                req = req.exclusive_start_table_name(table_name);
            }

            let result = req.send().await;
            let output = result.map_err(|e| AppError::new("failed to list tables", e))?;

            tables.extend(
                output
                    .table_names
                    .unwrap_or_default()
                    .into_iter()
                    .map(Into::into),
            );

            if output.last_evaluated_table_name.is_none() {
                break;
            }
            last_evaluated_table_name = output.last_evaluated_table_name;
        }
        Ok(tables)
    }

    pub async fn describe_table(&self, table_name: &str) -> AppResult<TableDescription> {
        let req = self.client.describe_table().table_name(table_name);

        let result = req.send().await;
        let output = result.map_err(|e| AppError::new("failed to load table description", e))?;

        let desc = to_table_description(output.table.unwrap());
        Ok(desc)
    }

    pub async fn scan_all_items(
        &self,
        table_name: &str,
        schema: &KeySchemaType,
    ) -> AppResult<Vec<Item>> {
        let mut last_evaluated_key = None;
        let mut items = Vec::new();
        loop {
            let mut req = self.client.scan().table_name(table_name);
            if last_evaluated_key.is_some() {
                req = req.set_exclusive_start_key(last_evaluated_key);
            }

            let result = req.send().await;
            let output = result.map_err(|e| AppError::new("failed to scan items", e))?;

            items.extend(output.items.unwrap_or_default().into_iter().map(to_item));

            if output.last_evaluated_key.is_none() {
                break;
            }
            last_evaluated_key = output.last_evaluated_key;
        }
        sort_items(&mut items, schema);
        Ok(items)
    }

    pub async fn delete_item(
        &self,
        table_name: &str,
        schema: &KeySchemaType,
        item: &Item,
    ) -> AppResult<()> {
        let key = build_key_attributes(item, schema);
        let result = self
            .client
            .delete_item()
            .table_name(table_name)
            .set_key(Some(key))
            .send()
            .await;

        result
            .map(|_| ())
            .map_err(|e| AppError::new("failed to delete item", e))
    }
}

impl From<String> for Table {
    fn from(name: String) -> Self {
        Table { name }
    }
}

fn to_table_description(desc: AwsTableDescription) -> TableDescription {
    let attribute_definitions = vec_into(desc.attribute_definitions.unwrap());
    let table_name = desc.table_name.unwrap();
    let key_schema = vec_into(desc.key_schema.unwrap());
    let table_status = desc.table_status.unwrap().into();
    let creation_date_time = convert_datetime(desc.creation_date_time.unwrap());
    let provisioned_throughput = desc.provisioned_throughput.map(Into::into);
    let total_size_bytes = desc.table_size_bytes.unwrap() as u64;
    let item_count = desc.item_count.unwrap() as u64;
    let table_arn = desc.table_arn.unwrap();
    let local_secondary_indexes = desc.local_secondary_indexes.map(vec_into);
    let global_secondary_indexes = desc.global_secondary_indexes.map(vec_into);

    let key_schema_type = to_key_schema_type(key_schema.clone());

    TableDescription {
        attribute_definitions,
        table_name,
        key_schema,
        table_status,
        creation_date_time,
        provisioned_throughput,
        total_size_bytes,
        item_count,
        table_arn,
        local_secondary_indexes,
        global_secondary_indexes,

        key_schema_type,
    }
}

impl From<AwsAttributeDefinition> for AttributeDefinition {
    fn from(def: AwsAttributeDefinition) -> Self {
        AttributeDefinition::new(def.attribute_name, def.attribute_type.into())
    }
}

impl From<AwsScalarAttributeType> for ScalarAttributeType {
    fn from(s: AwsScalarAttributeType) -> Self {
        match s {
            AwsScalarAttributeType::B => ScalarAttributeType::B,
            AwsScalarAttributeType::N => ScalarAttributeType::N,
            AwsScalarAttributeType::S => ScalarAttributeType::S,
            _ => unreachable!("unexpected scalar attribute type: {:?}", s),
        }
    }
}

impl From<AwsTableStatus> for TableStatus {
    fn from(s: AwsTableStatus) -> Self {
        match s {
            AwsTableStatus::Active => TableStatus::Active,
            AwsTableStatus::Archived => TableStatus::Archived,
            AwsTableStatus::Archiving => TableStatus::Archiving,
            AwsTableStatus::Creating => TableStatus::Creating,
            AwsTableStatus::Deleting => TableStatus::Deleting,
            AwsTableStatus::InaccessibleEncryptionCredentials => {
                TableStatus::InaccessibleEncryptionCredentials
            }
            AwsTableStatus::Updating => TableStatus::Updating,
            _ => unreachable!("unexpected table status: {:?}", s),
        }
    }
}

fn to_key_schema(key_schema: Vec<AwsKeySchemaElement>) -> Vec<KeySchemaElement> {
    key_schema.into_iter().map(Into::into).collect()
}

impl From<AwsKeySchemaElement> for KeySchemaElement {
    fn from(schema: AwsKeySchemaElement) -> Self {
        KeySchemaElement {
            attribute_name: schema.attribute_name,
            key_type: schema.key_type.into(),
        }
    }
}

impl From<AwsKeyType> for KeyType {
    fn from(t: AwsKeyType) -> Self {
        match t {
            AwsKeyType::Hash => KeyType::Hash,
            AwsKeyType::Range => KeyType::Range,
            _ => unreachable!("unexpected key type: {:?}", t),
        }
    }
}

impl From<AwsLocalSecondaryIndexDescription> for LocalSecondaryIndexDescription {
    fn from(value: AwsLocalSecondaryIndexDescription) -> Self {
        let index_name = value.index_name.unwrap();
        let key_schema = to_key_schema(value.key_schema.unwrap());
        let projection = value.projection.unwrap().into();
        let index_size_bytes = value.index_size_bytes.unwrap_or(0) as u64;
        let item_count = value.item_count.unwrap_or(0) as u64;
        let index_arn = value.index_arn.unwrap_or("".to_string());
        LocalSecondaryIndexDescription {
            index_name,
            key_schema,
            projection,
            index_size_bytes,
            item_count,
            index_arn,
        }
    }
}

impl From<AwsGlobalSecondaryIndexDescription> for GlobalSecondaryIndexDescription {
    fn from(value: AwsGlobalSecondaryIndexDescription) -> Self {
        let index_name = value.index_name.unwrap();
        let key_schema = to_key_schema(value.key_schema.unwrap());
        let projection = value.projection.unwrap().into();
        let index_size_bytes = value.index_size_bytes.unwrap() as u64;
        let item_count = value.item_count.unwrap() as u64;
        let index_arn = value.index_arn.unwrap();
        GlobalSecondaryIndexDescription {
            index_name,
            key_schema,
            projection,
            index_size_bytes,
            item_count,
            index_arn,
        }
    }
}

impl From<AwsProjection> for Projection {
    fn from(p: AwsProjection) -> Self {
        let projection_type = p.projection_type.unwrap().into();
        let non_key_attributes = p.non_key_attributes;
        Projection {
            projection_type,
            non_key_attributes,
        }
    }
}

impl From<AwsProjectionType> for ProjectionType {
    fn from(t: AwsProjectionType) -> Self {
        match t {
            AwsProjectionType::All => ProjectionType::All,
            AwsProjectionType::KeysOnly => ProjectionType::KeysOnly,
            AwsProjectionType::Include => ProjectionType::Include,
            _ => unreachable!("unexpected projection type: {:?}", t),
        }
    }
}

fn to_key_schema_type(elements: Vec<KeySchemaElement>) -> KeySchemaType {
    let mut hash_key = None;
    let mut range_key = None;
    for elem in elements {
        match elem.key_type {
            KeyType::Hash => {
                if hash_key.is_some() {
                    panic!("multiple hash keys");
                }
                hash_key = Some(elem.attribute_name);
            }
            KeyType::Range => {
                if range_key.is_some() {
                    panic!("multiple range keys");
                }
                range_key = Some(elem.attribute_name);
            }
        }
    }
    match (hash_key, range_key) {
        (Some(hash_key), Some(range_key)) => KeySchemaType::HashRange(hash_key, range_key),
        (Some(hash_key), None) => KeySchemaType::Hash(hash_key),
        (hash_key, range_key) => {
            panic!("unexpected key schema: ({hash_key:?}, {range_key:?})")
        }
    }
}

fn to_item(attributes: HashMap<String, AwsAttributeValue>) -> Item {
    let attributes = attributes.into_iter().map(|(k, v)| (k, v.into())).collect();
    Item { attributes }
}

fn build_key_attributes(item: &Item, schema: &KeySchemaType) -> HashMap<String, AwsAttributeValue> {
    match schema {
        KeySchemaType::Hash(hash_key) => {
            let mut key = HashMap::with_capacity(1);
            let attr = item
                .attributes
                .get(hash_key)
                .expect("missing hash key attribute");
            key.insert(hash_key.clone(), attribute_to_aws(attr));
            key
        }
        KeySchemaType::HashRange(hash_key, range_key) => {
            let mut key = HashMap::with_capacity(2);
            let hash_attr = item
                .attributes
                .get(hash_key)
                .expect("missing hash key attribute");
            let range_attr = item
                .attributes
                .get(range_key)
                .expect("missing range key attribute");
            key.insert(hash_key.clone(), attribute_to_aws(hash_attr));
            key.insert(range_key.clone(), attribute_to_aws(range_attr));
            key
        }
    }
}

fn attribute_to_aws(attr: &Attribute) -> AwsAttributeValue {
    match attr {
        Attribute::S(s) => AwsAttributeValue::S(s.clone()),
        Attribute::N(n) => AwsAttributeValue::N(n.to_string()),
        Attribute::B(b) => AwsAttributeValue::B(Blob::new(b.clone())),
        Attribute::BOOL(b) => AwsAttributeValue::Bool(*b),
        Attribute::NULL => AwsAttributeValue::Null(true),
        Attribute::L(list) => {
            let values = list.iter().map(attribute_to_aws).collect();
            AwsAttributeValue::L(values)
        }
        Attribute::M(map) => {
            let values = map
                .iter()
                .map(|(k, v)| (k.clone(), attribute_to_aws(v)))
                .collect();
            AwsAttributeValue::M(values)
        }
        Attribute::SS(set) => AwsAttributeValue::Ss(set.iter().cloned().collect()),
        Attribute::NS(set) => AwsAttributeValue::Ns(set.iter().map(|n| n.to_string()).collect()),
        Attribute::BS(set) => {
            let values = set.iter().cloned().map(Blob::new).collect();
            AwsAttributeValue::Bs(values)
        }
    }
}

impl From<AwsAttributeValue> for Attribute {
    fn from(value: AwsAttributeValue) -> Self {
        match value {
            AwsAttributeValue::S(s) => Attribute::S(s),
            AwsAttributeValue::N(n) => Attribute::N(Decimal::from_str(&n).unwrap()),
            AwsAttributeValue::B(b) => Attribute::B(b.into_inner()),
            AwsAttributeValue::Bool(b) => Attribute::BOOL(b),
            AwsAttributeValue::Null(_) => Attribute::NULL,
            AwsAttributeValue::M(m) => {
                let m = m.into_iter().map(|(k, v)| (k, v.into())).collect();
                Attribute::M(m)
            }
            AwsAttributeValue::L(vs) => {
                let vs = vs.into_iter().map(Into::into).collect();
                Attribute::L(vs)
            }
            AwsAttributeValue::Ss(ss) => {
                let ss = BTreeSet::from_iter(ss);
                Attribute::SS(ss)
            }
            AwsAttributeValue::Ns(ns) => {
                let ns =
                    BTreeSet::from_iter(ns.into_iter().map(|n| Decimal::from_str(&n).unwrap()));
                Attribute::NS(ns)
            }
            AwsAttributeValue::Bs(bs) => {
                let bs = BTreeSet::from_iter(bs.into_iter().map(|b| b.into_inner()));
                Attribute::BS(bs)
            }
            _ => unreachable!("unexpected attribute value: {:?}", value),
        }
    }
}

impl From<AwsProvisionedThroughputDescription> for ProvisionedThroughput {
    fn from(t: AwsProvisionedThroughputDescription) -> Self {
        ProvisionedThroughput {
            last_increase_date_time: t.last_increase_date_time.map(convert_datetime),
            last_decrease_date_time: t.last_decrease_date_time.map(convert_datetime),
            number_of_decreases_today: t.number_of_decreases_today.unwrap() as u64,
            read_capacity_units: t.read_capacity_units.unwrap() as u64,
            write_capacity_units: t.write_capacity_units.unwrap() as u64,
        }
    }
}

fn sort_items(items: &mut [Item], schema: &KeySchemaType) {
    match schema {
        KeySchemaType::Hash(hash_key) => {
            items.sort_by(|a, b| {
                let a = a.attributes.get(hash_key).unwrap();
                let b = b.attributes.get(hash_key).unwrap();
                a.partial_cmp(b).unwrap()
            });
        }
        KeySchemaType::HashRange(hash_key, range_key) => {
            items.sort_by(|a, b| {
                let a_hash = a.attributes.get(hash_key).unwrap();
                let b_hash = b.attributes.get(hash_key).unwrap();
                match a_hash.partial_cmp(b_hash).unwrap() {
                    std::cmp::Ordering::Equal => {
                        let a_range = a.attributes.get(range_key).unwrap();
                        let b_range = b.attributes.get(range_key).unwrap();
                        a_range.partial_cmp(b_range).unwrap()
                    }
                    ord => ord,
                }
            });
        }
    }
}

fn convert_datetime(dt: AwsDateTime) -> DateTime<Local> {
    let nanos = dt.as_nanos();
    Local.timestamp_nanos(nanos as i64)
}

fn vec_into<T, U>(ts: Vec<T>) -> Vec<U>
where
    U: From<T>,
{
    ts.into_iter().map(Into::into).collect()
}
