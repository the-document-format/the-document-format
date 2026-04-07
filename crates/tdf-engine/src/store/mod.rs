use crate::{
    backend::Backend,
    primitives::{data::DataTypes, item::ItemTypes, page::PageTypes, signature::SignatureTypes},
    store::frontend::{append_only::AppendOnlyFrontend, optimized::OptimizedFrontend},
};

pub mod frontend;
pub mod traits;

// Concrete store type aliases used for VecBackend's internal storage.
pub type PageStore<B: Backend> = AppendOnlyFrontend<PageTypes<B::Types>, B>;
pub type ItemStore<B: Backend> = OptimizedFrontend<ItemTypes<B::Types>, B>;
pub type DataStore<B: Backend> = OptimizedFrontend<DataTypes, B>;
pub type SignatureStore<B: Backend> = AppendOnlyFrontend<SignatureTypes, B>;
