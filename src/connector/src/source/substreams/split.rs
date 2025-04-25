use std::sync::Arc;

use risingwave_common::types::JsonbVal;
use serde::{Deserialize, Serialize};

use crate::error::ConnectorResult;
use crate::source::google_pubsub::PubsubSplit;
use crate::source::substreams::pb::sf::substreams::v1::Package;
use crate::source::substreams::substreams::SubstreamsEndpoint;
use crate::source::substreams::substreams_stream::SubstreamsStream;
use crate::source::{SplitId, SplitMetaData};

// #[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Hash)]
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct SubstreamsSplit {
    pub package_file: String,
    pub module_name: String,
    pub endpoint_url: String,
    pub cursor: Option<String>,
    pub token: String,

    #[serde(rename = "start_offset")]
    #[serde(skip_serializing)]
    pub(crate) __deprecated_start_offset: Option<String>,
    #[serde(rename = "stop_offset")]
    #[serde(skip_serializing)]
    pub(crate) __deprecated_stop_offset: Option<String>,
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
        serde_json::from_value(value.take()).map_err(Into::into)
    }

    fn update_offset(&mut self, _last_seen_offset: String) -> ConnectorResult<()> {
        self.__deprecated_start_offset = None;
        Ok(())
    }
}
