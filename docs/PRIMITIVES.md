# Primitives and Uniques

Every [store](./STORE.md) is generic on two types: a **Primitive** (the internable, shared data) and a **Unique** (the non-internable data that is always unique to a given instance on the page — things like position and styling that differ even when two items share the same primitive). The unique rides along with every [BackendPointer](./BACKEND.md#backendpointert-u). This file defines the concrete types for all four stores.

## TDF's Four Stores

| Store | Frontend | Primitive | Unique |
|-------|----------|-----------|--------|
| Page store | [AppendOnlyStore](./FRONTEND.md#appendonlystore) | `ItemPointer` | `()` |
| Item store | [OptimizedStore](./FRONTEND.md#optimizedstore) | `ItemPrimitive` | `ItemUnique` |
| Data store | [OptimizedStore](./FRONTEND.md#optimizedstore) | `DataPrimitive` | `()` |
| Signature store | [AppendOnlyStore](./FRONTEND.md#appendonlystore) | `SignaturePrimitive` | `SignatureUnique` |

## Handles

A **Handle** is a `BackendPointer<DataPrimitive, ()>` embedded inside an item primitive. It is a lazy cross-store reference — the item store entry carries the handle, but the data it points to lives in the data store and is only loaded on demand via [`reader.deref_handle()`](./READER.md).

Handles exist so that large blobs (fonts, images) are not loaded when iterating page items. The renderer encounters a handle and can choose when to resolve it. Because both pointers and primitives implement `Hash`, consumers can maintain a cache keyed by handle (e.g., `HashMap<Handle, LoadedImage>`) to avoid resolving the same blob twice.

## Page store types

### `ItemPointer` (Primitive)

An `ItemPointer` is a `BackendPointer<ItemPrimitive, ItemUnique>` — a pointer from the page store into the item store. Each page store entry maps a page to the top-level items on that page. In practice this is usually a `PointerRange` covering all items on the page.

### `()` (Unique)

The page store carries no unique data. Pages are purely structural — they are entry points into the item store, not data-bearing entities themselves.

## Item store types

### `ItemPrimitive`

An enum of everything that can appear on a page. Each variant carries only its core data — visual styling (font, font size, colors, stroke width, opacity, text alignment, etc.) lives in the `ItemUnique` tags, not in the primitive.

- **TextBox** — the raw text content, plus a Handle to font data
- **Image** — a Handle to image data, width, height
- **Vector** — list of bezier points
- **Shape** — a `ShapeKind` enum (circle, rectangle, TDF logo, etc.)

`TextBox` and `Image` contain **Handles** — `BackendPointer<DataPrimitive, ()>` pointers into the data store, resolved lazily by the renderer.

### `ItemUnique`

The non-internable data attached to every item pointer:

- **position** — a `Position` struct (x, y coordinates)
- **tags** — `ItemTags`, a set of optional key-value styling/metadata that applies to any item kind

`ItemTags` are not specific to any one primitive variant. They include things like font, font size, stroke width, stroke color, fill color, opacity, and text alignment. Because tags live in the unique (not the primitive), they participate in [unique reduction](./BACKEND.md#unique-reduction) and can be inherited through groups.

`ItemUnique` implements `reduce`:
- **position**: adds component-wise — `Position(0, 1).reduce(Position(2, 2))` = `Position(2, 3)`
- **tags**: right-biased set union — if both sides define the same tag key, the right (deeper) value wins

## Data store types

### `DataPrimitive`

An enum of large, blobular data that is not loaded eagerly:

- **FontData** — raw font file bytes
- **ImageData** — raw image file bytes

### `()` (Unique)

The data store carries no unique data. Data store items (fonts, images) are purely shared blobs — every reference to the same font or image points to the same data with no per-instance variation.

## Signature store types

### `SignaturePrimitive`

- **Signature** — public key, hash of the entire document (including this signature's own public key), timestamp

The signature store is [append-only](./FRONTEND.md#appendonlystore), which guarantees that everything before signature N was written before N. Each signature signs over all prior content, creating a verifiable chain.

### `SignatureUnique`

TBD — will be defined as the signature store's requirements become clearer.
