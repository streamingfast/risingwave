use risingwave_common::array::{ListValue, StructValue};
use risingwave_common::types::{DataType, Datum, DatumRef, ScalarImpl, ScalarRefImpl, StructType};
use crate::parser::additional_columns::get_kafka_header_item_datatype;
use crate::source::nats::source::NatsMeta;

#[derive(Debug, Clone)]
pub struct SubstreamsMeta {
    pub block_number: u64,
}

impl SubstreamsMeta {
    pub fn extract_block_number(&self) -> DatumRef<'_> {
        Some(ScalarRefImpl::Int64(self.block_number as i64))
    }
}