use std::collections::HashMap;

use phf::{Set, phf_set};
use serde_derive::Deserialize;
use serde_with::serde_as;
pub use source::*;
use with_options::WithOptions;

use crate::enforce_secret::EnforceSecret;
use crate::source::SourceProperties;
pub use crate::source::substreams::split::SubstreamsSplit;

mod pb;
pub mod substreams;
pub mod substreams_stream;

pub mod enumerator;
pub mod source;
pub mod split;
mod cursor;
mod key;
mod step;

pub const STREAMINGFAST_SUBSTREAMS_CONNECTOR: &str = "streamingfast-substreams";

#[serde_as]
#[derive(Clone, Debug, Deserialize, WithOptions)]
pub struct SubstreamsProperties {
    #[serde(flatten)]
    pub unknown_fields: HashMap<String, String>,

    #[serde(rename = "substreams.package_url")]
    pub spkg_url: String,

    #[serde(rename = "substreams.output_module")]
    pub output_module: String,

    #[serde(rename = "substreams.endpoint_url")]
    pub endpoint_url: String,

    #[serde(rename = "substreams.start_block")]
    pub start_block: String,

    #[serde(rename = "substreams.stop_block")]
    pub stop_block: String,
}

impl EnforceSecret for SubstreamsProperties {
    const ENFORCE_SECRET_PROPERTIES: Set<&'static str> = phf_set! {};
}

impl crate::source::UnknownFields for SubstreamsProperties {
    fn unknown_fields(&self) -> HashMap<String, String> {
        self.unknown_fields.clone()
    }
}

impl SourceProperties for SubstreamsProperties {
    type Split = SubstreamsSplit;
    type SplitEnumerator = enumerator::client::SubstreamSplitEnumerator;
    type SplitReader = reader::SubstreamsSplitReader;

    const SOURCE_NAME: &'static str = STREAMINGFAST_SUBSTREAMS_CONNECTOR;
}
