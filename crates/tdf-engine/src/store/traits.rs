//! Core store trait definitions.

use crate::backend::UniqueReduce;
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

pub type StoreItems<S: StoreTypes> = Vec<(<S as StoreTypes>::Primitive, <S as StoreTypes>::Unique)>;
