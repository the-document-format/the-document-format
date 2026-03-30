# Stores

A store sits between the [reader](./READER.md) and the [backend](./BACKEND.md). It manages typed items — the reader asks the store for what it needs and gets back correctly typed results without knowing anything about how or where data is physically stored.

A store is a **trait**, generic on two type parameters:

```
Store<Primitive, Unique>
```

- **Primitive** — the internable data type (e.g., `ItemPrimitive`, `DataPrimitive`)
- **Unique** — the non-internable data carried by every pointer (e.g., `ItemUnique`, `()`)

See [Primitives](./PRIMITIVES.md) for the concrete types used by each of the four stores.

## How stores access the backend — BackendView

A store does **not** hold a reference to the full backend. Instead, it holds a `BackendView`:

```
BackendView<Primitive, B: Backend>
```

A `BackendView` is a lightweight struct constructed with the **offset** for that store's region within the backend. It provides type-safe access scoped to just that store's data:

- `view.get(&backend)` — reads from the backend at the correct offset
- `view.set(&mut backend, ...)` — writes to the backend at the correct offset

The view is generic on both the store's primitive type and the concrete backend type. Each store gets its own view, and the view is what prevents one store from accidentally reading another store's data.

See [BackendView](./BACKEND.md#backendview) for more on how views are constructed and how they communicate with the backend.

## Store trait operations

Every store implementation must provide these operations:

### `push(item) → Handle`

Insert an item into the store. Returns a handle (a pointer to the inserted item) that can be used to retrieve it later.

### `get(pointer) → StoreItemCell`

Retrieve a single item from the store given a pointer. Returns a `StoreItemCell` — see [below](#storeitemcell).

### `size() → usize`

The number of items currently in the store.

### `group(items) → StoreItemGroupCell`

Combine multiple items in the store into a single group. Groups are a backend-level concept that allows related items to be stored and fetched together efficiently.

### `iter() → Iterator<StoreItemCell>`

Iterate over **all** items in the store. This is used by `StoreExt::checksum()` to compute a rolling hash over the entire store.

## StoreExt

`StoreExt` is auto-implemented for all types that implement `Store`. It provides higher-level utilities that are generic across all stores.

### `iter_rec(pointer) → Iterator<(Primitive, reduced Unique)>`

Recursively dereference a single pointer. Follows the pointer through the store, resolving any intermediate backend pointers, and yields `(Primitive, reduced Unique)` pairs for every leaf primitive reachable from the pointer.

Delegates to the backend's recursive iterator, which handles [unique reduction](./BACKEND.md#unique-reduction) — accumulating unique data at each pointer hop.

A single pointer is just the degenerate case of a range, so this uses the same backend machinery as `iter_range_rec`.

### `iter_range_rec(pointer_range) → Iterator<(Primitive, reduced Unique)>`

Same as `iter_rec` but for a range of pointers. Also delegates to the backend's recursive iterator. Both methods return iterators and are named consistently since single dereference is the one-item case of a range.

### `checksum() → Hash`

Computes a rolling hash over all items in the store by iterating via `iter()`. The [header](./SEGMENTS.md#header) stores the checksum for integrity verification.

## StoreItemCell

When you read from a store, you get back a `StoreItemCell` — an enum that is either:

- **`StorePrimitive(T)`** — a leaf item: the actual primitive data
- **`BackendPointer`** — a pointer or pointer-group into the **same** store, indicating that further dereferencing is needed

```
StoreItemCell<Primitive, Unique>
├── StorePrimitive(Primitive)
└── BackendPointer(BackendPointer<Primitive, Unique>)
```

The recursive iteration methods in `StoreExt` handle `BackendPointer` resolution automatically — callers get back only resolved primitives with their reduced uniques.

## Pseudocode

```rust
trait BackendPointerType = Ord + Hash;

pub enum StoreItemCell<
    Primitive,
    Unique,
    BackendPointer<Primitive, Unique>: BackendPointerType,
> {
    BackendPointer(BackendPointer<Primitive, Unique>),
    StorePrimitive(Primitive),
}
```
