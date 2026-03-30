//! AppendOnlyStore: insertion order is preserved and meaningful.

use crate::backend::{Backend, BackendAccess, BackendPointer, BackendView, StoreItemCell};
use crate::store::traits::{PrimitiveType, Store, UniqueType};

/// Enforces append-only insertion order.
#[derive(Debug)]
pub struct AppendOnlyStore<P, U, B: Backend> {
    pub view: BackendView<P, B>,
    _phantom: std::marker::PhantomData<U>,
}

impl<P, U, B: Backend> AppendOnlyStore<P, U, B> {
    pub fn new(offset: usize) -> Self {
        AppendOnlyStore {
            view: BackendView::new(offset),
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<P, U, B: Backend> Default for AppendOnlyStore<P, U, B> {
    fn default() -> Self {
        Self::new(0)
    }
}

impl<P: PrimitiveType, U: UniqueType, B: Backend + BackendAccess<P, U>> Store<P, U, B>
    for AppendOnlyStore<P, U, B>
{
    fn push(&mut self, item: P, backend: &mut B) -> BackendPointer<P, U> {
        backend.push_cell(StoreItemCell::StorePrimitive(item))
    }
    fn get<'a>(
        &self,
        pointer: &BackendPointer<P, U>,
        backend: &'a B,
    ) -> Option<&'a StoreItemCell<P, U>> {
        backend.get_cell(pointer)
    }
    fn size(&self, backend: &B) -> usize {
        todo!()
    }
    fn group(&mut self, items: Vec<BackendPointer<P, U>>, backend: &mut B) -> BackendPointer<P, U> {
        todo!()
    }
    fn iter<'a>(&self, backend: &'a B) -> Box<dyn Iterator<Item = &'a StoreItemCell<P, U>> + 'a> {
        todo!()
    }
}
