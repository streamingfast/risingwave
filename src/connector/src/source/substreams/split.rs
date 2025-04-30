use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use risingwave_common::types::JsonbVal;
use serde::{Deserialize, Serialize};

use crate::error::ConnectorResult;
use crate::source::google_pubsub::PubsubSplit;
use crate::source::substreams::cursor::Cursor;
use crate::source::substreams::pb::sf::substreams::v1::Package;
use crate::source::substreams::substreams::SubstreamsEndpoint;
use crate::source::substreams::substreams_stream::SubstreamsStream;
use crate::source::{SplitId, SplitMetaData};

// #[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Hash)]
#[derive(Clone, Serialize, Deserialize, PartialEq, Debug, Hash)]
pub struct SubstreamsSplit {
    pub package_file: String,
    pub module_name: String,
    pub endpoint_url: String,
    pub opaque_cursor: Option<String>,
    pub start_block: u64,
    pub stop_block: u64,
}

impl SubstreamsSplit {
    pub fn new(
        package_file: String,
        module_name: String,
        endpoint_url: String,
        cursor: Option<Cursor>,
        start_block: u64,
        stop_block: u64,
    ) -> Self {

        let opaque_cursor = match cursor {
            None => { None }
            Some(c) => {
                Some(c.to_opaque())
            }
        };

        SubstreamsSplit {
            package_file,
            module_name,
            endpoint_url,
            opaque_cursor,
            start_block,
            stop_block,
        }
    }
}

impl SplitMetaData for SubstreamsSplit {
    fn id(&self) -> SplitId {
        format!(
            "{}-{}-{}",
            self.module_name, self.package_file, self.endpoint_url
        )
        .into()
    }

    fn encode_to_json(&self) -> JsonbVal {
        serde_json::to_value(self.clone()).unwrap().into()
    }

    fn restore_from_json(value: JsonbVal) -> ConnectorResult<Self> {
        let result = serde_json::from_value(value.take()).map_err(Into::into);
        result
    }

    fn update_offset(&mut self, last_seen_offset: String) -> ConnectorResult<()> {
        self.opaque_cursor = Some(last_seen_offset.clone());

        let cursor = Cursor::from_opaque(last_seen_offset.as_str())?;
        cursor.save()?;

        Ok(())
    }
}
