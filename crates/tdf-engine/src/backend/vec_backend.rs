//! In-memory backend using four Vecs — one per store.

use std::marker::PhantomData;

use serde::{Deserialize, Serialize};

use crate::backend::{Backend, BackendAccess, BackendPointer, BackendTypes, StoreItemCell};
use crate::primitives::data::DataTypes;
use crate::primitives::item::ItemTypes;
use crate::primitives::page::PageTypes;
use crate::primitives::signature::SignatureTypes;
use crate::primitives::{item::ItemUnique, signature::SignatureUnique};
use crate::store::frontend::append_only::AppendOnlyFrontend;
use crate::store::frontend::optimized::OptimizedFrontend;
use crate::store::traits::{StoreTypes};

/// VecBackend's group range: a contiguous index range.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct VecRange {
    pub start: usize,
    pub len: usize,
}

// Concrete store type aliases used for VecBackend's internal storage.
pub type PageStore = AppendOnlyFrontend<PageTypes<VecTypes>, VecBackend>;
pub type ItemStore = OptimizedFrontend<ItemTypes<VecTypes>, VecBackend>;
pub type DataStore = OptimizedFrontend<DataTypes, VecBackend>;
pub type SignatureStore = AppendOnlyFrontend<SignatureTypes, VecBackend>;

/// The simplest possible backend: four `Vec`s, one per store.
#[derive(Debug, Default)]
pub struct VecBackend {
    page_store: Vec<StoreItemCell<PageTypes<VecTypes>, VecTypes>>,
    item_store: Vec<StoreItemCell<ItemTypes<VecTypes>, VecTypes>>,
    data_store: Vec<StoreItemCell<DataTypes, VecTypes>>,
    sig_store: Vec<StoreItemCell<SignatureTypes, VecTypes>>,
}

impl VecBackend {
    pub fn new() -> Self {
        Self::default()
    }
}

impl BackendAccess<PageTypes<VecTypes>, VecBackend> for VecBackend {
    fn push_cell(
        &mut self,
        item: StoreItemCell<PageTypes<VecTypes>, VecTypes>,
    ) -> BackendPointer<PageTypes<VecTypes>, VecTypes> {
        let index = self.page_store.len();
        self.page_store.push(item);
        BackendPointer::<PageTypes<VecTypes>, VecTypes>::Single(
            VecSinglePointer::<PageTypes<VecTypes>> {
                index: index,
                unique: (),
            }
        )
    }
    fn get_cell(
        &self,
        pointer: &BackendPointer<PageTypes<VecTypes>, VecTypes>,
    ) -> Option<&StoreItemCell<PageTypes<VecTypes>, VecTypes>> {
        match pointer {
            BackendPointer::Single(VecSinglePointer::<PageTypes<VecTypes>> { index, .. }) => self.page_store.get(*index),
            BackendPointer::Group { .. } => None,
        }
    }
    fn group_together(
        &mut self,
        items: Vec<BackendPointer<PageTypes<VecTypes>, VecTypes>>,
    ) -> BackendPointer<PageTypes<VecTypes>, VecTypes> {
        let uniques: Vec<()> = items.iter().map(|_| ()).collect();
        let start = self.page_store.len();
        for ptr in &items {
            self.push_cell(StoreItemCell::BackendPointer(ptr.clone()));
        }
        BackendPointer::Group(VecGroupPointer::<PageTypes<VecTypes>> {
            range: VecRange {
                start: start,
                len: items.len(),
            },
            uniques,
        })
    }
    fn expand_group(
        &self,
        group: &VecGroupPointer<PageTypes<VecTypes>>,
    ) -> Vec<BackendPointer<PageTypes<VecTypes>, VecTypes>> {
        (group.range.start..group.range.start + group.range.len)
            .zip(&group.uniques)
            .map(|(i, u)| BackendPointer::Single(VecSinglePointer::<PageTypes<VecTypes>> {
                index: i,
                unique: *u,
            }))
            .collect()
    }
}

// impl BackendAccess<ItemTypes<VecTypes>, VecTypes> for VecBackend {
//     fn push_cell(
//         &mut self,
//         item: StoreItemCell<ItemStore, VecBackend>,
//     ) -> BackendPointer<ItemStore, VecBackend> {
//         let index = self.item_store.len();
//         self.item_store.push(item);
//         BackendPointer::Single {
//             index,
//             unique: ItemUnique::default(),
//             _phantom: PhantomData,
//         }
//     }
//     fn get_cell(
//         &self,
//         pointer: &BackendPointer<ItemStore, VecBackend>,
//     ) -> Option<&StoreItemCell<ItemStore, VecBackend>> {
//         match pointer {
//             BackendPointer::Single { index, .. } => self.item_store.get(*index),
//             BackendPointer::Group { .. } => None,
//         }
//     }
//     fn group_together(
//         &mut self,
//         items: Vec<BackendPointer<ItemStore, VecBackend>>,
//     ) -> BackendPointer<ItemStore, VecBackend> {
//         let uniques: Vec<ItemUnique> = items
//             .iter()
//             .map(|p| match p {
//                 BackendPointer::Single { unique, .. } => unique.clone(),
//                 BackendPointer::Group { .. } => ItemUnique::default(),
//             })
//             .collect();
//         let start = self.item_store.len();
//         for ptr in &items {
//             self.push_cell(StoreItemCell::BackendPointer(ptr.clone()));
//         }
//         BackendPointer::Group {
//             range: VecRange {
//                 start,
//                 len: items.len(),
//             },
//             uniques,
//             _phantom: PhantomData,
//         }
//     }
//     fn expand_group(
//         &self,
//         range: &VecRange,
//         uniques: Vec<ItemUnique>,
//     ) -> Vec<BackendPointer<ItemStore, VecBackend>> {
//         (range.start..range.start + range.len)
//             .zip(uniques)
//             .map(|(i, u)| BackendPointer::Single {
//                 index: i,
//                 unique: u,
//                 _phantom: PhantomData,
//             })
//             .collect()
//     }
// }

// impl BackendAccess<DataStore, VecBackend> for VecBackend {
//     fn push_cell(
//         &mut self,
//         item: StoreItemCell<DataStore, VecBackend>,
//     ) -> BackendPointer<DataStore, VecBackend> {
//         let index = self.data_store.len();
//         self.data_store.push(item);
//         BackendPointer::Single {
//             index,
//             unique: (),
//             _phantom: PhantomData,
//         }
//     }
//     fn get_cell(
//         &self,
//         pointer: &BackendPointer<DataStore, VecBackend>,
//     ) -> Option<&StoreItemCell<DataStore, VecBackend>> {
//         match pointer {
//             BackendPointer::Single { index, .. } => self.data_store.get(*index),
//             BackendPointer::Group { .. } => None,
//         }
//     }
//     fn group_together(
//         &mut self,
//         items: Vec<BackendPointer<DataStore, VecBackend>>,
//     ) -> BackendPointer<DataStore, VecBackend> {
//         let uniques: Vec<()> = items.iter().map(|_| ()).collect();
//         let start = self.data_store.len();
//         for ptr in &items {
//             self.push_cell(StoreItemCell::BackendPointer(ptr.clone()));
//         }
//         BackendPointer::Group {
//             range: VecRange {
//                 start,
//                 len: items.len(),
//             },
//             uniques,
//             _phantom: PhantomData,
//         }
//     }
//     fn expand_group(
//         &self,
//         range: &VecRange,
//         uniques: Vec<()>,
//     ) -> Vec<BackendPointer<DataStore, VecBackend>> {
//         (range.start..range.start + range.len)
//             .zip(uniques)
//             .map(|(i, u)| BackendPointer::Single {
//                 index: i,
//                 unique: u,
//                 _phantom: PhantomData,
//             })
//             .collect()
//     }
// }

// impl BackendAccess<SignatureStore, VecBackend> for VecBackend {
//     fn push_cell(
//         &mut self,
//         item: StoreItemCell<SignatureStore, VecBackend>,
//     ) -> BackendPointer<SignatureStore, VecBackend> {
//         let index = self.sig_store.len();
//         self.sig_store.push(item);
//         BackendPointer::Single {
//             index,
//             unique: SignatureUnique,
//             _phantom: PhantomData,
//         }
//     }
//     fn get_cell(
//         &self,
//         pointer: &BackendPointer<SignatureStore, VecBackend>,
//     ) -> Option<&StoreItemCell<SignatureStore, VecBackend>> {
//         match pointer {
//             BackendPointer::Single { index, .. } => self.sig_store.get(*index),
//             BackendPointer::Group { .. } => None,
//         }
//     }
//     fn group_together(
//         &mut self,
//         items: Vec<BackendPointer<SignatureStore, VecBackend>>,
//     ) -> BackendPointer<SignatureStore, VecBackend> {
//         let uniques: Vec<SignatureUnique> = items
//             .iter()
//             .map(|p| match p {
//                 BackendPointer::Single { unique, .. } => unique.clone(),
//                 BackendPointer::Group { .. } => SignatureUnique,
//             })
//             .collect();
//         let start = self.sig_store.len();
//         for ptr in &items {
//             self.push_cell(StoreItemCell::BackendPointer(ptr.clone()));
//         }
//         BackendPointer::Group {
//             range: VecRange {
//                 start,
//                 len: items.len(),
//             },
//             uniques,
//             _phantom: PhantomData,
//         }
//     }
//     fn expand_group(
//         &self,
//         range: &VecRange,
//         uniques: Vec<SignatureUnique>,
//     ) -> Vec<BackendPointer<SignatureStore, VecBackend>> {
//         (range.start..range.start + range.len)
//             .zip(uniques)
//             .map(|(i, u)| BackendPointer::Single {
//                 index: i,
//                 unique: u,
//                 _phantom: PhantomData,
//             })
//             .collect()
//     }
// }

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
struct VecSinglePointer<S: StoreTypes> {
    index: usize,
    unique: S::Unique,
}

impl <S: StoreTypes>Default for VecSinglePointer<S> {
    //! Maybe use educe for this instead?
    fn default() -> Self {
        Self {
            index: 0, unique: S::Unique::default()
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
struct VecGroupPointer<S: StoreTypes> {
    range: VecRange,
    uniques: Vec<S::Unique>,
}

impl <S:StoreTypes>Default for VecGroupPointer<S> {
    fn default() -> Self {
        Self {
            range: VecRange{start: 0, len: 0},
            uniques: vec![],
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
struct VecTypes();

impl BackendTypes for VecTypes {
    type Single<S: StoreTypes> = VecSinglePointer<S>;
    type Group<S: StoreTypes> = VecGroupPointer<S>;
}

impl Backend for VecBackend {
    type Types = VecTypes;

    fn page_store_size(&self) -> usize {
        self.page_store.len()
    }
    fn item_store_size(&self) -> usize {
        self.item_store.len()
    }
    fn data_store_size(&self) -> usize {
        self.data_store.len()
    }
    fn sig_store_size(&self) -> usize {
        self.sig_store.len()
    }
}

