//! Core store trait definitions.

use crate::backend::{Backend, BackendAccess, BackendPointer, StoreItemCell, UniqueReduce};
use serde::{Deserialize, Serialize};
use std::hash::Hash;

pub trait PrimitiveType:
    Hash + Clone + Eq + PartialEq + Serialize + for<'de> Deserialize<'de>
{
}

pub trait UniqueType:
    Hash + Clone + Eq + PartialEq + Serialize + for<'de> Deserialize<'de> + UniqueReduce + Default
{
}

/// The store trait: generic over a primitive type `P`, unique type `U`, and backend `B`.
pub trait Store<P: PrimitiveType, U: UniqueType, B: Backend + BackendAccess<P, U>> {
    fn push(
        &mut self,
        item: P,
        backend: &mut B,
    ) -> BackendPointer<P, U, <B as BackendAccess<P, U>>::Group>;
    fn get<'a>(
        &self,
        pointer: &BackendPointer<P, U, <B as BackendAccess<P, U>>::Group>,
        backend: &'a B,
    ) -> Option<&'a StoreItemCell<P, U, <B as BackendAccess<P, U>>::Group>>;
    fn size(&self, backend: &B) -> usize;
    #[allow(clippy::type_complexity)]
    fn iter<'a>(
        &self,
        backend: &'a B,
    ) -> Box<dyn Iterator<Item = &'a StoreItemCell<P, U, <B as BackendAccess<P, U>>::Group>> + 'a>;
}

/// Higher-level utilities blanket-implemented for all `Store<P, U, B>`.
pub trait StoreExt<P: PrimitiveType, U: UniqueType, B: Backend + BackendAccess<P, U>>:
    Store<P, U, B>
{
    fn iter_rec(
        &self,
        pointer: &BackendPointer<P, U, <B as BackendAccess<P, U>>::Group>,
        backend: &B,
    ) -> Vec<(P, U)>;
    fn iter_range_rec(
        &self,
        pointer: &BackendPointer<P, U, <B as BackendAccess<P, U>>::Group>,
        backend: &B,
    ) -> Vec<(P, U)>;
    fn checksum(&self, backend: &B) -> crate::misc::Hash;
}

impl<P: PrimitiveType, U: UniqueType, B: Backend + BackendAccess<P, U>, S: Store<P, U, B>>
    StoreExt<P, U, B> for S
where
    <B as BackendAccess<P, U>>::Group: Clone,
{
    fn iter_rec(
        &self,
        pointer: &BackendPointer<P, U, <B as BackendAccess<P, U>>::Group>,
        backend: &B,
    ) -> Vec<(P, U)> {
        match pointer {
            BackendPointer::Single {
                unique: outer_unique,
                ..
            } => match self.get(pointer, backend) {
                Some(StoreItemCell::StorePrimitive(p)) => vec![(p.clone(), outer_unique.clone())],
                Some(StoreItemCell::BackendPointer(inner)) => {
                    let reduced = match inner.clone() {
                        BackendPointer::Single {
                            index,
                            unique: inner_u,
                            _phantom,
                        } => BackendPointer::Single {
                            index,
                            unique: outer_unique.clone().reduce(inner_u),
                            _phantom,
                        },
                        BackendPointer::Group {
                            group,
                            unique: inner_u,
                            _phantom,
                        } => BackendPointer::Group {
                            group,
                            unique: outer_unique.clone().reduce(inner_u),
                            _phantom,
                        },
                    };
                    self.iter_rec(&reduced, backend)
                }
                None => vec![],
            },
            BackendPointer::Group { group, unique, .. } => backend
                .expand_group(group, unique.clone())
                .iter()
                .flat_map(|ptr| self.iter_rec(ptr, backend))
                .collect(),
        }
    }
    fn iter_range_rec(
        &self,
        pointer: &BackendPointer<P, U, <B as BackendAccess<P, U>>::Group>,
        backend: &B,
    ) -> Vec<(P, U)> {
        todo!()
    }
    fn checksum(&self, backend: &B) -> crate::misc::Hash {
        todo!()
    }
}
