//! AppendOnlyStore: insertion order is preserved and meaningful.

use crate::backend::{Backend, BackendAccess, BackendPointer, BackendView, StoreItemCell};
use crate::store::traits::{Store, StoreTypes};

/// Enforces append-only insertion order.
#[derive(Debug)]
pub struct AppendOnlyFrontend<S: StoreTypes, B: Backend> {
    pub view: BackendView<S::Primitive, B>,
    _phantom: std::marker::PhantomData<S::Unique>,
}

impl<S: StoreTypes, B: Backend> AppendOnlyFrontend<S, B> {
    pub fn new(offset: usize) -> Self {
        AppendOnlyFrontend {
            view: BackendView::new(offset),
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<S: StoreTypes, B: Backend> Default for AppendOnlyFrontend<S, B> {
    fn default() -> Self {
        Self::new(0)
    }
}

impl<S: StoreTypes, B: Backend> StoreTypes for AppendOnlyFrontend<S, B> {
    type Primitive = S::Primitive;
    type Unique = S::Unique;
}

impl<S: StoreTypes, B: Backend + BackendAccess<Self, B>> Store<B> for AppendOnlyFrontend<S, B> {
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
