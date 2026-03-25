# The `Store`s

A store is responsible for one kind of data in a TDF document. It sits between the reader and the backend, so the reader does not have to worry about how or where data is actually stored -- it just asks the store for what it needs and gets back the right type.

# Operations

Every store has the following operations:

- Get item, which returns a store cell in a store given a pointer
- Insert an item into the store, which returns a handle, which is a pointer pointer
- Size, which is the number of items in the store
- Grouping many items in the store together into a single store item group cell

We also define store extension, which is a collection of utilities generic to all stores:

- Deference a store cell -- this calls get on the store with the inside of the cell until we have a store primitive
- It offers a function to get a store range iterator that iterates over a range of items in the store
- Checksum the entire store. Checksumming under the hood delegates to the backend to compute a rolling checksum of every individual store item

Then, different stores will have their own unique capabilities...

# The specific stores

There are three stores in the TDF specification,

- The item store
- The data store
- The signature store

Tags are optional data that live inside EVERY primitive. Each store has its own kind tag.

Container is a reference to an item in the data store. it is a simple BackendPointer<DataPrimitive, DataUnique>.

## The item store

Stores all items that are visible on pages.

The ItemUnique has the following fields:

- Tags -- see below. This is a collection of optional stuff that might apply to a specific Item or item that lives lower within the item.
- Location

The ItemStorePrimitive has the following tags available:

- Font
- Size
- Stroke width
- Stroke color
- Fill color
- Opacity
- Text align

Primitives include:

- Text box:
  - Text data, which is the text itself
- Image:
  - A container to the data store
  - Size data
- Vector:
  - List of Bezier points
  - Line width
  - Fill and stroke color
- Shape:
  - A special enum of common shapes like circles, squares, TDF logo
  - Most of the other parameters are similar to vector
  
The capabilities of an item store include:

- Grouping together multiple items into a "group" -- a concept defined by the backend.

## The data store

Stores blobular data. This is large data that we do not want to automatically load as we lazy-iterate the items on some page. Primitives include:

- Font data:
  - The font data itself
- Image data
  - The raw image data

## The signature store

An append-only store. This is where signature data lives. Signatures sign previous signatures which is why it must be append only.

The only primitive for the signature store is the "Signature" which contains:

- Cryptographic public key and signing of the entire document including our own public key in the hash.
- The time, which we will find a way to properly verify

# Structure

A `Store` is a thin wrapper on top of a backend that manages interaction with a backend scoped to some set of identically typed items of potentially variable sizes, referred to as "primitives," generic on that type of item, `T`, which we bound with some type `PrimitiveType`. Usually this is an enum when you are interacting with the `Store`, but on disc something that is automatically compacted.

We also define various utilities that you can use on `Store`s that are automatically implemented for stores or can easily be constructed. For example, for recursive dereferencing of items in stores, you can use `StoreExt`, which includes utilities for operations like that. This is defined generically for all types of stores.

```rust
trait BackendPointerType = Ord;

pub enum BackendItemCell<
    PrimitiveType,
    UniqueType,
    BackendPointer<PrimitiveType, UniqueType>: BackendPointerType,
    BackendPointerGroup<PrimitiveType, UniqueType>: BackendPointerType,
> {
    BackendItemRef(
        BackendItemRef<
            PrimitiveType,
            UniqueType,
            BackendPointer<PrimitiveType, UniqueType>,
            BackendPointerGroup<PrimitiveType, UniqueType>,
        >,
    ),
    BackendPrimitive(PrimitiveType),
}

// The actual pointers themselves are containers that are generic and specific to the `Backend` (and we have to propagate these generics through the `Backend`). More on this in our section on `Backend`s.
pub enum BackendItemRef<
    PrimitiveType,
    UniqueType,
    BackendPointer<PrimitiveType, UniqueType>: BackendPointerType,
    BackendPointerGroup<PrimitiveType, UniqueType>: BackendPointerType,
> {
    Pointer(BackendPointer<PrimitiveType, UniqueType>),
    PointerGroup(BackendPointerGroup<PrimitiveType, UniqueType>),
}
```
