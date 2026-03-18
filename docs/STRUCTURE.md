# The `Reader`

The `Reader` is the highest level interface that users use to read a TDF document. It provides various methods, like querying for an iterable of all the items on some specific page. Under the hood, it delegates to a respective `Store` (for example, if I am asking for an image, it will query the `DataStore`), and the stores know how to talk to some backend to actually retrieve the data. The querying of the backend is done by the `Store`, not the `Reader`.

# The `Store`s

A `Store` is a thin wrapper on top of a backend that manages interaction with a backend scoped to some set of identically typed items of potentially variable sizes, referred to as "primitives," generic on that type of item, `T`, which we bound with some type `PrimitiveType`. Usually this is an enum when you are interacting with the `Store`, but on disc something that is automatically compacted .

We also define various utilities that you can use on `Store`s that are automatically implemented for stores or can easily be constructed. For example, for recursive dereferencing of items in stores, you can use `StoreExt`, which includes utilities for operations like that. This is defined generically for all types of stores.

```rust
trait BackendPointerType = Ord;

pub enum StoreItemCell<
    PrimitiveType,
    UniqueType,
    BackendPointer<PrimitiveType, UniqueType>: BackendPointerType,
    BackendPointerGroup<PrimitiveType, UniqueType>: BackendPointerType,
> {
    StoreItemRef(
        StoreItemRef<
            PrimitiveType,
            UniqueType,
            BackendPointer<PrimitiveType, UniqueType>,
            BackendPointerGroup<PrimitiveType, UniqueType>,
        >,
    ),
    StorePrimitive(PrimitiveType),
}

// The actual pointers themselves are containers that are generic and specific to the `Backend` (and we have to propagate these generics through the `Store`). More on this in our section on `Backend`s.
pub enum StoreItemRef<
    PrimitiveType,
    UniqueType,
    BackendPointer<PrimitiveType, UniqueType>: BackendPointerType,
    BackendPointerGroup<PrimitiveType, UniqueType>: BackendPointerType,
> {
    Pointer(BackendPointer<PrimitiveType, UniqueType>),
    PointerGroup(BackendPointerGroup<PrimitiveType, UniqueType>),
}
```

## `Frontend`s

A store is a trait. A `Frontend` is a implementation of a `Store`, but still specific to some type of data (also usually some `enum`) for type `T`. The purpose of a `Frontend` is to maintain some additional invariants about how the data is stored. We plan on defining the following types of `Frontend`s:

- The `AppendOnlyStore` is a frontend that makes sure that the data being stored is sequential (pointers are ordered). That way when you sign the document, there is a guarantee that the items you are signing all come before the last item.
- The `OptimizedStore` is a frontend that automatically groups items and interns identical items. It is used for the `DataStore` and `ItemStore` where we are storing potentially duplicate data. It takes advantage of an abstract "pointer + group" layout (more on this in `Backends`).

Every store is responsible for implementing the following methods, which act atop an underlying `Backend`:

- `checksum`: This reads through the entire corresponding section of the `Backend` and gives you a unique hash for the entire section.
- `get`: This is a thin wrapper on the `Backend` that queries an item by some pointer all the way into the `Backend`, giving you a nicely typed item. The `backend` provides methods for each of the concrete stores we define, like `ItemStore`, where you can query an item at some index and get it back as the correct type.

# The `Backend`s

A backend is a data structure to store grouped, ordered, key-value data. It has no knowledge of the actual data being stored and there is an underlying assumption that the data is faster to query for some notion of a "group." Backends are generic on a "pointer type" and a "pointer group type," and the backend trait lets you query a concrete backend given one of these two reference types.

An example of a backend is the `VecBackend`:

A `VecBackend` defines its pointer as a usize index, and a pointer group as a usize index and usize length. The `VecBackend` stores all of its data in a single vector, and the pointer group is used to reference contiguous slices of the vector. This is a very simple dumb backend for tests.

A more complicated example is the `IPFSBackend`. This backend defines its pointer as a CID, and its pointer group is a vec of CIDs. The `IPFSBackend` stores all of its data in IPFS, and the pointer group is used to reference a group of data from IPFS. In this case we do get our performance gain from using a group by virtue of the fact that we can fetch all of the data in a group with a single IPFS request.

# Example Flow:

Let's say you are opening up a TDF file on your computer, and you want
