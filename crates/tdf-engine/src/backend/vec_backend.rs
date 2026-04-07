//! In-memory backend using four Vecs — one per store.

use serde::{Deserialize, Serialize};

use crate::backend::{
    Backend, BackendAccess, BackendPointer, BackendTypes, GetStore, StoreItemCell,
};
use crate::primitives::data::DataTypes;
use crate::primitives::item::ItemTypes;
use crate::primitives::page::PageTypes;
use crate::primitives::signature::SignatureTypes;
use crate::store::traits::StoreTypes;

/// VecBackend's group range: a contiguous index range.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct VecRange {
    pub start: usize,
    pub len: usize,
}

// Concrete in-memory storage (one `Vec` per store).
pub type PageStoreImpl = Vec<StoreItemCell<PageTypes<VecTypes>, VecTypes>>;
pub type ItemStoreImpl = Vec<StoreItemCell<ItemTypes<VecTypes>, VecTypes>>;
pub type DataStoreImpl = Vec<StoreItemCell<DataTypes, VecTypes>>;
pub type SignatureStoreImpl = Vec<StoreItemCell<SignatureTypes, VecTypes>>;

pub type VecInnerStoreImpl<Q> = Vec<StoreItemCell<Q, VecTypes>>;

/// The simplest possible backend: four `Vec`s, one per store.
#[derive(Debug, Default)]
pub struct VecBackend {
    page_store: PageStoreImpl,
    item_store: ItemStoreImpl,
    data_store: DataStoreImpl,
    sig_store: SignatureStoreImpl,
}

macro_rules! impl_get_store {
    ($store_impl:ty, $field:ident) => {
        impl GetStore<$store_impl> for VecBackend {
            fn get_store(&self) -> &$store_impl {
                &self.$field
            }

            fn get_store_mut(&mut self) -> &mut $store_impl {
                &mut self.$field
            }
        }
    };
}

impl_get_store!(PageStoreImpl, page_store);
impl_get_store!(ItemStoreImpl, item_store);
impl_get_store!(DataStoreImpl, data_store);
impl_get_store!(SignatureStoreImpl, sig_store);

impl VecBackend {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<S> BackendAccess<S, VecBackend> for VecBackend
where
    S: StoreTypes,
    VecBackend: GetStore<VecInnerStoreImpl<S>>,
{
    fn push_cell(
        &mut self,
        primitive: S::Primitive,
        unique: S::Unique,
    ) -> BackendPointer<S, VecTypes> {
        let index = self.page_store.len();
        let store = self.get_store_mut();

        let new_ptr = BackendPointer::Single(VecSinglePointer { index, unique });
        let new_cell = StoreItemCell::BackendPointer(new_ptr.clone());

        store.push(new_cell);

        new_ptr
    }

    fn get_cells(
        &self,
        pointer: &BackendPointer<S, <VecBackend as Backend>::Types>,
    ) -> Option<Vec<&StoreItemCell<S, <VecBackend as Backend>::Types>>> {
        let store = self.get_store();

        match pointer {
            BackendPointer::Single(single) => store.get(single.index).map(|cell| vec![cell]),
            BackendPointer::Group(group) => {
                let range = &group.range;
                store
                    .get(range.start..range.start + range.len)
                    .map(|slice| slice.iter().collect())
            }
        }
    }

    fn group_together(
        &mut self,
        items: Vec<BackendPointer<S, <VecBackend as Backend>::Types>>,
    ) -> BackendPointer<S, <VecBackend as Backend>::Types>
    where
        <S as StoreTypes>::Unique: Default,
    {
        todo!()
    }

    fn expand_group(
        &self,
        range: &<<VecBackend as Backend>::Types as BackendTypes>::Group<S>,
    ) -> Vec<BackendPointer<S, <VecBackend as Backend>::Types>> {
        todo!()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
struct VecSinglePointer<S: StoreTypes> {
    index: usize,
    unique: S::Unique,
}

impl<S: StoreTypes> Default for VecSinglePointer<S> {
    //! Maybe use educe for this instead?
    fn default() -> Self {
        Self {
            index: 0,
            unique: S::Unique::default(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
struct VecGroupPointer<S: StoreTypes> {
    range: VecRange,
    uniques: Vec<S::Unique>,
}

impl<S: StoreTypes> Default for VecGroupPointer<S> {
    fn default() -> Self {
        Self {
            range: VecRange { start: 0, len: 0 },
            uniques: vec![],
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct VecTypes;

impl BackendTypes for VecTypes {
    type Single<S: StoreTypes> = VecSinglePointer<S>;
    type Group<S: StoreTypes> = VecGroupPointer<S>;
}

impl Backend for VecBackend {
    type Types = VecTypes;
}
