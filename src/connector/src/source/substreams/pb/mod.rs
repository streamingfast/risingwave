use std::fmt::Display;

use crate::source::substreams::pb::sf::substreams::rpc::v2::BlockRange;

include!("pb.rs");

impl Display for BlockRange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}-{})", self.start_block, self.end_block)
    }
}
