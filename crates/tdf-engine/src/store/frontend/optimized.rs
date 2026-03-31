//! OptimizedStore: interns identical primitives so they share a single copy in storage.

use crate::backend::{Backend, BackendAccess, BackendPointer, BackendView, StoreItemCell};
use crate::store::traits::{Store, StoreTypes};
use std::collections::HashMap;

/// Deduplicates identical primitives.
#[derive(Debug)]
pub struct OptimizedFrontend<S: StoreTypes, B: Backend> {
    pub view: BackendView<S, B>,
    dedup: HashMap<u64, usize>,
    _phantom: std::marker::PhantomData<S>,
}

impl<S: StoreTypes, B: Backend> OptimizedFrontend<S, B> {
    pub fn new(offset: usize) -> Self {
        OptimizedFrontend {
            view: BackendView::new(offset),
            dedup: HashMap::new(),
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<S: StoreTypes, B: Backend> Default for OptimizedFrontend<S, B> {
    fn default() -> Self {
        Self::new(0)
    }
}

impl<S: StoreTypes, B: Backend> StoreTypes for OptimizedFrontend<S, B> {
    type Primitive = S::Primitive;
    type Unique = S::Unique;
}

impl<S: StoreTypes, B: Backend + BackendAccess<Self, B>> Store<B> for OptimizedFrontend<S, B> {
    fn push(&mut self, item: S::Primitive, backend: &mut B) -> BackendPointer<Self, B> {
        backend.push_cell(StoreItemCell::StorePrimitive(item))
    }
    fn get<'a>(
        &self,
        pointer: &BackendPointer<Self, B>,
        backend: &'a B,
    ) -> Option<&'a StoreItemCell<Self, B>> {
        backend.get_cell(pointer)
    }
    fn size(&self, _backend: &B) -> usize {
        todo!()
    }
    fn iter<'a>(&self, _backend: &'a B) -> Box<dyn Iterator<Item = &'a StoreItemCell<Self, B>> + 'a>
    where
        B: BackendAccess<Self, B>,
    {
        todo!()
    }
}
