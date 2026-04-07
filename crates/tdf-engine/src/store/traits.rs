//! Core store trait definitions.

use crate::backend::{Backend, BackendAccess, BackendPointer, StoreItemCell, UniqueReduce};
use serde::{Deserialize, Serialize};
use std::{fmt::Debug, hash::Hash};

pub trait PrimitiveType:
    Hash + std::fmt::Debug + Clone + Eq + PartialEq + Serialize + for<'de> Deserialize<'de>
{
}

pub trait UniqueType:
    Hash
    + std::fmt::Debug
    + Clone
    + Eq
    + PartialEq
    + Serialize
    + for<'de> Deserialize<'de>
    + UniqueReduce
    + Default
{
}

/// Lightweight descriptor: just the Primitive and Unique associated types.
/// Used as the bound on `BackendPointer<S, B>` and `BackendAccess<S, B>` to
/// avoid the cyclic bound that arises from using `Store<B>` there directly.
pub trait StoreTypes:
    Sized + Serialize + for<'de> Deserialize<'de> + Debug + Clone + PartialEq + Eq + Hash
{
    type Primitive: PrimitiveType;
    type Unique: UniqueType;
}

/// The store trait: generic over a backend `B`, with `Primitive` and `Unique` via `StoreTypes`.
pub trait Store<B: Backend> {
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
pub trait StoreExt<B: Backend>: Store<B> {
    fn iter_rec(
        &self,
        pointer: &BackendPointer<Self::Types, B::Types>,
        backend: &B,
    ) -> Vec<(
        <Self::Types as StoreTypes>::Primitive,
        <Self::Types as StoreTypes>::Unique,
    )>
    where
        B: BackendAccess<Self::Types, B>;
    fn iter_range_rec(
        &self,
        pointer: &BackendPointer<Self::Types, B::Types>,
        backend: &B,
    ) -> Vec<(
        <Self::Types as StoreTypes>::Primitive,
        <Self::Types as StoreTypes>::Unique,
    )>
    where
        B: BackendAccess<Self::Types, B>;
    fn checksum(&self, backend: &B) -> crate::misc::Hash;
}

// impl<B: Backend, S: Store<B>> StoreExt<B> for S {
//     fn iter_rec(
//         &self,
//         pointer: &BackendPointer<Self, B>,
//         backend: &B,
//     ) -> Vec<(S::Primitive, S::Unique)>
//     where
//         B: BackendAccess<Self, B>,
//     {
//         match pointer {
//             BackendPointer::Single { unique: outer_unique, .. } => {
//                 match self.get(pointer, backend) {
//                     Some(StoreItemCell::StorePrimitive(p)) => {
//                         vec![(p.clone(), outer_unique.clone())]
//                     }
//                     Some(StoreItemCell::BackendPointer(inner)) => {
//                         let reduced = match inner.clone() {
//                             BackendPointer::Single { index, unique: inner_u, _phantom } => {
//                                 BackendPointer::Single {
//                                     index,
//                                     unique: outer_unique.clone().reduce(inner_u),
//                                     _phantom,
//                                 }
//                             }
//                             BackendPointer::Group { range, uniques: inner_us, _phantom } => {
//                                 BackendPointer::Group {
//                                     range,
//                                     uniques: inner_us
//                                         .into_iter()
//                                         .map(|u| outer_unique.clone().reduce(u))
//                                         .collect(),
//                                     _phantom,
//                                 }
//                             }
//                         };
//                         self.iter_rec(&reduced, backend)
//                     }
//                     None => vec![],
//                 }
//             }
//             BackendPointer::Group { range, uniques, .. } => backend
//                 .expand_group(range, uniques.clone())
//                 .iter()
//                 .flat_map(|ptr| self.iter_rec(ptr, backend))
//                 .collect(),
//         }
//     }
//     fn iter_range_rec(
//         &self,
//         _pointer: &BackendPointer<Self, B>,
//         _backend: &B,
//     ) -> Vec<(S::Primitive, S::Unique)>
//     where
//         B: BackendAccess<Self, B>,
//     {
//         todo!()
//     }
//     fn checksum(&self, _backend: &B) -> crate::misc::Hash {
//         todo!()
//     }
// }
