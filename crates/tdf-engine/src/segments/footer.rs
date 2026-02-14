use derive_more::derive::Constructor;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct FooterSegement {
    signatures: Signature,
    checksum: Checksum,
}

impl FooterSegement {
    pub fn new() -> Self {
        Self {
            signatures: Signature::new(),
            checksum: Checksum::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Constructor)]
pub struct Signature {
    // TODO
}

#[derive(Serialize, Deserialize, Debug, Constructor)]
pub struct Checksum {
    // TODO
}
