//! In-memory backend using four Vecs — one per store.

use std::marker::PhantomData;

use serde::{Deserialize, Serialize};

use crate::backend::{Backend, BackendAccess, BackendPointer, StoreItemCell};
use crate::primitives::{
    data::DataPrimitive,
    item::{ItemPrimitive, ItemUnique},
    page::ItemPointer,
    signature::{SignaturePrimitive, SignatureUnique},
};

/// VecBackend's group representation: a contiguous index range.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct VecRange {
    pub start: usize,
    pub len: usize,
}

/// The simplest possible backend: four `Vec`s, one per store.
#[derive(Debug, Default)]
pub struct VecBackend {
    page_store: Vec<StoreItemCell<ItemPointer, (), VecRange>>,
    item_store: Vec<StoreItemCell<ItemPrimitive, ItemUnique, VecRange>>,
    data_store: Vec<StoreItemCell<DataPrimitive, (), VecRange>>,
    sig_store: Vec<StoreItemCell<SignaturePrimitive, SignatureUnique, VecRange>>,
}

impl VecBackend {
    pub fn new() -> Self {
        Self::default()
    }
}

impl BackendAccess<ItemPointer, ()> for VecBackend {
    type Group = VecRange;

    fn push_cell(
        &mut self,
        item: StoreItemCell<ItemPointer, (), VecRange>,
    ) -> BackendPointer<ItemPointer, (), VecRange> {
        let index = self.page_store.len();
        self.page_store.push(item);
        BackendPointer::new(index)
    }
    fn get_cell(
        &self,
        pointer: &BackendPointer<ItemPointer, (), VecRange>,
    ) -> Option<&StoreItemCell<ItemPointer, (), VecRange>> {
        match pointer {
            BackendPointer::Single { index, .. } => self.page_store.get(*index),
            BackendPointer::Group { .. } => None,
        }
    }
    fn group_together(
        &mut self,
        items: Vec<BackendPointer<ItemPointer, (), VecRange>>,
    ) -> BackendPointer<ItemPointer, (), VecRange> {
        let uniques: Vec<()> = items
            .iter()
            .map(|p| match p {
                BackendPointer::Single { unique, .. } => *unique,
                BackendPointer::Group { .. } => (),
            })
            .collect();
        let start = self.page_store.len();
        for ptr in &items {
            self.push_cell(StoreItemCell::BackendPointer(ptr.clone()));
        }
        BackendPointer::Group {
            group: VecRange {
                start,
                len: items.len(),
            },
            unique: uniques,
            _phantom: PhantomData,
        }
    }
    fn expand_group(
        &self,
        group: &VecRange,
        uniques: Vec<()>,
    ) -> Vec<BackendPointer<ItemPointer, (), VecRange>> {
        (group.start..group.start + group.len)
            .zip(uniques)
            .map(|(i, u)| BackendPointer::Single {
                index: i,
                unique: u,
                _phantom: PhantomData,
            })
            .collect()
    }
}

impl BackendAccess<ItemPrimitive, ItemUnique> for VecBackend {
    type Group = VecRange;

    fn push_cell(
        &mut self,
        item: StoreItemCell<ItemPrimitive, ItemUnique, VecRange>,
    ) -> BackendPointer<ItemPrimitive, ItemUnique, VecRange> {
        let index = self.item_store.len();
        self.item_store.push(item);
        BackendPointer::new(index)
    }
    fn get_cell(
        &self,
        pointer: &BackendPointer<ItemPrimitive, ItemUnique, VecRange>,
    ) -> Option<&StoreItemCell<ItemPrimitive, ItemUnique, VecRange>> {
        match pointer {
            BackendPointer::Single { index, .. } => self.item_store.get(*index),
            BackendPointer::Group { .. } => None,
        }
    }
    fn group_together(
        &mut self,
        items: Vec<BackendPointer<ItemPrimitive, ItemUnique, VecRange>>,
    ) -> BackendPointer<ItemPrimitive, ItemUnique, VecRange> {
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
            group: VecRange {
                start,
                len: items.len(),
            },
            unique: uniques,
            _phantom: PhantomData,
        }
    }
    fn expand_group(
        &self,
        group: &VecRange,
        uniques: Vec<ItemUnique>,
    ) -> Vec<BackendPointer<ItemPrimitive, ItemUnique, VecRange>> {
        (group.start..group.start + group.len)
            .zip(uniques)
            .map(|(i, u)| BackendPointer::Single {
                index: i,
                unique: u,
                _phantom: PhantomData,
            })
            .collect()
    }
}

impl BackendAccess<DataPrimitive, ()> for VecBackend {
    type Group = VecRange;

    fn push_cell(
        &mut self,
        item: StoreItemCell<DataPrimitive, (), VecRange>,
    ) -> BackendPointer<DataPrimitive, (), VecRange> {
        let index = self.data_store.len();
        self.data_store.push(item);
        BackendPointer::new(index)
    }
    fn get_cell(
        &self,
        pointer: &BackendPointer<DataPrimitive, (), VecRange>,
    ) -> Option<&StoreItemCell<DataPrimitive, (), VecRange>> {
        match pointer {
            BackendPointer::Single { index, .. } => self.data_store.get(*index),
            BackendPointer::Group { .. } => None,
        }
    }
    fn group_together(
        &mut self,
        items: Vec<BackendPointer<DataPrimitive, (), VecRange>>,
    ) -> BackendPointer<DataPrimitive, (), VecRange> {
        let uniques: Vec<()> = items
            .iter()
            .map(|p| match p {
                BackendPointer::Single { unique, .. } => *unique,
                BackendPointer::Group { .. } => (),
            })
            .collect();
        let start = self.data_store.len();
        for ptr in &items {
            self.push_cell(StoreItemCell::BackendPointer(ptr.clone()));
        }
        BackendPointer::Group {
            group: VecRange {
                start,
                len: items.len(),
            },
            unique: uniques,
            _phantom: PhantomData,
        }
    }
    fn expand_group(
        &self,
        group: &VecRange,
        uniques: Vec<()>,
    ) -> Vec<BackendPointer<DataPrimitive, (), VecRange>> {
        (group.start..group.start + group.len)
            .zip(uniques)
            .map(|(i, u)| BackendPointer::Single {
                index: i,
                unique: u,
                _phantom: PhantomData,
            })
            .collect()
    }
}

impl BackendAccess<SignaturePrimitive, SignatureUnique> for VecBackend {
    type Group = VecRange;

    fn push_cell(
        &mut self,
        item: StoreItemCell<SignaturePrimitive, SignatureUnique, VecRange>,
    ) -> BackendPointer<SignaturePrimitive, SignatureUnique, VecRange> {
        let index = self.sig_store.len();
        self.sig_store.push(item);
        BackendPointer::new(index)
    }
    fn get_cell(
        &self,
        pointer: &BackendPointer<SignaturePrimitive, SignatureUnique, VecRange>,
    ) -> Option<&StoreItemCell<SignaturePrimitive, SignatureUnique, VecRange>> {
        match pointer {
            BackendPointer::Single { index, .. } => self.sig_store.get(*index),
            BackendPointer::Group { .. } => None,
        }
    }
    fn group_together(
        &mut self,
        items: Vec<BackendPointer<SignaturePrimitive, SignatureUnique, VecRange>>,
    ) -> BackendPointer<SignaturePrimitive, SignatureUnique, VecRange> {
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
            group: VecRange {
                start,
                len: items.len(),
            },
            unique: uniques,
            _phantom: PhantomData,
        }
    }
    fn expand_group(
        &self,
        group: &VecRange,
        uniques: Vec<SignatureUnique>,
    ) -> Vec<BackendPointer<SignaturePrimitive, SignatureUnique, VecRange>> {
        (group.start..group.start + group.len)
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
    fn push_page(
        &mut self,
        item: StoreItemCell<ItemPointer, (), VecRange>,
    ) -> BackendPointer<ItemPointer, (), VecRange> {
        todo!()
    }
    fn get_page(
        &self,
        pointer: &BackendPointer<ItemPointer, (), VecRange>,
    ) -> Option<&StoreItemCell<ItemPointer, (), VecRange>> {
        todo!()
    }
    fn page_store_size(&self) -> usize {
        self.page_store.len()
    }

    fn push_item(
        &mut self,
        item: StoreItemCell<ItemPrimitive, ItemUnique, VecRange>,
    ) -> BackendPointer<ItemPrimitive, ItemUnique, VecRange> {
        todo!()
    }
    fn get_item(
        &self,
        pointer: &BackendPointer<ItemPrimitive, ItemUnique, VecRange>,
    ) -> Option<&StoreItemCell<ItemPrimitive, ItemUnique, VecRange>> {
        todo!()
    }
    fn item_store_size(&self) -> usize {
        self.item_store.len()
    }

    fn push_data(
        &mut self,
        item: StoreItemCell<DataPrimitive, (), VecRange>,
    ) -> BackendPointer<DataPrimitive, (), VecRange> {
        todo!()
    }
    fn get_data(
        &self,
        pointer: &BackendPointer<DataPrimitive, (), VecRange>,
    ) -> Option<&StoreItemCell<DataPrimitive, (), VecRange>> {
        todo!()
    }
    fn data_store_size(&self) -> usize {
        self.data_store.len()
    }

    fn push_sig(
        &mut self,
        item: StoreItemCell<SignaturePrimitive, SignatureUnique, VecRange>,
    ) -> BackendPointer<SignaturePrimitive, SignatureUnique, VecRange> {
        todo!()
    }
    fn get_sig(
        &self,
        pointer: &BackendPointer<SignaturePrimitive, SignatureUnique, VecRange>,
    ) -> Option<&StoreItemCell<SignaturePrimitive, SignatureUnique, VecRange>> {
        todo!()
    }
    fn sig_store_size(&self) -> usize {
        self.sig_store.len()
    }
}
