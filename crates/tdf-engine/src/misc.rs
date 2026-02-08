use derive_more::Constructor;
use serde::{Deserialize, Serialize};

/// A Unix timestamp.
#[derive(Serialize, Deserialize, Constructor, Debug)]
pub struct Instant(u64);

impl Instant {
    pub fn from_instant(instant: std::time::Instant) -> Self {
        Instant(instant.elapsed().as_secs())
    }

    pub fn as_instant(&self) -> std::time::Instant {
        std::time::Instant::now()
    }
}

/// A reference to a page
///
/// TODO
#[derive(Serialize, Deserialize, Constructor, Debug)]
pub struct PageRef();

/// A reference to a spot on a page
///
/// TODO
#[derive(Serialize, Deserialize, Constructor, Debug)]
pub struct PageAnchor();
