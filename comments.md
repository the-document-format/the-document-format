Here are many thoughts. I would like to go back and forth together and have you update our docs to be nice and through and non ambiguous and document the way that TDF works.

also have a PRIMITIVES.md and move the primitive defs from STORE.md

The general flow for when you want to read a TDF file:

1. Load the file via the reader. This reads the whole header, because the header segment is fixed-size. Now you know the offset to the other segments and are able to start accessing pages in the TDF. You now know that you want to read page 40.
2. Now you call reader.iter_page_items(40) and it goes to the page store, which is a store of <Primitive=ItemPointer, Unique=()>, which gets you the ItemPointer. We have to have a unique store for the pages because the pointers themselves that reference the top level items in the ItemStore are themselves variable sized (pointers are variable sized except in unique cases like when Unique=()).
  1. The ItemPointer is a BackendPointer<ItemPrimitive, ItemUnique>, which in this case happens to be a PointerRange of all of the items on that page. It may recurse and have pointers in that range that point to other items in the item store. To be clear, the pages store stores item pointers, the item store stores the actual items or other item pointers to other slots in that item store, including primitive types that may refer to items in the data store.
  2. Now we call backend.iter_page_children_rec(pointer) on that pointer, which returns an iter of all of the underlying items that that range refers to. We have similar methods like backend.rec_iter_data_children() for iterating over all data items for a DataStorePointer. This recursively dereferences using the non rec version and gets us an iter of actual Item primitives.
  3. Now the renderer can draw these primitives onto the page. Some of the primitives are Handles that refer to items in the data store. These items could include large blobs like images. The renderer will lazily load them by asking the reader to deref a handle via .deref_handle<T>(Handles<T>), which gives you a DataStoreItem::T.
  
iter_page_children_rec(pointer: BackendPointer<ItemPrimitive, ItemUnique>): it returns a new struct, which is an ItemPrimitiveAndUniqueIterator. It recurses all the way down to get them, each unique that applies to each one is a "sum" of all the uniques it took to get there.

# BackendPointers

BackendPointers have two components: a pointer to a store item, and some non internable concrete data, the "unique." Along your journey to the final primitive that lives in a store item cell that the pointer may take you to, you will follow through possibly many BackendPointers, which will accumulate many uniques, and we need to be able to reduce all of these unique into a single unique. The trait bounds to make this possible are that BackendUnique impls BackendUniqueData which has a reduce method that takes another unique and reduces the two. In our impl it will add together the positions (e.g. (0, 1) and (2, 2) becomes (2, 3), and will merge the two tag sets together).

---

Additional notes on stores:

Stores:
- Append only stores: you can only append things, so you get 
- Optimized stores

Additional notes on backends:

BackendPointer must be ordered 

The non internable data exists inside the definition of pointer itself.

A backend is concrete and stores three stores:

item_store: OptimizedStore<ItemPrimitive, ItemUnique>
item_store_offset:
data_store: OptimizedStore<DataPrimitive, DataUnique>
data_store_offset:
signature_store: OptimizedStore<SignaturePrimitive, SignatureUnique>
signature_store_offset:

Backends are where the actual data lives. It stores where all the stores live, and every item in every store is referred to by a BackendPointer. This BackendPointer *is* a relative offset. Combining that relative offset with the offset of the store that item is in, and knowing the type of items that live in that store, we are able to implement functions that can get you an item in a specific store, where all stores live in the same backend, returning the specific dereferenced type of the backend pointer you asked the backend for.

For our simple implementation of a backend that uses a byte stream, these three functions will be along the lines of iter_item_ptr(pointer: BackendPointer<ItemPrimitive, ItemUnique>), iter_data_ptr(pointer: BackendPointer<DataPrimitive, DataUnique>), iter_signature_store(pointer: BackendPointer<SignaturePrimitive, SignatureUnique>), which return an iterator of the item cells in that store. An item cell is a primitive or a pointer to another item in that same store. These functions internally know where the store items live, and because these functions are defined specifically for one of the stores, the amount of bytes we need to read to access concrete types is straightforward.

All backends have the following operations:

BackendPointer 

- get(backend_pointer<T, U>) -> T
- set(backend_pointer)

We can then generically store three stores of three concreteish store types.

A backend has methods to dereference a item given a BackendPointer<ItemPrimitive, ItemUnique> via .deref_item(), dereference a item given BackendPointer<DataPrimitive, DataUnique>, and dereference a item given SignaturePointer<SignaturePrimitive, SignatureUnique>.

Concrete pointer implementations are enums generic on <T, U>:

Pointer or PointerRange:

Under the hood, a backend pointer may choose how to implement the meaning of "PointerRange" -- for example, IPFS might implement it as a vec of hashes, whereas a vector store will implement it as a index offset and length. It also chooses the meaning of Pointer.


When you use a BackendPointer via any of the backend's functions  you end up using a backend pointer to retrieve a iterator of items even if you are getting a single item, and in that way it is nice and generic.


and for example for the vec backend implementation a pointer is an index and the U data. when you are looking at the page you see a single pointer which points to a pointer group and has position 0,0 always (since the page group of items which is a pointer group) always has everything relative to the top left. the item gr

TODO: wolf thinks this should be concrete

# Example flow

1. Iterate the items on page 4:
1. Then the reader gets a pointer for the page
2. The pointer is (probably) a pointer range
3. We call get on the backend
