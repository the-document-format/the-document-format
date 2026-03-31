use crate::{misc::Instant, store::traits::StoreTypes};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct SignaturePrimitive {
    pub public_key: Vec<u8>,
    pub document_hash: Vec<u8>,
    pub timestamp: Instant,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct SignatureUnique;

impl crate::store::traits::PrimitiveType for SignaturePrimitive {}
impl crate::store::traits::UniqueType for SignatureUnique {}
impl crate::backend::UniqueReduce for SignatureUnique {
    fn reduce(self, _other: Self) -> Self {
        Self
    }
}

pub struct SignatureTypes;

impl StoreTypes for SignatureTypes {
    type Primitive = SignaturePrimitive;
    type Unique = SignatureUnique;
}
