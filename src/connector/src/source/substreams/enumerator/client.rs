use std::collections::HashMap;
use std::env;
use std::process::exit;
use std::str::FromStr;
use std::sync::{Arc, Mutex};

use anyhow::{Context, Error, format_err};
use async_trait::async_trait;
use futures::future::ok;
use lazy_static::lazy_static;
use prost::Message;
use regex::Regex;
use risingwave_common::ensure;
use semver::Version;
use tokio::task::id;

use crate::error::{ConnectorError, ConnectorResult};
use crate::source::google_pubsub::PubsubSplit;
use crate::source::substreams::cursor::Cursor;
use crate::source::substreams::pb::sf::substreams::v1::Package;
use crate::source::substreams::split::SubstreamsSplit;
use crate::source::substreams::substreams::SubstreamsEndpoint;
use crate::source::substreams::substreams_stream::SubstreamsStream;
use crate::source::substreams::{SubstreamsProperties, cursor};
use crate::source::{SourceEnumeratorContextRef, SplitEnumerator, SplitMetaData};

pub struct SubstreamSplitEnumerator {
    pub package_file: String,
    pub module_name: String,
    pub endpoint_url: String,
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
            package_file: properties.spkg_url,
            endpoint_url: properties.endpoint_url,
            module_name: properties.output_module,
            start_block: start_block,
            stop_block: stop_block,
        })
    }

    async fn list_splits(&mut self) -> ConnectorResult<Vec<Self::Split>> {
        let cursor = Cursor::load()?;
        tracing::info!("Grrr: SubstreamEnumerator list_splits");

        match &cursor {
            None => {
                tracing::info!("Grrr: No cursor found.");
            }
            Some(c) => {
                tracing::info!("Grrr: Cursor found: {:?}", c);
                if self.stop_block > 0 && c.block.number >= self.stop_block {
                    tracing::info!("Grrr: Stop block reached, returning empty split list");
                    return Ok(vec![]);
                }
            }
        } 
        
        let split = SubstreamsSplit::new(
            self.package_file.clone(),
            self.module_name.clone(),
            self.endpoint_url.clone(),
            cursor,
            self.start_block,
            self.stop_block,
        );

        Ok(vec![split])

    }

    async fn on_drop_fragments(&mut self, _fragment_ids: Vec<u32>) -> Result<(), ConnectorError> {
        tracing::info!("Grrr: SubstreamEnumerator on_drop_fragments");
        Ok(())
    }

    /// Do some cleanup work when a backfill fragment is finished, e.g., drop Kafka consumer group.
    async fn on_finish_backfill(&mut self, _fragment_ids: Vec<u32>) -> Result<(), ConnectorError> {
        tracing::info!("Grrr: SubstreamEnumerator on_finish_backfill");
        Ok(())
    }
}
