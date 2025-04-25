use std::collections::HashMap;
use phf::{phf_set, Set};
use serde_derive::Deserialize;
use serde_with::serde_as;
pub use source::*;
use with_options::WithOptions;
use crate::enforce_secret::EnforceSecret;
use crate::source::SourceProperties;
pub use crate::source::substreams::split::SubstreamsSplit;

mod substreams;
mod substreams_stream;
mod pb;

pub mod source;
pub mod enumerator;
pub mod split;


pub const STREAMINGFAST_SUBSTREAMS_CONNECTOR: &str = "streamingfast-substreams";

#[serde_as]
#[derive(Clone, Debug, Deserialize, WithOptions)]
pub struct SubstreamsProperties {
    #[serde(flatten)]
    pub unknown_fields: HashMap<String, String>,

    #[serde(rename = "substreams.package.url")]
    pub spkg_url: String,

}


impl EnforceSecret for SubstreamsProperties {
    const ENFORCE_SECRET_PROPERTIES: Set<&'static str> = phf_set! {
        ">>>>>>>>>>>>>>>>>??????<<<<<<<<<<<<<<<",
    };
}

impl crate::source::UnknownFields for SubstreamsProperties {
    fn unknown_fields(&self) -> HashMap<String, String> {
        self.unknown_fields.clone()
    }
}

impl SourceProperties for SubstreamsProperties {
    const SOURCE_NAME: &'static str = STREAMINGFAST_SUBSTREAMS_CONNECTOR;
    type Split = SubstreamsSplit;
    type SplitEnumerator = enumerator::client::SubstreamSplitEnumerator;
    type SplitReader = reader::SubstreamsSplitReader;
}