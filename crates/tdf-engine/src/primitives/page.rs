use serde::{Deserialize, Serialize};

use crate::backend::{BackendPointer, BackendTypes};
use crate::primitives::item::ItemTypes;
use crate::store::traits::StoreTypes;

/// A pointer from the page store into the item store.
pub type ItemPointer<B: BackendTypes> = BackendPointer<ItemTypes<B>, B>;

/// A pointer into the page store itself.
pub type PageStorePointer<B: BackendTypes> = BackendPointer<PageTypes<B>, B>;

impl<B: BackendTypes> crate::store::traits::PrimitiveType for ItemPointer<B> {}

/// Per-page metadata stored in the segment alongside each page store pointer.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct PageTags {
    pub physical_page_number: Option<u32>,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

/// The primitive stored in the page store for each page: tags + pointer into the item store.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(bound(
    serialize = "B: Serialize",
    deserialize = "ItemPointer<B>: Deserialize<'de>"
))]
pub struct PageStorePrimitive<B: BackendTypes> {
    pub tags: PageTags,
    pub items: ItemPointer<B>,
}

impl<B: BackendTypes> crate::store::traits::PrimitiveType for PageStorePrimitive<B> {}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct PageTypes<B: BackendTypes>(std::marker::PhantomData<B>);

pub type PageUnique = ();

impl<B: BackendTypes> StoreTypes for PageTypes<B> {
    type Primitive = PageStorePrimitive<B>;
    type Unique = PageUnique;
}
