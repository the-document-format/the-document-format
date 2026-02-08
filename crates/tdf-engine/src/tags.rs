use derive_more::Constructor;
use serde::{Deserialize, Serialize};

/// A tag in a TDF is an optional piece of metadata.
///
/// Right now, it is a thin wrapper around an option.
#[derive(Serialize, Deserialize, Constructor, Debug)]
pub struct Tag<T> {
    data: Option<T>,
}

impl<T> std::ops::Deref for Tag<T> {
    type Target = Option<T>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}
