use crate::backend::{BackendPointer, VecRange};
use crate::primitives::item::{ItemPrimitive, ItemUnique};

/// A pointer from the page store into the item store.
pub type ItemPointer = BackendPointer<ItemPrimitive, ItemUnique, VecRange>;

/// A pointer into the page store itself.
pub type PageStorePointer = BackendPointer<ItemPointer, (), VecRange>;

impl crate::store::traits::PrimitiveType for ItemPointer {}
