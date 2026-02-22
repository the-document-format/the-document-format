use derive_more::derive::Constructor;
use serde::{Deserialize, Serialize};

use crate::segments::Segment;

#[derive(Serialize, Deserialize, Debug, Constructor)]
pub struct SuffixSegment {
    checksum: Checksum,
}

impl Segment for SuffixSegment {}

#[derive(Serialize, Deserialize, Debug)]
pub struct Checksum {
    // TODO
}
