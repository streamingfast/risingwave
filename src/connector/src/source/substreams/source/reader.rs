use std::env;
use std::sync::Arc;
use std::time::Duration;
use tokio::time;

use anyhow::{Context, Error, format_err};
use async_trait::async_trait;
use futures::StreamExt;
use futures_async_stream::try_stream;
use google_cloud_gax::grpc::Code;
use lazy_static::lazy_static;
use prost::Message;
use regex::Regex;
use risingwave_common::{bail, ensure};
use semver::Version;
use sqlx::postgres::{PgPool, PgPoolOptions};

use crate::error::{ConnectorError, ConnectorResult as Result};
use crate::parser::ParserConfig;
use crate::source::google_pubsub::PubsubSplitReader;
use crate::source::substreams::SubstreamsProperties;
use crate::source::substreams::meta::SubstreamsMeta;
use crate::source::substreams::pb::sf::substreams::v1::Package;
use crate::source::substreams::split::SubstreamsSplit;
use crate::source::substreams::substreams::SubstreamsEndpoint;
use crate::source::substreams::substreams_stream::{BlockResponse, SubstreamsStream};
use crate::source::{
    BoxSourceChunkStream, Column, SourceContextRef, SourceMessage, SourceMeta, SplitId,
    SplitMetaData, SplitReader, into_chunk_stream,
};
use crate::source::substreams::cursor::Cursor;

lazy_static! {
    static ref MODULE_NAME_REGEXP: Regex = Regex::new(r"^([a-zA-Z][a-zA-Z0-9_-]{0,63})$").unwrap();
}

const REGISTRY_URL: &str = "https://spkg.io";

pub struct SubstreamsSplitReader {
    pub package: Package,
    pub block_range: (i64, u64),
    pub endpoint: Arc<SubstreamsEndpoint>,
    pub module_name: String,
    pub split_id: SplitId,
    pub parser_config: ParserConfig,
    pub source_ctx: SourceContextRef,
    pub db_pool: PgPool,
    pub opaque_cursor: Option<String>,
}

impl SubstreamsSplitReader {
    #[try_stream(ok = Vec<SourceMessage>, error = ConnectorError)]
    async fn into_data_stream(self) {
        tracing::info!("Starting StreamingFast Substreams stream with cursor: {:?}", self.opaque_cursor.clone());

        let stop_block = self.block_range.1;

        match self.opaque_cursor.clone() {
            None => {}
            Some(oc) => {
                let cursor = Cursor::from_opaque(oc.as_str());
                if cursor.is_ok() {
                    if stop_block > 0 && cursor.unwrap().block.number >= stop_block {
                        tracing::info!("Grrr: Stop block reached, skipping substreams request");

                        loop {
                            tracing::info!("Sleeping");
                            time::sleep(Duration::from_secs(10)).await;
                        }

                    }
                }
            }
        }

        let mut stream = SubstreamsStream::new(
            self.endpoint,
            self.opaque_cursor,
            self.package.modules,
            self.module_name,
            self.block_range.0,
            stop_block,
        );

        loop {
            match stream.next().await {
                None => {
                    loop {
                        tracing::info!("Stream consume... Sleeping");
                        time::sleep(Duration::from_secs(10)).await;
                    }
                }

                Some(Ok(BlockResponse::New(data))) => {
                    let output = data.output.unwrap().map_output.unwrap();
                    let block_num = data.clock.unwrap().number;
                    // if block_num%1000 == 0 {
                    //
                    //     match sqlx::query("DELETE FROM grrrr")
                    //         .execute(&self.db_pool)
                    //         .await {
                    //         Ok(_) => println!("Deleted {} rows", block_num),
                    //         Err(e) => bail!("Error delete rows: {}", e)
                    //     }
                    // }
                    // tracing::info!("New block received: {} {}", output.type_url, output.value.len());

                    let meta = SubstreamsMeta {
                        block_number: block_num,
                    };
                    let chunk: Vec<SourceMessage> = vec![SourceMessage {
                        key: Some(block_num.to_string().as_str().into()),
                        payload: Some(output.value),
                        offset: data.cursor,
                        split_id: self.split_id.clone(),
                        meta: SourceMeta::Substreams(meta),
                    }];

                    yield chunk;
                }
                Some(Ok(BlockResponse::Undo(undo_signal))) => {}
                Some(Err(err)) => bail!("Error: {}", err),
            }
        }
    }

}


#[async_trait]
impl SplitReader for SubstreamsSplitReader {
    type Properties = SubstreamsProperties;
    type Split = SubstreamsSplit;

    async fn new(
        properties: Self::Properties,
        splits: Vec<Self::Split>,
        parser_config: ParserConfig,
        source_ctx: SourceContextRef,
        _columns: Option<Vec<Column>>,
    ) -> Result<Self> {
        ensure!(
            splits.len() == 1,
            "the substreams reader only supports a single split"
        );
        let split = splits.into_iter().next().unwrap();
        tracing::info!("New SubstreamsSplitReader with cursor: {:?}", split.opaque_cursor);

        let package = read_package(&split.package_file).await?;
        // let block_range = read_block_range(&package, &split.module_name)?;
        let block_range = (split.start_block as i64, split.stop_block);
        let token_env = env::var("SUBSTREAMS_API_TOKEN").unwrap_or("".to_string());
        ensure!(token_env.len() > 0, "Missing SUBSTREAMS_API_TOKEN env var");
        let endpoint =
            Arc::new(SubstreamsEndpoint::new(&split.endpoint_url, Some(token_env)).await?);

        let db_pool = PgPoolOptions::new()
            .max_connections(5)
            .connect("postgres://root@0.0.0.0:4566/dev")
            .await?;

        Ok(Self {
            split_id: split.id().clone(),
            module_name: split.module_name,
            package,
            block_range,
            endpoint,
            parser_config,
            source_ctx,
            db_pool,
            opaque_cursor: split.opaque_cursor,
        })
    }

    fn into_stream(self) -> BoxSourceChunkStream {
        let parser_config = self.parser_config.clone();
        let source_context = self.source_ctx.clone();
        into_chunk_stream(self.into_data_stream(), parser_config, source_context)
    }
}

async fn read_package(input: &str) -> std::result::Result<Package, anyhow::Error> {
    let mut mutable_input = input.to_string();

    let val = parse_standard_package_and_version(input);
    if val.is_ok() {
        let package_and_version = val.unwrap();
        mutable_input = format!(
            "{}/v1/packages/{}/{}",
            REGISTRY_URL, package_and_version.0, package_and_version.1
        );
    }

    if mutable_input.starts_with("http") {
        return read_http_package(&mutable_input).await;
    }

    if mutable_input.starts_with("file://") {
        mutable_input = mutable_input.replace("file://", "");
    }

    // Assume it's a local file
    let content = std::fs::read(&mutable_input)
        .context(format_err!("read package from file '{}'", mutable_input))?;
    Package::decode(content.as_ref()).context("decode command")
}

fn parse_standard_package_and_version(input: &str) -> std::result::Result<(String, String), Error> {
    let parts: Vec<&str> = input.split('@').collect();
    if parts.len() > 2 {
        return Err(format_err!(
            "package name: {} does not follow the convention of <package>@<version>",
            input
        ));
    }

    let package_name = parts[0].to_string();
    if !MODULE_NAME_REGEXP.is_match(&package_name) {
        return Err(format_err!(
            "package name {} does not match regexp {}",
            package_name,
            MODULE_NAME_REGEXP.as_str()
        ));
    }

    if parts.len() == 1
        || parts
            .get(1)
            .map_or(true, |v| v.is_empty() || *v == "latest")
    {
        return Ok((package_name, "latest".to_string()));
    }

    let version = parts[1];
    if !is_valid_version(&version.replace("v", "")) {
        return Err(format_err!(
            "version '{}' is not valid Semver format",
            version
        ));
    }

    Ok((package_name, version.to_string()))
}

async fn read_http_package(input: &str) -> std::result::Result<Package, anyhow::Error> {
    let body = reqwest::get(input).await?.bytes().await?;

    Package::decode(body).context("decode command")
}

fn is_valid_version(version: &str) -> bool {
    Version::parse(version).is_ok()
}

fn read_block_range(
    pkg: &Package,
    module_name: &str,
) -> std::result::Result<(i64, u64), anyhow::Error> {
    let module = pkg
        .modules
        .as_ref()
        .unwrap()
        .modules
        .iter()
        .find(|m| m.name == module_name)
        .ok_or_else(|| format_err!("module '{}' not found in package", module_name))?;

    let mut input: String = "".to_string();
    if let Some(range) = env::args().nth(4) {
        input = range;
    };

    let (prefix, suffix) = match input.split_once(":") {
        Some((prefix, suffix)) => (prefix.to_string(), suffix.to_string()),
        None => ("".to_string(), input),
    };

    let start: i64 = match prefix.as_str() {
        "" => module.initial_block as i64,
        x if x.starts_with("+") => {
            let block_count = x
                .trim_start_matches("+")
                .parse::<u64>()
                .context("argument <stop> is not a valid integer")?;

            (module.initial_block + block_count) as i64
        }
        x => x
            .parse::<i64>()
            .context("argument <start> is not a valid integer")?,
    };

    let stop: u64 = match suffix.as_str() {
        "" => 0,
        "-" => 0,
        x if x.starts_with("+") => {
            let block_count = x
                .trim_start_matches("+")
                .parse::<u64>()
                .context("argument <stop> is not a valid integer")?;

            start as u64 + block_count
        }
        x => x
            .parse::<u64>()
            .context("argument <stop> is not a valid integer")?,
    };

    return Ok((start, stop));
}
