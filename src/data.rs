use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    slice,
};

use chrono::{DateTime, Local};
use rust_decimal::{prelude::ToPrimitive, Decimal};
use serde::{
    ser::{SerializeMap, SerializeSeq},
    Serialize, Serializer,
};
use serde_with::skip_serializing_none;

use crate::util::to_base64_str;

#[derive(Debug)]
pub struct Table {
    pub name: String,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct TableDescription {
    pub attribute_definitions: Vec<AttributeDefinition>,
    pub table_name: String,
    pub key_schema: Vec<KeySchemaElement>,
    pub table_status: TableStatus,
    pub creation_date_time: DateTime<Local>,
    pub provisioned_throughput: Option<ProvisionedThroughput>,
    pub total_size_bytes: u64,
    pub item_count: u64,
    pub table_arn: String,
    pub local_secondary_indexes: Option<Vec<LocalSecondaryIndexDescription>>,
    pub global_secondary_indexes: Option<Vec<GlobalSecondaryIndexDescription>>,

    #[serde(skip)]
    pub key_schema_type: KeySchemaType,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct AttributeDefinition {
    pub attribute_name: String,
    pub attribute_type: ScalarAttributeType,
}

impl AttributeDefinition {
    pub fn new(attribute_name: String, attribute_type: ScalarAttributeType) -> AttributeDefinition {
        AttributeDefinition {
            attribute_name,
            attribute_type,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ScalarAttributeType {
    B,
    N,
    S,
}

impl ScalarAttributeType {
    pub fn as_str(&self) -> &str {
        match self {
            ScalarAttributeType::B => "B",
            ScalarAttributeType::N => "N",
            ScalarAttributeType::S => "S",
        }
    }
}

impl Serialize for ScalarAttributeType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct KeySchemaElement {
    pub attribute_name: String,
    pub key_type: KeyType,
}

#[derive(Debug, Clone, Copy)]
pub enum KeyType {
    Hash,
    Range,
}

impl KeyType {
    pub fn as_str(&self) -> &str {
        match self {
            KeyType::Hash => "HASH",
            KeyType::Range => "RANGE",
        }
    }
}

impl Serialize for KeyType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

#[derive(Debug, Clone)]
pub enum KeySchemaType {
    Hash(String),
    HashRange(String, String),
}

#[derive(Debug, Clone)]
pub enum TableStatus {
    Active,
    Archived,
    Archiving,
    Creating,
    Deleting,
    InaccessibleEncryptionCredentials,
    Updating,
}

impl TableStatus {
    pub fn as_str(&self) -> &str {
        match self {
            TableStatus::Active => "ACTIVE",
            TableStatus::Archived => "ARCHIVED",
            TableStatus::Archiving => "ARCHIVING",
            TableStatus::Creating => "CREATING",
            TableStatus::Deleting => "DELETING",
            TableStatus::InaccessibleEncryptionCredentials => "INACCESSIBLE_ENCRYPTION_CREDENTIALS",
            TableStatus::Updating => "UPDATING",
        }
    }
}

impl Serialize for TableStatus {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct ProvisionedThroughput {
    pub last_increase_date_time: Option<DateTime<Local>>,
    pub last_decrease_date_time: Option<DateTime<Local>>,
    pub number_of_decreases_today: u64,
    pub read_capacity_units: u64,
    pub write_capacity_units: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct LocalSecondaryIndexDescription {
    pub index_name: String,
    pub key_schema: Vec<KeySchemaElement>,
    pub projection: Projection,
    pub index_size_bytes: u64,
    pub item_count: u64,
    pub index_arn: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct GlobalSecondaryIndexDescription {
    pub index_name: String,
    pub key_schema: Vec<KeySchemaElement>,
    pub projection: Projection,
    pub index_size_bytes: u64,
    pub item_count: u64,
    pub index_arn: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Projection {
    pub projection_type: ProjectionType,
    pub non_key_attributes: Option<Vec<String>>,
}

#[derive(Debug, Clone, Copy)]
pub enum ProjectionType {
    All,
    Include,
    KeysOnly,
}

impl ProjectionType {
    pub fn as_str(&self) -> &str {
        match self {
            ProjectionType::All => "ALL",
            ProjectionType::Include => "INCLUDE",
            ProjectionType::KeysOnly => "KEYS_ONLY",
        }
    }
}

impl Serialize for ProjectionType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

#[derive(Debug, Clone)]
pub struct Item {
    pub attributes: HashMap<String, Attribute>,
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Attribute {
    S(String),
    N(Decimal),
    B(Vec<u8>),
    BOOL(bool),
    NULL,
    L(Vec<Attribute>),
    M(BTreeMap<String, Attribute>),
    SS(BTreeSet<String>),
    NS(BTreeSet<Decimal>),
    BS(BTreeSet<Vec<u8>>),
}

impl Attribute {
    pub fn as_type_str(&self) -> &str {
        match self {
            Attribute::S(_) => "S",
            Attribute::N(_) => "N",
            Attribute::B(_) => "B",
            Attribute::BOOL(_) => "BOOL",
            Attribute::NULL => "NULL",
            Attribute::L(_) => "L",
            Attribute::M(_) => "M",
            Attribute::SS(_) => "SS",
            Attribute::NS(_) => "NS",
            Attribute::BS(_) => "BS",
        }
    }

    pub fn to_simple_string(&self) -> String {
        fn vec<T>(vec: &[T], f: fn(&T) -> String) -> String {
            format!("[{}]", vec.iter().map(f).collect::<Vec<_>>().join(", "))
        }
        fn map<K, V>(map: &BTreeMap<K, V>, f: fn((&K, &V)) -> String) -> String {
            format!("{{{}}}", map.iter().map(f).collect::<Vec<_>>().join(", "))
        }
        fn set<T>(set: &BTreeSet<T>, f: fn(&T) -> String) -> String {
            format!("[{}]", set.iter().map(f).collect::<Vec<_>>().join(", "))
        }
        match self {
            Attribute::S(s) => s.clone(),
            Attribute::N(n) => n.to_string(),
            Attribute::B(b) => format!("Blob ({})", b.len()),
            Attribute::BOOL(b) => b.to_string(),
            Attribute::NULL => "null".to_string(),
            Attribute::L(l) => vec(l, |a| a.to_simple_string()),
            Attribute::M(m) => map(m, |(k, v)| format!("{}: {}", k, v.to_simple_string())),
            Attribute::SS(s) => set(s, |s| s.clone()),
            Attribute::NS(n) => set(n, |n| n.to_string()),
            Attribute::BS(b) => set(b, |b| format!("Blob ({})", b.len())),
        }
    }
}

impl PartialOrd for Attribute {
    fn partial_cmp(&self, other: &Attribute) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (Attribute::S(a), Attribute::S(b)) => a.partial_cmp(b),
            (Attribute::N(a), Attribute::N(b)) => a.partial_cmp(b),
            (Attribute::B(a), Attribute::B(b)) => a.partial_cmp(b),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AttributeType {
    String,
    Number,
    Blob,
    Bool,
    Null,
    List,
    Map,
    StringSet,
    NumberSet,
    BlobSet,
    None,
}

impl From<Option<&Attribute>> for AttributeType {
    fn from(attr: Option<&Attribute>) -> AttributeType {
        match attr {
            Some(Attribute::S(_)) => AttributeType::String,
            Some(Attribute::N(_)) => AttributeType::Number,
            Some(Attribute::B(_)) => AttributeType::Blob,
            Some(Attribute::BOOL(_)) => AttributeType::Bool,
            Some(Attribute::NULL) => AttributeType::Null,
            Some(Attribute::L(_)) => AttributeType::List,
            Some(Attribute::M(_)) => AttributeType::Map,
            Some(Attribute::SS(_)) => AttributeType::StringSet,
            Some(Attribute::NS(_)) => AttributeType::NumberSet,
            Some(Attribute::BS(_)) => AttributeType::BlobSet,
            None => AttributeType::None,
        }
    }
}

impl AttributeType {
    pub fn as_str(&self) -> &str {
        match self {
            AttributeType::String => "S",
            AttributeType::Number => "N",
            AttributeType::Blob => "B",
            AttributeType::Bool => "BOOL",
            AttributeType::Null => "NULL",
            AttributeType::List => "L",
            AttributeType::Map => "M",
            AttributeType::StringSet => "SS",
            AttributeType::NumberSet => "NS",
            AttributeType::BlobSet => "BS",
            AttributeType::None => "undefined",
        }
    }
}

pub struct RawJsonItem<'a> {
    item: &'a Item,
    schema: &'a KeySchemaType,
}

impl<'a> RawJsonItem<'a> {
    pub fn new(item: &'a Item, schema: &'a KeySchemaType) -> RawJsonItem<'a> {
        RawJsonItem { item, schema }
    }
}

impl Serialize for RawJsonItem<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut attributes_map = serializer.serialize_map(Some(self.item.attributes.len()))?;
        let keys = list_attribute_keys(slice::from_ref(self.item), self.schema);
        for key in &keys {
            let attr = self.item.attributes.get(key).unwrap();
            attributes_map.serialize_entry(key, &RawAttributeJsonWrapper(attr))?;
        }
        attributes_map.end()
    }
}

pub struct PlainJsonItem<'a> {
    item: &'a Item,
    schema: &'a KeySchemaType,
}

impl<'a> PlainJsonItem<'a> {
    pub fn new(item: &'a Item, schema: &'a KeySchemaType) -> PlainJsonItem<'a> {
        PlainJsonItem { item, schema }
    }
}

impl Serialize for PlainJsonItem<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut attributes_map = serializer.serialize_map(Some(self.item.attributes.len()))?;
        let keys = list_attribute_keys(slice::from_ref(self.item), self.schema);
        for key in &keys {
            let attr = self.item.attributes.get(key).unwrap();
            attributes_map.serialize_entry(key, attr)?;
        }
        attributes_map.end()
    }
}

impl Serialize for Attribute {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Attribute::S(value) => serializer.serialize_str(value),
            Attribute::N(value) => match decimal_type(value) {
                DecimalType::Integer(i) => serializer.serialize_i64(i),
                DecimalType::Float(f) => serializer.serialize_f64(f),
                DecimalType::String(s) => serializer.serialize_str(&s),
            },
            Attribute::B(value) => serializer.serialize_str(&to_base64_str(value)),
            Attribute::BOOL(value) => serializer.serialize_bool(*value),
            Attribute::NULL => serializer.serialize_none(),
            Attribute::L(vec) => vec.serialize(serializer),
            Attribute::M(map) => map.serialize(serializer),
            Attribute::SS(set) => set.serialize(serializer),
            Attribute::NS(set) => {
                let mut seq = serializer.serialize_seq(Some(set.len()))?;
                for value in set {
                    match decimal_type(value) {
                        DecimalType::Integer(i) => seq.serialize_element(&i)?,
                        DecimalType::Float(f) => seq.serialize_element(&f)?,
                        DecimalType::String(s) => seq.serialize_element(&s)?,
                    }
                }
                seq.end()
            }
            Attribute::BS(set) => {
                let mut seq = serializer.serialize_seq(Some(set.len()))?;
                for value in set {
                    seq.serialize_element(&to_base64_str(value))?;
                }
                seq.end()
            }
        }
    }
}

struct RawAttributeJsonWrapper<'a>(&'a Attribute);

impl Serialize for RawAttributeJsonWrapper<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self.0 {
            attr @ Attribute::NULL => {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry(attr.as_type_str(), &Attribute::BOOL(true))?;
                map.end()
            }
            attr @ Attribute::L(vec) => {
                let attrs = vec.iter().map(RawAttributeJsonWrapper).collect::<Vec<_>>();
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry(attr.as_type_str(), &attrs)?;
                map.end()
            }
            attr @ Attribute::M(map) => {
                let attrs = map
                    .iter()
                    .map(|(k, v)| (k.clone(), RawAttributeJsonWrapper(v)))
                    .collect::<BTreeMap<_, _>>();
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry(attr.as_type_str(), &attrs)?;
                map.end()
            }
            attr => {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry(attr.as_type_str(), attr)?;
                map.end()
            }
        }
    }
}

pub fn to_key_string(item: &Item, schema: &KeySchemaType) -> String {
    match schema {
        KeySchemaType::Hash(key) => item.attributes.get(key).unwrap().to_simple_string(),
        KeySchemaType::HashRange(key1, key2) => {
            let key1_str = item.attributes.get(key1).unwrap().to_simple_string();
            let key2_str = item.attributes.get(key2).unwrap().to_simple_string();
            format!("{} / {}", key1_str, key2_str)
        }
    }
}

pub fn list_attribute_keys(items: &[Item], schema: &KeySchemaType) -> Vec<String> {
    let keys_set: HashSet<_> = get_all_keys(items);
    let mut keys: Vec<_> = keys_set.into_iter().cloned().collect();
    sort_keys(&mut keys, schema);
    keys
}

fn get_all_keys(items: &[Item]) -> HashSet<&String> {
    items
        .iter()
        .flat_map(|item| item.attributes.keys())
        .collect()
}

fn sort_keys(keys: &mut [String], schema: &KeySchemaType) {
    match schema {
        KeySchemaType::Hash(k) => {
            keys.sort_by(|a, b| {
                if a == k {
                    std::cmp::Ordering::Less
                } else if b == k {
                    std::cmp::Ordering::Greater
                } else {
                    a.cmp(b)
                }
            });
        }
        KeySchemaType::HashRange(k1, k2) => {
            keys.sort_by(|a, b| {
                if a == k1 {
                    std::cmp::Ordering::Less
                } else if b == k1 {
                    std::cmp::Ordering::Greater
                } else if a == k2 {
                    std::cmp::Ordering::Less
                } else if b == k2 {
                    std::cmp::Ordering::Greater
                } else {
                    a.cmp(b)
                }
            });
        }
    }
}

enum DecimalType {
    Integer(i64),
    Float(f64),
    String(String),
}

fn decimal_type(value: &Decimal) -> DecimalType {
    if value.is_integer() {
        if let Some(i) = value.to_i64() {
            return DecimalType::Integer(i);
        }
    }
    if let Some(f) = value.to_f64() {
        return DecimalType::Float(f);
    }
    DecimalType::String(value.to_string())
}

pub struct TableInsight {
    pub table_name: String,
    pub total_items: usize,
    pub attribute_distributions: Vec<AttributeDistribution>,
}

impl TableInsight {
    pub fn new(table_description: &TableDescription, items: &[Item]) -> TableInsight {
        let table_name = table_description.table_name.clone();
        let total_items = items.len();

        let attribute_keys = list_attribute_keys(items, &table_description.key_schema_type);
        let attribute_distributions = build_attribute_distributions(items, &attribute_keys);

        TableInsight {
            table_name,
            total_items,
            attribute_distributions,
        }
    }
}

fn build_attribute_distributions(
    items: &[Item],
    attribute_keys: &[String],
) -> Vec<AttributeDistribution> {
    let mut key_counter = Vec::new();
    for key in attribute_keys {
        key_counter.push((key, HashMap::new()));
    }
    for item in items {
        for (i, key) in attribute_keys.iter().enumerate() {
            let attr = item.attributes.get(key);
            let attr_type = AttributeType::from(attr);
            let (_, counter) = key_counter.get_mut(i).unwrap();
            *counter.entry(attr_type).or_insert(0) += 1;
        }
    }
    key_counter
        .into_iter()
        .map(|(key, counter)| {
            let mut distributions: Vec<(AttributeType, usize)> = counter.into_iter().collect();
            distributions.sort_by(|a, b| b.1.cmp(&a.1)); // sort by count desc
            AttributeDistribution {
                attribute_name: key.clone(),
                distributions,
            }
        })
        .collect()
}

pub struct AttributeDistribution {
    pub attribute_name: String,
    pub distributions: Vec<(AttributeType, usize)>,
}

#[cfg(test)]
mod tests {
    use rust_decimal::prelude::FromPrimitive;

    use super::*;

    #[test]
    fn test_raw_json_item_serialize() {
        let item = fixture_item();
        let schema = KeySchemaType::Hash("b".into());

        let json_item = RawJsonItem::new(&item, &schema);
        let json = serde_json::to_string_pretty(&json_item).unwrap();

        assert_eq!(
            json,
            r#"{
  "b": {
    "N": 123
  },
  "a": {
    "S": "aaa"
  },
  "c": {
    "SS": [
      "c1",
      "c2"
    ]
  },
  "d": {
    "L": [
      {
        "NULL": true
      },
      {
        "B": "YWJj"
      },
      {
        "BS": [
          "bG1u",
          "eHl6"
        ]
      }
    ]
  },
  "e": {
    "M": {
      "e1": {
        "BOOL": true
      },
      "e2": {
        "NS": [
          -2.34,
          0.2,
          3
        ]
      }
    }
  }
}"#
        );
    }

    #[test]
    fn test_plain_json_item_serialize() {
        let item = fixture_item();
        let schema = KeySchemaType::Hash("b".into());

        let json_item = PlainJsonItem::new(&item, &schema);
        let json = serde_json::to_string_pretty(&json_item).unwrap();

        assert_eq!(
            json,
            r#"{
  "b": 123,
  "a": "aaa",
  "c": [
    "c1",
    "c2"
  ],
  "d": [
    null,
    "YWJj",
    [
      "bG1u",
      "eHl6"
    ]
  ],
  "e": {
    "e1": true,
    "e2": [
      -2.34,
      0.2,
      3
    ]
  }
}"#
        );
    }

    fn fixture_item() -> Item {
        Item {
            attributes: vec![
                (
                    "d".into(),
                    Attribute::L(vec![
                        Attribute::NULL,
                        Attribute::B("abc".as_bytes().to_vec()),
                        Attribute::BS(set(vec![
                            "lmn".as_bytes().to_vec(),
                            "xyz".as_bytes().to_vec(),
                        ])),
                    ]),
                ),
                ("a".into(), Attribute::S("aaa".into())),
                (
                    "e".into(),
                    Attribute::M(
                        vec![
                            ("e1".into(), Attribute::BOOL(true)),
                            (
                                "e2".into(),
                                Attribute::NS(set(vec![
                                    Decimal::from(3),
                                    Decimal::from_f64(0.2).unwrap(),
                                    Decimal::from_f64(-2.34).unwrap(),
                                ])),
                            ),
                        ]
                        .into_iter()
                        .collect(),
                    ),
                ),
                ("c".into(), Attribute::SS(set(vec!["c1", "c2"]))),
                ("b".into(), Attribute::N(Decimal::from(123))),
            ]
            .into_iter()
            .collect(),
        }
    }

    #[test]
    fn test_list_attribute_keys() {
        fn item(keys: &[&str]) -> Item {
            let attributes = keys
                .iter()
                .map(|k| (k.to_string(), Attribute::NULL))
                .collect();
            Item { attributes }
        }

        let items = vec![item(&["a", "b", "c"]), item(&["c", "d"]), item(&["e", "b"])];

        let schema = KeySchemaType::Hash("b".into());
        let keys = list_attribute_keys(&items, &schema);
        assert_eq!(keys, vec!["b", "a", "c", "d", "e"]);

        let schema = KeySchemaType::HashRange("b".into(), "c".into());
        let keys = list_attribute_keys(&items, &schema);
        assert_eq!(keys, vec!["b", "c", "a", "d", "e"]);
    }

    fn set<T, U>(values: Vec<T>) -> BTreeSet<U>
    where
        U: From<T> + Ord,
    {
        values.into_iter().map(|t| t.into()).collect()
    }
}
