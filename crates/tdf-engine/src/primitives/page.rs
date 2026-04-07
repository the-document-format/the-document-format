use serde::{Deserialize, Serialize};

use crate::backend::{BackendPointer, BackendTypes};
use crate::primitives::item::ItemTypes;
use crate::store::traits::StoreTypes;

/// A pointer from the page store into the item store.
pub type ItemPointer<B: BackendTypes> = BackendPointer<ItemTypes<B>, B>;

/// A pointer into the page store itself.
pub type PageStorePointer<B: BackendTypes> = BackendPointer<PageTypes<B>, B>;

impl<B: BackendTypes> crate::store::traits::PrimitiveType for ItemPointer<B> {}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct PageTypes<B: BackendTypes>(std::marker::PhantomData<B>);

impl<B: BackendTypes> StoreTypes for PageTypes<B> {
    type Primitive = ItemPointer<B>;
    type Unique = ();
}
