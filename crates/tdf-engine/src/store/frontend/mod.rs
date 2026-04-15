use std::borrow::Cow;

use crate::{
    backend::{Backend, BackendAccess, BackendPointer, CacheHints, StoreItemCell, UniqueReduce},
    impls::binary::error::TdfBinaryError,
    misc::Hash,
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
        backend: &'a mut B,
        cache_hints: CacheHints,
    ) -> Result<Vec<Cow<'a, StoreItemCell<Self::Types, B::Types>>>, TdfBinaryError>
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
        backend: &mut B,
        cache_hints: CacheHints,
    ) -> StoreItems<Self::Types>
    where
        B: BackendAccess<Self::Types, B>;

    fn checksum(&self, backend: &B) -> Hash;
}

impl<B, T> FrontendExt<B> for T
where
    B: Backend,
    T: Frontend<B>,
{
    fn iter_rec(
        &self,
        pointer: &BackendPointer<Self::Types, B::Types>,
        backend: &mut B,
        hints: CacheHints,
    ) -> StoreItems<Self::Types>
    where
        B: BackendAccess<Self::Types, B>,
    {
        let mut stack: Vec<(
            StoreItemCell<Self::Types, B::Types>,
            <Self::Types as StoreTypes>::Unique,
        )> = Vec::new();
        let mut output = Vec::new();

        // Seed the stack from the initial pointer
        let initial_cells =
            <B as BackendAccess<Self::Types, B>>::get_cells(backend, pointer, hints)
                .unwrap_or_default();
        stack.extend(
            initial_cells
                .into_iter()
                .zip(pointer.uniques())
                .rev()
                .map(|(cow, unique)| (cow.into_owned(), unique)),
        );

        while let Some((cell, acc_unique)) = stack.pop() {
            match cell {
                StoreItemCell::StorePrimitive(primitive) => {
                    output.push((primitive, acc_unique));
                }
                StoreItemCell::BackendPointer(inner_ptr) => {
                    let inner_uniques = inner_ptr.uniques();
                    let inner_cells =
                        <B as BackendAccess<Self::Types, B>>::get_cells(backend, &inner_ptr, hints)
                            .unwrap_or_default();

                    stack.extend(
                        inner_cells
                            .into_iter()
                            .zip(inner_uniques)
                            .rev()
                            .map(|(cow, u)| (cow.into_owned(), acc_unique.clone().reduce(u))),
                    );
                }
            }
        }

        output
    }

    fn checksum(&self, backend: &B) -> Hash {
        todo!()
    }
}
