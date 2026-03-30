//! Core store trait definitions.

use crate::backend::{Backend, BackendPointer, StoreItemCell, UniqueReduce};
use serde::{Deserialize, Serialize};
use std::hash::Hash;
use std::marker::PhantomData;

pub trait PrimitiveType: Hash + Clone + Eq + PartialEq + Serialize + for<'de> Deserialize<'de> {}

pub trait UniqueType: Hash + Clone + Eq + PartialEq + Serialize + for<'de> Deserialize<'de> + UniqueReduce + Default {}

/// The store trait: generic over a primitive type `P`, unique type `U`, and backend `B`.
pub trait Store<P: PrimitiveType, U: UniqueType, B: Backend> {
    fn push(&mut self, item: P, backend: &mut B) -> BackendPointer<P, U>;
    fn get<'a>(&self, pointer: &BackendPointer<P, U>, backend: &'a B)
        -> Option<&'a StoreItemCell<P, U>>;
    fn size(&self, backend: &B) -> usize;
    fn group(&mut self, items: Vec<BackendPointer<P, U>>, backend: &mut B)
        -> BackendPointer<P, U>;
    fn iter<'a>(&self, backend: &'a B)
        -> Box<dyn Iterator<Item = &'a StoreItemCell<P, U>> + 'a>;
}

/// Higher-level utilities blanket-implemented for all `Store<P, U, B>`.
pub trait StoreExt<P: PrimitiveType, U: UniqueType, B: Backend>: Store<P, U, B> {
    fn iter_rec(&self, pointer: &BackendPointer<P, U>, backend: &B) -> Vec<(P, U)>;
    fn iter_range_rec(&self, pointer: &BackendPointer<P, U>, backend: &B) -> Vec<(P, U)>;
    fn checksum(&self, backend: &B) -> crate::misc::Hash;
}

impl<P: PrimitiveType, U: UniqueType, B: Backend, S: Store<P, U, B>> StoreExt<P, U, B> for S {
    fn iter_rec(&self, pointer: &BackendPointer<P, U>, backend: &B) -> Vec<(P, U)> {
        match pointer {
            BackendPointer::Pointer { unique: outer_unique, .. } => {
                match self.get(pointer, backend) {
                    Some(StoreItemCell::StorePrimitive(p)) => vec![(p.clone(), outer_unique.clone())],
                    Some(StoreItemCell::BackendPointer(inner)) => {
                        let reduced = match inner.clone() {
                            BackendPointer::Pointer { index, unique: inner_u, _phantom } =>
                                BackendPointer::Pointer {
                                    index,
                                    unique: outer_unique.clone().reduce(inner_u),
                                    _phantom,
                                },
                            BackendPointer::PointerRange { start, len, unique: inner_u, _phantom } =>
                                BackendPointer::PointerRange {
                                    start, len,
                                    unique: outer_unique.clone().reduce(inner_u),
                                    _phantom,
                                },
                        };
                        self.iter_rec(&reduced, backend)
                    }
                    None => vec![],
                }
            }
            BackendPointer::PointerRange { start, len, unique, .. } => {
                (*start..*start + *len)
                    .flat_map(|i| {
                        let ptr = BackendPointer::Pointer {
                            index: i,
                            unique: unique.clone(),
                            _phantom: PhantomData,
                        };
                        self.iter_rec(&ptr, backend)
                    })
                    .collect()
            }
        }
    }
    fn iter_range_rec(&self, pointer: &BackendPointer<P, U>, backend: &B) -> Vec<(P, U)> { todo!() }
    fn checksum(&self, backend: &B) -> crate::misc::Hash { todo!() }
}
