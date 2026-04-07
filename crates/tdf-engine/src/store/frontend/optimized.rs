//! OptimizedStore: interns identical primitives so they share a single copy in storage.

use crate::backend::{Backend, BackendAccess, BackendPointer, StoreItemCell};
use crate::store::traits::{Store, StoreTypes};
use std::collections::HashMap;

/// Deduplicates identical primitives.
#[derive(Debug)]
pub struct OptimizedFrontend<S: StoreTypes, B: Backend> {
    dedup: HashMap<u64, usize>,
    _s: std::marker::PhantomData<S>,
    _b: std::marker::PhantomData<B>,
}

impl<S: StoreTypes, B: Backend> OptimizedFrontend<S, B> {
    pub fn new(offset: usize) -> Self {
        Self {
            dedup: HashMap::new(),
            _s: std::marker::PhantomData,
            _b: std::marker::PhantomData,
        }
    }
}

impl<S: StoreTypes, B: Backend> Default for OptimizedFrontend<S, B> {
    fn default() -> Self {
        Self::new(0)
    }
}

impl<S: StoreTypes, B: Backend + BackendAccess<S, B>> Store<B> for OptimizedFrontend<S, B> {
    type Types = S;
    fn push(&mut self, item: S::Primitive, backend: &mut B) -> BackendPointer<S, B::Types> {
        backend.push_cell(StoreItemCell::StorePrimitive(item))
    }
    fn get<'a>(
        &self,
        pointer: &BackendPointer<S, B::Types>,
        backend: &'a B,
    ) -> Option<&'a StoreItemCell<S, B::Types>> {
        backend.get_cell(pointer)
    }
    fn size(&self, _backend: &B) -> usize {
        todo!()
    }
    fn iter<'a>(
        &self,
        _backend: &'a B,
    ) -> Box<dyn Iterator<Item = &'a StoreItemCell<S, B::Types>> + 'a>
    where
        B: BackendAccess<S, B>,
    {
        todo!()
    }
}
