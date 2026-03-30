# The Reader

The `TDFReader` is the highest-level interface for reading a TDF document. It is the entry point that users interact with — all access to document content flows through the reader.

## Generic over a Backend

The reader is generic over a concrete [backend](./BACKEND.md) implementation:

```
TDFReader<B: Backend>
```

This means the same reader logic works regardless of whether the underlying storage is a `VecBackend`, `IPFSBackend`, `ByteArrayBackend`, or any other type that implements the `Backend` trait. Helper constructors create readers with specific backend types.

## Operations

### `iter_page_items(page_number) → Iterator<(ItemPrimitive, ItemUnique)>`

Returns a lazy iterator of all items on the given page. Each element is a tuple of:

- An [`ItemPrimitive`](./PRIMITIVES.md#itemprimitive) — the concrete item (TextBox, Image, Vector, Shape)
- A reduced [`ItemUnique`](./PRIMITIVES.md#itemunique) — the accumulated position and tags after [unique reduction](./BACKEND.md#unique-reduction) through the entire pointer chain

Under the hood, this:
1. Looks up the page entry in the [pages segment](./SEGMENTS.md#pages)
2. Reads the page's `ItemPointer` from the [page store](./PRIMITIVES.md#page-store-types)
3. Delegates to `StoreExt::iter_rec()` on the item store, which calls the backend's recursive iterator

See [Flow](./FLOW.md) for a full trace of this path.

### `deref_handle(handle: BackendPointer<DataPrimitive, ()>) → DataPrimitive`

Resolves a [Handle](./PRIMITIVES.md#handles) — a lazy reference from an item primitive into the data store. Returns the [`DataPrimitive`](./PRIMITIVES.md#dataprimitive) (e.g., font bytes or image bytes).

Handles appear inside item primitives like `TextBox.font` and `Image.data`. The renderer calls `deref_handle` when it actually needs the data, keeping page iteration lightweight.

### Segment getters

The reader exposes getters for the parsed [segments](./SEGMENTS.md):

- **Header** — magic bytes, version, file length, compression, segment offsets, checksum
- **Meta** — title, search index, table of contents, tags
- **Pages** — fixed-size page entries with page tags and page store references

These are populated when the reader is constructed (header is read immediately; meta and pages are read on demand or eagerly depending on the backend).

## Delegation model

The reader never accesses the backend directly. All data retrieval follows the chain:

```
Reader → Store → Backend
```

The reader asks a [store](./STORE.md) for data. The store uses its [BackendView](./BACKEND.md#backendview) to read from the backend. This separation means the reader is independent of storage details, and stores are independent of each other.
