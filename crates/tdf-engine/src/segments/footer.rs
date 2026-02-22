use derive_more::derive::Constructor;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct FooterSegment {
    signatures: Signature,
    checksum: Checksum,
}

#[derive(Serialize, Deserialize, Debug, Constructor, Default)]
pub struct Signature {
    // TODO
}

#[derive(Serialize, Deserialize, Debug, Constructor, Default)]
pub struct Checksum {
    // TODO
}
