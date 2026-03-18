use derive_more::derive::Constructor;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Constructor)]
pub struct SuffixSegment {
    checksum: Checksum,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Checksum {
    // TODO
}
