use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct TrailerChunk {
    signatures: Signature,
    checksum: Checksum,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Signature {
    // TODO
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Checksum {
    // TODO
}
