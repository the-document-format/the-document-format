//! In-memory backend using four Vecs — one per store.

use std::marker::PhantomData;

use serde::{Deserialize, Serialize};

use crate::backend::{Backend, BackendAccess, BackendPointer, StoreItemCell};
use crate::primitives::data::DataTypes;
use crate::primitives::item::ItemTypes;
use crate::primitives::page::PageTypes;
use crate::primitives::signature::SignatureTypes;
use crate::primitives::{item::ItemUnique, signature::SignatureUnique};
use crate::store::frontend::append_only::AppendOnlyFrontend;
use crate::store::frontend::optimized::OptimizedFrontend;

/// VecBackend's group range: a contiguous index range.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct VecRange {
    pub start: usize,
    pub len: usize,
}

// Concrete store type aliases used for VecBackend's internal storage.
pub type PageStore = AppendOnlyFrontend<PageTypes<VecBackend>, VecBackend>;
pub type ItemStore = OptimizedFrontend<ItemTypes, VecBackend>;
pub type DataStore = OptimizedFrontend<DataTypes, VecBackend>;
pub type SignatureStore = AppendOnlyFrontend<SignatureTypes, VecBackend>;

/// The simplest possible backend: four `Vec`s, one per store.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct VecBackend {
    page_store: Vec<StoreItemCell<PageStore, VecBackend>>,
    item_store: Vec<StoreItemCell<ItemStore, VecBackend>>,
    data_store: Vec<StoreItemCell<DataStore, VecBackend>>,
    sig_store: Vec<StoreItemCell<SignatureStore, VecBackend>>,
}

impl VecBackend {
    pub fn new() -> Self {
        Self::default()
    }
}

impl BackendAccess<PageStore, VecBackend> for VecBackend {
    fn push_cell(
        &mut self,
        item: StoreItemCell<PageStore, VecBackend>,
    ) -> BackendPointer<PageStore, VecBackend> {
        let index = self.page_store.len();
        self.page_store.push(item);
        BackendPointer::Single {
            index,
            unique: (),
            _phantom: PhantomData,
        }
    }
    fn get_cell(
        &self,
        pointer: &BackendPointer<PageStore, VecBackend>,
    ) -> Option<&StoreItemCell<PageStore, VecBackend>> {
        match pointer {
            BackendPointer::Single { index, .. } => self.page_store.get(*index),
            BackendPointer::Group { .. } => None,
        }
    }
    fn group_together(
        &mut self,
        items: Vec<BackendPointer<PageStore, VecBackend>>,
    ) -> BackendPointer<PageStore, VecBackend> {
        let uniques: Vec<()> = items.iter().map(|_| ()).collect();
        let start = self.page_store.len();
        for ptr in &items {
            self.push_cell(StoreItemCell::BackendPointer(ptr.clone()));
        }
        BackendPointer::Group {
            range: VecRange {
                start,
                len: items.len(),
            },
            uniques,
            _phantom: PhantomData,
        }
    }
    fn expand_group(
        &self,
        range: &VecRange,
        uniques: Vec<()>,
    ) -> Vec<BackendPointer<PageStore, VecBackend>> {
        (range.start..range.start + range.len)
            .zip(uniques)
            .map(|(i, u)| BackendPointer::Single {
                index: i,
                unique: u,
                _phantom: PhantomData,
            })
            .collect()
    }
}

impl BackendAccess<ItemStore, VecBackend> for VecBackend {
    fn push_cell(
        &mut self,
        item: StoreItemCell<ItemStore, VecBackend>,
    ) -> BackendPointer<ItemStore, VecBackend> {
        let index = self.item_store.len();
        self.item_store.push(item);
        BackendPointer::Single {
            index,
            unique: ItemUnique::default(),
            _phantom: PhantomData,
        }
    }
    fn get_cell(
        &self,
        pointer: &BackendPointer<ItemStore, VecBackend>,
    ) -> Option<&StoreItemCell<ItemStore, VecBackend>> {
        match pointer {
            BackendPointer::Single { index, .. } => self.item_store.get(*index),
            BackendPointer::Group { .. } => None,
        }
    }
    fn group_together(
        &mut self,
        items: Vec<BackendPointer<ItemStore, VecBackend>>,
    ) -> BackendPointer<ItemStore, VecBackend> {
        let uniques: Vec<ItemUnique> = items
            .iter()
            .map(|p| match p {
                BackendPointer::Single { unique, .. } => unique.clone(),
                BackendPointer::Group { .. } => ItemUnique::default(),
            })
            .collect();
        let start = self.item_store.len();
        for ptr in &items {
            self.push_cell(StoreItemCell::BackendPointer(ptr.clone()));
        }
        BackendPointer::Group {
            range: VecRange {
                start,
                len: items.len(),
            },
            uniques,
            _phantom: PhantomData,
        }
    }
    fn expand_group(
        &self,
        range: &VecRange,
        uniques: Vec<ItemUnique>,
    ) -> Vec<BackendPointer<ItemStore, VecBackend>> {
        (range.start..range.start + range.len)
            .zip(uniques)
            .map(|(i, u)| BackendPointer::Single {
                index: i,
                unique: u,
                _phantom: PhantomData,
            })
            .collect()
    }
}

impl BackendAccess<DataStore, VecBackend> for VecBackend {
    fn push_cell(
        &mut self,
        item: StoreItemCell<DataStore, VecBackend>,
    ) -> BackendPointer<DataStore, VecBackend> {
        let index = self.data_store.len();
        self.data_store.push(item);
        BackendPointer::Single {
            index,
            unique: (),
            _phantom: PhantomData,
        }
    }
    fn get_cell(
        &self,
        pointer: &BackendPointer<DataStore, VecBackend>,
    ) -> Option<&StoreItemCell<DataStore, VecBackend>> {
        match pointer {
            BackendPointer::Single { index, .. } => self.data_store.get(*index),
            BackendPointer::Group { .. } => None,
        }
    }
    fn group_together(
        &mut self,
        items: Vec<BackendPointer<DataStore, VecBackend>>,
    ) -> BackendPointer<DataStore, VecBackend> {
        let uniques: Vec<()> = items.iter().map(|_| ()).collect();
        let start = self.data_store.len();
        for ptr in &items {
            self.push_cell(StoreItemCell::BackendPointer(ptr.clone()));
        }
        BackendPointer::Group {
            range: VecRange {
                start,
                len: items.len(),
            },
            uniques,
            _phantom: PhantomData,
        }
    }
    fn expand_group(
        &self,
        range: &VecRange,
        uniques: Vec<()>,
    ) -> Vec<BackendPointer<DataStore, VecBackend>> {
        (range.start..range.start + range.len)
            .zip(uniques)
            .map(|(i, u)| BackendPointer::Single {
                index: i,
                unique: u,
                _phantom: PhantomData,
            })
            .collect()
    }
}

impl BackendAccess<SignatureStore, VecBackend> for VecBackend {
    fn push_cell(
        &mut self,
        item: StoreItemCell<SignatureStore, VecBackend>,
    ) -> BackendPointer<SignatureStore, VecBackend> {
        let index = self.sig_store.len();
        self.sig_store.push(item);
        BackendPointer::Single {
            index,
            unique: SignatureUnique,
            _phantom: PhantomData,
        }
    }
    fn get_cell(
        &self,
        pointer: &BackendPointer<SignatureStore, VecBackend>,
    ) -> Option<&StoreItemCell<SignatureStore, VecBackend>> {
        match pointer {
            BackendPointer::Single { index, .. } => self.sig_store.get(*index),
            BackendPointer::Group { .. } => None,
        }
    }
    fn group_together(
        &mut self,
        items: Vec<BackendPointer<SignatureStore, VecBackend>>,
    ) -> BackendPointer<SignatureStore, VecBackend> {
        let uniques: Vec<SignatureUnique> = items
            .iter()
            .map(|p| match p {
                BackendPointer::Single { unique, .. } => unique.clone(),
                BackendPointer::Group { .. } => SignatureUnique,
            })
            .collect();
        let start = self.sig_store.len();
        for ptr in &items {
            self.push_cell(StoreItemCell::BackendPointer(ptr.clone()));
        }
        BackendPointer::Group {
            range: VecRange {
                start,
                len: items.len(),
            },
            uniques,
            _phantom: PhantomData,
        }
    }
    fn expand_group(
        &self,
        range: &VecRange,
        uniques: Vec<SignatureUnique>,
    ) -> Vec<BackendPointer<SignatureStore, VecBackend>> {
        (range.start..range.start + range.len)
            .zip(uniques)
            .map(|(i, u)| BackendPointer::Single {
                index: i,
                unique: u,
                _phantom: PhantomData,
            })
            .collect()
    }
}

impl Backend for VecBackend {
    type Range = VecRange;

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
