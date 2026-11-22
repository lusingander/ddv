use std::{fs, time::Duration};

use anyhow::{ensure, Context, Result};
use aws_sdk_dynamodb::{
    config::{Credentials, Region},
    types::{
        AttributeDefinition, AttributeValue, KeySchemaElement, KeyType, ProvisionedThroughput,
        ScalarAttributeType, TableStatus,
    },
};
use tempfile::{tempdir, TempDir};
use termlens::{Color, Key, Screen, Terminal};
use testcontainers::{
    core::IntoContainerPort, runners::AsyncRunner, ContainerAsync, GenericImage, ImageExt,
};
use tokio::time::{sleep, timeout};

const TABLE_LIST_WIDTH: usize = 30;

pub struct Database {
    pub endpoint: String,
    client: aws_sdk_dynamodb::Client,
    _container: ContainerAsync<GenericImage>,
}

impl Database {
    pub async fn new(tables: &[&str]) -> Result<Self> {
        let container = GenericImage::new("amazon/dynamodb-local", "3.3.2")
            .with_exposed_port(8000.tcp())
            .with_cmd(["-jar", "DynamoDBLocal.jar", "-inMemory", "-sharedDb"])
            .start()
            .await
            .context("Failed to start DynamoDB Local; terminal E2E tests require a running Docker daemon")?;
        let host = container.get_host().await?;
        let port = container.get_host_port_ipv4(8000).await?;
        let endpoint = format!("http://{host}:{port}");
        let config = aws_sdk_dynamodb::Config::builder()
            .behavior_version_latest()
            .region(Region::new("us-east-1"))
            .credentials_provider(Credentials::new("test", "test", None, None, "e2e"))
            .endpoint_url(&endpoint)
            .build();
        let client = aws_sdk_dynamodb::Client::from_conf(config);

        // Check the API itself: a running container need not be ready for requests.
        let ready = timeout(Duration::from_secs(30), async {
            while client.list_tables().send().await.is_err() {
                sleep(Duration::from_millis(100)).await;
            }
        })
        .await;
        if ready.is_err() {
            for log in [
                container.stdout_to_vec().await,
                container.stderr_to_vec().await,
            ]
            .into_iter()
            .flatten()
            {
                eprintln!("DynamoDB Local: {}", String::from_utf8_lossy(&log));
            }
        }
        ready.with_context(|| format!("DynamoDB Local did not become ready at {endpoint}"))?;

        let database = Self {
            endpoint,
            client,
            _container: container,
        };
        for table in tables {
            database.create_table(table).await?;
        }
        Ok(database)
    }

    async fn create_table(&self, name: &str) -> Result<()> {
        self.client
            .create_table()
            .table_name(name)
            .attribute_definitions(
                AttributeDefinition::builder()
                    .attribute_name("pk")
                    .attribute_type(ScalarAttributeType::S)
                    .build()?,
            )
            .key_schema(
                KeySchemaElement::builder()
                    .attribute_name("pk")
                    .key_type(KeyType::Hash)
                    .build()?,
            )
            .provisioned_throughput(
                ProvisionedThroughput::builder()
                    .read_capacity_units(1)
                    .write_capacity_units(1)
                    .build()?,
            )
            .send()
            .await
            .with_context(|| {
                format!(
                    "Failed to create fixture table '{name}' at {}",
                    self.endpoint
                )
            })?;

        timeout(Duration::from_secs(30), async {
            loop {
                let output = self.client.describe_table().table_name(name).send().await;
                if output.is_ok_and(|output| {
                    output.table().and_then(|table| table.table_status())
                        == Some(&TableStatus::Active)
                }) {
                    break;
                }
                sleep(Duration::from_millis(100)).await;
            }
        })
        .await
        .with_context(|| format!("Fixture table '{name}' did not become active"))?;
        Ok(())
    }

    pub async fn put_item(&self, table: &str, key: &str, value: &str) -> Result<()> {
        self.client
            .put_item()
            .table_name(table)
            .item("pk", AttributeValue::S(key.into()))
            .item("value", AttributeValue::S(value.into()))
            .send()
            .await
            .with_context(|| format!("Failed to create fixture item '{key}' in table '{table}'"))?;
        Ok(())
    }
}

pub struct TestApp {
    pub terminal: Terminal,
    _directory: TempDir,
    exited: bool,
}

impl TestApp {
    pub fn spawn(endpoint: &str) -> Result<Self> {
        let directory = tempdir()?;
        let config = directory.path().join("config.toml");
        fs::write(&config, "")?;
        // bin! clears inherited environment variables, including AWS profiles and tokens.
        let terminal = termlens::bin!(
            "ddv",
            size(100, 30),
            timeout(Duration::from_secs(10)),
            current_dir(directory.path()),
            env("DDV_CONFIG", &config),
            env("AWS_ACCESS_KEY_ID", "test"),
            env("AWS_SECRET_ACCESS_KEY", "test"),
            env("AWS_EC2_METADATA_DISABLED", "true"),
            env("AWS_CONFIG_FILE", directory.path().join("aws-config")),
            env(
                "AWS_SHARED_CREDENTIALS_FILE",
                directory.path().join("aws-credentials")
            ),
            env("TZ", "UTC"),
            args(["--endpoint-url", endpoint, "--region", "us-east-1"])
        )?;
        Ok(Self {
            terminal,
            _directory: directory,
            exited: false,
        })
    }

    pub fn quit(mut self) -> Result<()> {
        self.terminal.send(Key::Ctrl('c'))?;
        let status = self.terminal.wait_exit()?;
        ensure!(status.success(), "ddv exit status: {status}");
        let screen = self.terminal.screen();
        ensure!(
            !screen.alternate_screen(),
            "Alternate screen was not restored"
        );
        ensure!(screen.cursor_visible(), "Cursor was not restored");
        self.exited = true;
        Ok(())
    }
}

impl Drop for TestApp {
    fn drop(&mut self) {
        if !self.exited {
            eprintln!("ddv screen before cleanup:\n{}", self.terminal.screen());
        }
    }
}

pub fn selected_table(screen: &Screen, name: &str) -> bool {
    (1..screen.rows().saturating_sub(2)).any(|row| {
        let list: String = screen
            .row_text(row)
            .chars()
            .take(TABLE_LIST_WIDTH)
            .collect();
        list.trim_matches(['│', ' ']) == name
            && screen
                .cell(row, 2)
                .is_some_and(|cell| cell.style().bg == Color::Indexed(10))
    }) && !screen.contains("Loading...")
}

pub fn table_list_ready(screen: &Screen, name: &str) -> bool {
    selected_table(screen, name)
        && (1..screen.rows().saturating_sub(2)).any(|row| {
            let detail: String = screen
                .row_text(row)
                .chars()
                .skip(TABLE_LIST_WIDTH)
                .collect();
            detail.contains("Table Name")
                && detail
                    .split(':')
                    .nth(1)
                    .is_some_and(|value| value.trim_matches(['│', ' ']) == name)
        })
}

pub fn at_table(screen: &Screen, name: &str) -> bool {
    screen.row_text(0).contains(&format!(" {name} ─")) && !screen.contains("Loading...")
}

pub fn at_item(screen: &Screen, table: &str, key: &str) -> bool {
    screen.row_text(0).contains(&format!(" {table} ({key}) ")) && !screen.contains("Loading...")
}

pub fn selected_item(screen: &Screen, key: &str) -> bool {
    let displayed_key = format!("\"{key}\"");
    (2..screen.rows().saturating_sub(2)).any(|row| {
        screen
            .row_text(row)
            .trim_matches(['│', ' '])
            .split_whitespace()
            .next()
            == Some(displayed_key.as_str())
            && screen
                .cell(row, 2)
                .is_some_and(|cell| cell.style().bg == Color::Indexed(8))
    }) && !screen.contains("Loading...")
}
