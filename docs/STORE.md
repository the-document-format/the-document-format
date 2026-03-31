# Frontend

A frontend sits between the [reader](./READER.md) and the [backend](./BACKEND.md). It manages typed items — the reader asks the frontend for what it needs and gets back correctly typed results without knowing anything about how or where data is physically frontendd.

A frontend is a **trait**, generic on two type parameters:

```
Frontend<Primitive, Unique>
```

- **Primitive** — the internable data type (e.g., `ItemPrimitive`, `DataPrimitive`)
- **Unique** — the non-internable data carried by every pointer (e.g., `ItemUnique`, `()`)

See [Primitives](./PRIMITIVES.md) for the concrete types used by each of the four frontends.

## How frontends access the backend — BackendView

A frontend does **not** hold a reference to the full backend. Instead, it holds a `BackendView`:

```
BackendView<Primitive, B: Backend>
```

A `BackendView` is a lightweight struct constructed with the **offset** for that frontend's region within the backend. It provides type-safe access scoped to just that frontend's data:

- `view.get(&backend)` — reads from the backend at the correct offset
- `view.set(&mut backend, ...)` — writes to the backend at the correct offset

The view is generic on both the frontend's primitive type and the concrete backend type. Each frontend gets its own view, and the view is what prevents one frontend from accidentally reading another frontend's data.

See [BackendView](./BACKEND.md#backendview) for more on how views are constructed and how they communicate with the backend.

## Frontend trait operations

Every frontend implementation must provide these operations:

### `push(item) → Handle`

Insert an item into the frontend. Returns a handle (a pointer to the inserted item) that can be used to retrieve it later.

### `get(pointer) → FrontendItemCell`

Retrieve a single item from the frontend given a pointer. Returns a `FrontendItemCell` — see [below](#frontenditemcell).

### `size() → usize`

The number of items currently in the frontend.

### `group(items) → FrontendItemGroupCell`

Combine multiple items in the frontend into a single group. Groups are a backend-level concept that allows related items to be frontendd and fetched together efficiently.

### `iter() → Iterator<FrontendItemCell>`

Iterate over **all** items in the frontend. This is used by `FrontendExt::checksum()` to compute a rolling hash over the entire frontend.

## FrontendExt

`FrontendExt` is auto-implemented for all types that implement `Frontend`. It provides higher-level utilities that are generic across all frontends.

### `iter_rec(pointer) → Iterator<(Primitive, reduced Unique)>`

Recursively dereference a single pointer. Follows the pointer through the frontend, resolving any intermediate backend pointers, and yields `(Primitive, reduced Unique)` pairs for every leaf primitive reachable from the pointer.

Delegates to the backend's recursive iterator, which handles [unique reduction](./BACKEND.md#unique-reduction) — accumulating unique data at each pointer hop.

A single pointer is just the degenerate case of a range, so this uses the same backend machinery as `iter_range_rec`.

### `iter_range_rec(pointer_range) → Iterator<(Primitive, reduced Unique)>`

Same as `iter_rec` but for a range of pointers. Also delegates to the backend's recursive iterator. Both methods return iterators and are named consistently since single dereference is the one-item case of a range.

### `checksum() → Hash`

Computes a rolling hash over all items in the frontend by iterating via `iter()`. The [header](./SEGMENTS.md#header) frontends the checksum for integrity verification.

## FrontendItemCell

When you read from a frontend, you get back a `FrontendItemCell` — an enum that is either:

- **`FrontendPrimitive(T)`** — a leaf item: the actual primitive data
- **`BackendPointer`** — a pointer or pointer-group into the **same** frontend, indicating that further dereferencing is needed

```
FrontendItemCell<Primitive, Unique>
├── FrontendPrimitive(Primitive)
└── BackendPointer(BackendPointer<Primitive, Unique>)
```

The recursive iteration methods in `FrontendExt` handle `BackendPointer` resolution automatically — callers get back only resolved primitives with their reduced uniques.

## Pseudocode

```rust
trait BackendPointerType = Ord + Hash;

pub enum FrontendItemCell<
    Primitive,
    Unique,
    BackendPointer<Primitive, Unique>: BackendPointerType,
> {
    BackendPointer(BackendPointer<Primitive, Unique>),
    FrontendPrimitive(Primitive),
}
```
