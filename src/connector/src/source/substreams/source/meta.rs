use risingwave_common::array::{ListValue, StructValue};
use risingwave_common::types::{DataType, Datum, ScalarImpl, StructType};
use crate::parser::additional_columns::get_kafka_header_item_datatype;

#[derive(Debug, Clone)]
pub struct SubstreamsMeta {
    pub block_number: u64,
}

impl SubstreamsMeta {
    // pub(crate) fn extract_headers(&self) -> Option<Datum> {
    //     Some(Datum::from(ScalarImpl::Struct(StructValue::new(vec![
    //         Some(ScalarImpl::Utf8("block_num".to_owned().into())),
    //         Datum::from(self.block_number),
    //     ]))))
    // }
}

