use crate::backend::{Backend, BackendPointer};
use crate::primitives::item::ItemTypes;
use crate::store::frontend::append_only::AppendOnlyFrontend;
use crate::store::frontend::optimized::OptimizedFrontend;
use crate::store::traits::StoreTypes;

/// A pointer from the page store into the item store.
pub type ItemPointer<B> = BackendPointer<OptimizedFrontend<ItemTypes, B>, B>;

/// A pointer into the page store itself.
pub type PageStorePointer<B> = BackendPointer<AppendOnlyFrontend<PageTypes<B>, B>, B>;

impl<B: Backend> crate::store::traits::PrimitiveType for ItemPointer<B> {}

pub struct PageTypes<B: Backend>(std::marker::PhantomData<B>);

impl<B: Backend> StoreTypes for PageTypes<B> {
    type Primitive = ItemPointer<B>;
    type Unique = ();
}
