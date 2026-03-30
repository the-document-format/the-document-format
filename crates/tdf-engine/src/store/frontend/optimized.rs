//! OptimizedStore: interns identical primitives so they share a single copy in storage.

use crate::backend::{Backend, BackendPointer, BackendView, StoreItemCell};
use crate::store::traits::{PrimitiveType, Store, UniqueType};
use std::collections::HashMap;

/// Deduplicates identical primitives.
#[derive(Debug)]
pub struct OptimizedStore<P, U, B: Backend> {
    pub view: BackendView<P, B>,
    dedup: HashMap<u64, usize>,
    _phantom: std::marker::PhantomData<U>,
}

impl<P, U, B: Backend> OptimizedStore<P, U, B> {
    pub fn new(offset: usize) -> Self {
        OptimizedStore { view: BackendView::new(offset), dedup: HashMap::new(), _phantom: std::marker::PhantomData }
    }
}

impl<P, U, B: Backend> Default for OptimizedStore<P, U, B> {
    fn default() -> Self { Self::new(0) }
}

impl<P: PrimitiveType, U: UniqueType, B: Backend> Store<P, U, B> for OptimizedStore<P, U, B> {
    fn push(&mut self, item: P, backend: &mut B) -> BackendPointer<P, U> { todo!() }
    fn get<'a>(&self, pointer: &BackendPointer<P, U>, backend: &'a B)
        -> Option<&'a StoreItemCell<P, U>> { todo!() }
    fn size(&self, backend: &B) -> usize { todo!() }
    fn group(&mut self, items: Vec<BackendPointer<P, U>>, backend: &mut B)
        -> BackendPointer<P, U> { todo!() }
    fn iter<'a>(&self, backend: &'a B)
        -> Box<dyn Iterator<Item = &'a StoreItemCell<P, U>> + 'a> { todo!() }
}
