# The Backend

A backend is a **trait** that physically stores all four stores in the same underlying storage medium — a byte array, a vector, IPFS, or anything else. It has no knowledge of document semantics (pages, items, fonts); it only knows how to store and retrieve typed data at pointer-addressed locations.

The [reader](./READER.md) is generic over a concrete backend:

```
TDFReader<B: Backend>
```

Concrete implementations include `VecBackend`, `IPFSBackend`, `ByteArrayBackend`, and `JsonBackend`.

## What a backend manages

Every backend must manage four stores:

```
page_store:      AppendOnlyStore<ItemPointer, ()>
item_store:      OptimizedStore<ItemPrimitive, ItemUnique>
data_store:      OptimizedStore<DataPrimitive, ()>
signature_store: AppendOnlyStore<SignaturePrimitive, SignatureUnique>
```

The backend makes **no guarantees about physical layout** — items are not necessarily stored in a fixed order on disk. The only guarantee is that **iteration order matches insertion order** within each store.

See [Primitives](./PRIMITIVES.md) for the concrete types and [Frontends](./FRONTEND.md) for the invariants each store type enforces.

## BackendView

Stores do not hold a reference to the full backend. Instead, each store holds a `BackendView`:

```
BackendView<Primitive, B: Backend>
```

A `BackendView` is a lightweight struct constructed by the backend when setting up stores — one view per store. It is constructed with the **offset** for that store's region within the backend, and provides type-safe access:

- `view.get(&backend)` — reads from the backend at the correct offset for this store
- `view.set(&mut backend, ...)` — writes to the backend at the correct offset for this store

The view is generic on both the store's primitive type and the concrete backend type. It knows which region of the backend belongs to its store and communicates that to the backend in a type-safe way.

The view exists so that each store can talk to the backend **without knowing about other stores' data**. The page store's view cannot accidentally read from the data store's region, and vice versa.

## BackendPointer\<T, U\>

The `BackendPointer` is the core reference type in TDF. It is how everything in every store is addressed.

A `BackendPointer<T, U>` contains **two things**:

1. A **pointer** to a store item of type `T`
2. **Unique data** of type `U` — non-internable data that travels with the pointer

It is an **enum** with two variants:

```
BackendPointer<T, U>
├── Pointer      — references a single item
└── PointerRange — references a contiguous range of items
```

Key properties:

- **Relative offset** — the pointer value is relative to its store's offset. Combined with the store offset (known by the `BackendView`), it locates the item within the backend.
- **Ordered** — `BackendPointer` must implement `Ord`. This is required for the append-only store's sequential guarantee and for range-based iteration.
- **Hashable** — `BackendPointer` must implement `Hash` (and primitives must too). This makes it easy for consumers to maintain a cache keyed by handle — e.g., a renderer can keep a `HashMap<Handle, LoadedImage>` to avoid re-dereferencing the same data store entry twice.
- **Backend-specific representation** — each backend chooses how to represent pointers:
  - `VecBackend`: pointer = usize index + U data; pointer range = index + length
  - `IPFSBackend`: pointer = CID + U data; pointer range = vec of CIDs

## Unique reduction

As you follow a chain of pointers through a store, each pointer carries unique data `U`. These accumulate via:

```
BackendUniqueData::reduce(self, other) -> Self
```

Every step in the chain produces a new reduced unique that combines the current pointer's unique with all prior uniques.

For [`ItemUnique`](./PRIMITIVES.md#itemunique), reduction works as:
- **Positions add**: `Position(0, 1).reduce(Position(2, 2))` = `Position(2, 3)`
- **Tags merge via right-biased union**: if both sides define the same tag key, the right (deeper) value wins

This is how nested groups work — a group at `Position(10, 20)` containing an item at `Position(5, 0)` yields a final position of `Position(15, 20)`. The item's position is relative to its group, and reduction converts it to an absolute position.

See [Flow](./FLOW.md) for a concrete end-to-end example of unique reduction during page iteration.

## Backend trait methods

Every backend implementation must provide the following methods.

### Core access

| Method | Description |
|--------|-------------|
| `get(BackendPointer<T, U>) → StoreItemCell<T, U>` | Read a single item from the backend |
| `set(BackendPointer<T, U>, item)` | Write an item to the backend |

### Recursive iterators (one per store)

These methods recursively dereference a pointer through a specific store, following all backend pointers and reducing uniques at each step. They return iterators of `(Primitive, reduced Unique)`.

| Method | Returns |
|--------|---------|
| `iter_page_children_rec(BackendPointer<ItemPointer, ()>)` | `Iterator<(ItemPrimitive, reduced ItemUnique)>` |
| `iter_item_children_rec(BackendPointer<ItemPrimitive, ItemUnique>)` | `Iterator<(ItemPrimitive, reduced ItemUnique)>` |
| `iter_data_children_rec(BackendPointer<DataPrimitive, ()>)` | `Iterator<(DataPrimitive, ())>` |
| `iter_signature_children_rec(BackendPointer<SignaturePrimitive, SignatureUnique>)` | `Iterator<(SignaturePrimitive, reduced SignatureUnique)>` |

Note that `iter_page_children_rec` takes a page store pointer (`BackendPointer<ItemPointer, ()>`) but returns item store results (`ItemPrimitive, ItemUnique`). This is because the page store's primitives *are* item pointers — the recursive iterator crosses from the page store into the item store and continues recursing there.

### Non-recursive pointer dereferencing

Each store also has a non-recursive variant that dereferences a single pointer one level, returning a `StoreItemCell` (which may be a primitive or another backend pointer). These are the building blocks that the recursive iterators use internally.

## Concrete backend examples

### VecBackend

The simplest backend, useful for testing.

- **Pointer**: a `usize` index into a vector, plus `U` data
- **PointerRange**: a `usize` index + `usize` length (a slice of contiguous vector entries)
- All four stores live in a single `Vec`, partitioned by offsets

Example layout with offsets and views:

```
Vec: [ page₀ page₁ | item₀ item₁ item₂ item₃ | data₀ data₁ | sig₀ ]
      ↑ offset 0     ↑ offset 2                 ↑ offset 6    ↑ offset 8

page_view:  BackendView<ItemPointer, VecBackend>  { offset: 0 }
item_view:  BackendView<ItemPrimitive, VecBackend> { offset: 2 }
data_view:  BackendView<DataPrimitive, VecBackend> { offset: 6 }
sig_view:   BackendView<SigPrimitive, VecBackend>  { offset: 8 }
```

A `BackendPointer` with index `1` in the item store refers to `Vec[2 + 1]` = `item₁`. The view adds its offset (`2`) to the pointer's relative index (`1`) to produce the absolute position in the underlying vector. Each view only knows about its own offset, so the item view cannot address entries in the data region and vice versa.

### IPFSBackend

A content-addressed backend for distributed storage.

- **Pointer**: a CID (content identifier), plus `U` data
- **PointerRange**: a `Vec<CID>` (since IPFS items are not contiguous, ranges are explicit lists)
- Grouping provides a performance benefit: a group of CIDs can be fetched in a single IPFS request
