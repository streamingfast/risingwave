use std::env;
use std::process::exit;
use std::sync::Arc;

use anyhow::{Context, Error, format_err};
use async_trait::async_trait;
use futures::future::ok;
use lazy_static::lazy_static;
use prost::Message;
use regex::Regex;
use risingwave_common::ensure;
use semver::Version;

use crate::error::{ConnectorError, ConnectorResult};
use crate::source::google_pubsub::PubsubSplit;
use crate::source::substreams::SubstreamsProperties;
use crate::source::substreams::pb::sf::substreams::v1::Package;
use crate::source::substreams::split::SubstreamsSplit;
use crate::source::substreams::substreams::SubstreamsEndpoint;
use crate::source::substreams::substreams_stream::SubstreamsStream;
use crate::source::{SourceEnumeratorContextRef, SplitEnumerator};

pub struct SubstreamSplitEnumerator {
    pub package_file: String,
    pub module_name: String,
    pub endpoint_url: String,
    pub cursor: Option<String>,
    pub start_block: u64,
    pub stop_block: u64,
}

#[async_trait]
impl SplitEnumerator for SubstreamSplitEnumerator {
    type Properties = SubstreamsProperties;
    type Split = SubstreamsSplit;

    async fn new(
        properties: Self::Properties,
        context: SourceEnumeratorContextRef,
    ) -> ConnectorResult<SubstreamSplitEnumerator> {
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

        let start_block = properties
            .start_block
            .parse::<u64>()
            .context("Failed to parse start_block as u64")?;

        let stop_block = properties
            .stop_block
            .parse::<u64>()
            .context("Failed to parse stop_block as u64")?;

        tracing::info!("SubstreamEnumerator created");

        Ok(Self {
            cursor,
            package_file: properties.spkg_url,
            endpoint_url: properties.endpoint_url,
            module_name: properties.output_module,
            start_block: start_block,
            stop_block: stop_block,
        })
    }

    async fn list_splits(&mut self) -> ConnectorResult<Vec<Self::Split>> {
        tracing::info!("Grrrr: SubstreamEnumerator list_splits");
        let splits: Vec<SubstreamsSplit> = vec![SubstreamsSplit {
            package_file: self.package_file.clone(),
            module_name: self.module_name.clone(),
            endpoint_url: self.endpoint_url.clone(),
            cursor: self.cursor.clone(),
            start_block: self.start_block,
            stop_block: self.stop_block,
            __deprecated_start_offset: None,
            __deprecated_stop_offset: None,
        }];
        Ok(splits)
    }
}
