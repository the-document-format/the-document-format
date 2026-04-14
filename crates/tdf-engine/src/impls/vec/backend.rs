//! In-memory backend using four Vecs — one per store.

use serde::{Deserialize, Serialize};

use crate::backend::{
    Backend, BackendAccess, BackendPointer, BackendTypes, GetStore, HasUnique, StoreItemCell,
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

pub type VecInnerStoreImpl<Q: StoreTypes> = Vec<StoreItemCell<Q, VecTypes>>;

/// The simplest possible backend: four `Vec`s, one per store.
#[derive(Debug, Default, Serialize, Deserialize)]
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
        let store = self.get_store_mut();
        let index = store.len();
        store.push(StoreItemCell::StorePrimitive(primitive));
        BackendPointer::Single(VecSinglePointer { index, unique })
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
        if items.is_empty() {
            return BackendPointer::Group(VecGroupPointer::default());
        }

        let start = match &items[0] {
            BackendPointer::Single(s) => s.index,
            BackendPointer::Group(_) => todo!("nested recursive groups in group_together"),
        };

        let uniques = items
            .iter()
            .map(|ptr| match ptr {
                BackendPointer::Single(s) => s.unique.clone(),
                BackendPointer::Group(_) => todo!("nested recursive groups in group_together"),
            })
            .collect();

        BackendPointer::Group(VecGroupPointer {
            range: VecRange {
                start,
                len: items.len(),
            },
            uniques,
        })
    }

    fn expand_group(
        &self,
        group: &<<VecBackend as Backend>::Types as BackendTypes>::Group<S>,
    ) -> Vec<BackendPointer<S, <VecBackend as Backend>::Types>> {
        group
            .uniques
            .iter()
            .enumerate()
            .map(|(i, unique)| {
                BackendPointer::Single(VecSinglePointer {
                    index: group.range.start + i,
                    unique: unique.clone(),
                })
            })
            .collect()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct VecSinglePointer<S: StoreTypes> {
    pub index: usize,
    pub unique: S::Unique,
}

impl<S: StoreTypes> HasUnique<S::Unique> for VecSinglePointer<S> {
    fn unique(&self) -> S::Unique {
        self.unique.clone()
    }
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
pub struct VecGroupPointer<S: StoreTypes> {
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

    type PageStore = PageStoreImpl;
    type ItemStore = ItemStoreImpl;
    type DataStore = DataStoreImpl;
    type SigStore = SignatureStoreImpl;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::BackendAccess;
    use crate::primitives::item::{
        ItemPrimitive, ItemTypes, ItemUnique, Position, Shape, ShapeKind,
    };

    #[test]
    fn vec_group_pointer_default_is_empty() {
        let d = VecGroupPointer::<ItemTypes<VecTypes>>::default();
        assert_eq!(d.range.start, 0);
        assert_eq!(d.range.len, 0);
        assert!(d.uniques.is_empty());
    }

    #[test]
    fn group_together_captures_start_and_uniques() {
        let mut backend = VecBackend::new();

        let u0 = ItemUnique {
            position: Position { x: 1, y: 2 },
            ..Default::default()
        };
        let u1 = ItemUnique {
            position: Position { x: 3, y: 4 },
            ..Default::default()
        };

        // Push two items so they land at indices 0 and 1
        let ptr0 = backend.push_cell(
            ItemPrimitive::Shape(Shape {
                kind: ShapeKind::Circle,
            }),
            u0.clone(),
        );
        let ptr1 = backend.push_cell(
            ItemPrimitive::Shape(Shape {
                kind: ShapeKind::Rectangle,
            }),
            u1.clone(),
        );

        let group = <VecBackend as BackendAccess<ItemTypes<VecTypes>, VecBackend>>::group_together(
            &mut backend,
            vec![ptr0, ptr1],
        );

        match group {
            BackendPointer::Group(g) => {
                assert_eq!(g.range.start, 0);
                assert_eq!(g.range.len, 2);
                assert_eq!(g.uniques[0], u0);
                assert_eq!(g.uniques[1], u1);
            }
            _ => panic!("expected a Group pointer"),
        }
    }
}
