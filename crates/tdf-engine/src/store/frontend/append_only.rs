//! AppendOnlyStore: insertion order is preserved and meaningful.

use std::borrow::Cow;

use crate::backend::{Backend, BackendAccess, BackendPointer, CacheHints, StoreItemCell};
use crate::impls::binary::error::TdfBinaryError;
use crate::store::frontend::Frontend;
use crate::store::traits::StoreTypes;

/// Enforces append-only insertion order.
#[derive(Debug)]
pub struct AppendOnlyFrontend<S: StoreTypes, B: Backend> {
    _s: std::marker::PhantomData<S::Unique>,
    _b: std::marker::PhantomData<B>,
}

impl<S: StoreTypes, B: Backend> AppendOnlyFrontend<S, B> {
    pub fn new(_offset: usize) -> Self {
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
        backend: &'a mut B,
        cache_hints: CacheHints,
    ) -> Result<Vec<Cow<'a, StoreItemCell<S, B::Types>>>, TdfBinaryError> {
        backend.get_cells(pointer, cache_hints)
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
