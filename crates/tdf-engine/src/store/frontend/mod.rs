use crate::{
    backend::{Backend, BackendAccess, BackendPointer, StoreItemCell},
    store::traits::{StoreItems, StoreTypes},
};

pub mod append_only;
pub mod optimized;

/// The store trait: generic over a backend `B`, with `Primitive` and `Unique` via `StoreTypes`.
pub trait Frontend<B: Backend> {
    type Types: StoreTypes;
    fn push(
        &mut self,
        item: <Self::Types as StoreTypes>::Primitive,
        unique: <Self::Types as StoreTypes>::Unique,
        backend: &mut B,
    ) -> BackendPointer<Self::Types, B::Types>
    where
        B: BackendAccess<Self::Types, B>;

    fn get<'a>(
        &self,
        pointer: &BackendPointer<Self::Types, B::Types>,
        backend: &'a B,
    ) -> Option<&'a StoreItemCell<Self::Types, B::Types>>
    where
        B: BackendAccess<Self::Types, B>;

    fn size(&self, backend: &B) -> usize;

    #[allow(clippy::type_complexity)]
    fn iter<'a>(
        &self,
        backend: &'a B,
    ) -> Box<dyn Iterator<Item = &'a StoreItemCell<Self::Types, B::Types>> + 'a>
    where
        B: BackendAccess<Self::Types, B>;
}

/// Higher-level utilities blanket-implemented for all `Store<B>`.
pub trait FrontendExt<B: Backend>: Frontend<B> {
    fn iter_rec(
        &self,
        pointer: &BackendPointer<Self::Types, B::Types>,
        backend: &B,
    ) -> StoreItems<Self::Types>
    where
        B: BackendAccess<Self::Types, B>;
    fn iter_range_rec(
        &self,
        pointer: &BackendPointer<Self::Types, B::Types>,
        backend: &B,
    ) -> StoreItems<Self::Types>
    where
        B: BackendAccess<Self::Types, B>;
    fn checksum(&self, backend: &B) -> crate::misc::Hash;
}
