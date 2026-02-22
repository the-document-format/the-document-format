use derive_more::derive::Constructor;
use serde::{Deserialize, Serialize};

use crate::segments::Segment;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct FooterSegment {
    signatures: Signature,
    checksum: Checksum,
}

impl Segment for FooterSegment {}

#[derive(Serialize, Deserialize, Debug, Constructor, Default)]
pub struct Signature {
    // TODO
}

#[derive(Serialize, Deserialize, Debug, Constructor, Default)]
pub struct Checksum {
    // TODO
}
