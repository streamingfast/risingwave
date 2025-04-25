use std::env;
use std::process::exit;
use std::sync::Arc;

use anyhow::{Context, Error, format_err};
use async_trait::async_trait;
use futures::future::ok;
use lazy_static::lazy_static;
use prost::Message;
use regex::Regex;
use semver::Version;
use risingwave_common::ensure;
use crate::error::{ConnectorError, ConnectorResult};
use crate::source::substreams::SubstreamsProperties;
use crate::source::substreams::pb::sf::substreams::v1::Package;
use crate::source::substreams::split::SubstreamsSplit;
use crate::source::substreams::substreams_stream::SubstreamsStream;
use crate::source::{SourceEnumeratorContextRef, SplitEnumerator};
use crate::source::google_pubsub::PubsubSplit;
use crate::source::substreams::substreams::SubstreamsEndpoint;


pub struct SubstreamSplitEnumerator {
    pub package_file: String,
    pub module_name: String,
    pub endpoint_url: String,
    pub cursor: Option<String>,
    pub token: String,
}

#[async_trait]
impl SplitEnumerator for SubstreamSplitEnumerator {
    type Properties = SubstreamsProperties;
    type Split = SubstreamsSplit;

    async fn new(
        properties: Self::Properties,
        context: SourceEnumeratorContextRef,
    ) -> ConnectorResult<SubstreamSplitEnumerator> {
        let token_env = env::var("SUBSTREAMS_API_TOKEN").unwrap_or("".to_string());
        let mut token: Option<String> = None;
        ensure!(token_env.len() > 0, "Missing SUBSTREAMS_API_TOKEN env var");

        let mut endpoint_url = "https://mainnet.sol.streamingfast.io:443";
        let package_file = "/Users/cbillett/devel/sf/substreams-sink-sql/db_proto/test/substreams/order/order-v0.1.0.spkg";
        let module_name = "map_output";

        // let package = read_package(&package_file).await?;
        // let block_range = read_block_range(&package, &module_name)?;
        // let endpoint = Arc::new(SubstreamsEndpoint::new(&endpoint_url, token).await?);

        // let cursor: Option<String> = load_persisted_cursor()?;
        let cursor = None;

        // let mut stream = SubstreamsStream::new(
        //     endpoint,
        //     cursor,
        //     package.modules,
        //     module_name.to_string(),
        //     block_range.0,
        //     block_range.1,
        // );

        tracing::info!("SubstreamEnumerator created");

        Ok(
            Self {
                cursor: cursor,
                package_file: package_file.to_string(),
                endpoint_url: endpoint_url.to_string(),
                module_name: module_name.to_string(),
                token:token_env,
            }
        )
    }

    async fn list_splits(&mut self) -> ConnectorResult<Vec<Self::Split>> {
        tracing::info!("Grrrr: SubstreamEnumerator list_splits");
        let splits: Vec<SubstreamsSplit> =  vec![
            SubstreamsSplit{
                package_file: self.package_file.clone(),
                module_name: self.module_name.clone(),
                endpoint_url: self.endpoint_url.clone(),
                cursor: self.cursor.clone(),
                token: self.token.clone(),
                __deprecated_start_offset: None,
                __deprecated_stop_offset: None,
            }
        ];
        Ok(splits)
    }
}

