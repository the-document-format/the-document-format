//! AppendOnlyStore: insertion order is preserved and meaningful.

use crate::backend::{Backend, BackendAccess, BackendPointer, StoreItemCell};
use crate::store::frontend::Frontend;
use crate::store::traits::StoreTypes;

/// Enforces append-only insertion order.
#[derive(Debug)]
pub struct AppendOnlyFrontend<S: StoreTypes, B: Backend> {
    _s: std::marker::PhantomData<S::Unique>,
    _b: std::marker::PhantomData<B>,
}

impl<S: StoreTypes, B: Backend> AppendOnlyFrontend<S, B> {
    pub fn new(offset: usize) -> Self {
        AppendOnlyFrontend {
            _s: std::marker::PhantomData,
            _b: std::marker::PhantomData,
        }
    }
}

impl<S: StoreTypes, B: Backend> Default for AppendOnlyFrontend<S, B> {
    fn default() -> Self {
        Self::new(0)
    }
}

impl<S: StoreTypes, B: Backend + BackendAccess<S, B>> Frontend<B> for AppendOnlyFrontend<S, B> {
    type Types = S;

    fn push(
        &mut self,
        item: S::Primitive,
        unique: S::Unique,
        backend: &mut B,
    ) -> BackendPointer<S, B::Types> {
        backend.push_cell(item, unique)
    }

    fn get<'a>(
        &self,
        pointer: &BackendPointer<S, B::Types>,
        backend: &'a B,
    ) -> Option<&'a StoreItemCell<S, B::Types>> {
        backend
            .get_cells(pointer)
            .and_then(|cells| cells.into_iter().next())
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
