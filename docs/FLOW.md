# End-to-End Read Flow

This document traces a complete read through every layer of TDF, from opening a file to rendering pixels on screen. Each step references the doc where the relevant concept is defined.

## 1. Load the file and read the header

The [reader](./READER.md) opens the file and reads the fixed-size [header](./SEGMENTS.md#header). Because the header size is known at compile time, this is a single read. Afterward the reader knows:

- Segment offsets (where meta and pages live)
- The checksum (for verifying store integrity)
- Compression settings

## 2. Look up page 40

The caller asks for items on page 40:

```
reader.iter_page_items(40)
```

The reader seeks into the [pages segment](./SEGMENTS.md#pages) at `pages_offset + 40 * page_entry_size` and reads the fixed-size page entry. That entry contains a reference into the **page store**.

## 3. Page store returns an ItemPointer

The page store is an `AppendOnlyStore<ItemPointer, ()>` — one of the [four stores](./PRIMITIVES.md#four-stores-at-a-glance) in the [backend](./BACKEND.md). The entry for page 40 yields an [`ItemPointer`](./PRIMITIVES.md#itempointer-primitive), which is a `BackendPointer<ItemPrimitive, ItemUnique>`.

In practice this is usually a **PointerRange** — a range covering all the top-level items on that page.

## 4. Recursive dereference through the item store

The reader calls `StoreExt::iter_rec(pointer)` on the item store, which delegates to the backend's recursive iterator:

```
backend.iter_item_children_rec(pointer)
```

This is where the core traversal happens. The backend follows the pointer into the item store and finds a mix of:

- **Primitives** (`ItemPrimitive`) — actual items like text boxes and images
- **BackendPointers** — pointers to other slots in the same item store (this is how groups work)

The backend recursively follows all backend pointers, collecting primitives. Along the way, it performs [unique reduction](./BACKEND.md#unique-reduction) — each pointer in the chain carries an `ItemUnique` (position + tags), and these accumulate as the traversal descends.

### Unique reduction example

Consider a group at `Position(10, 20)` with `tags: {opacity: 0.5}`, containing a text box at `Position(5, 0)` with `tags: {opacity: 1.0, font_size: 14}`:

```
Page store entry → ItemPointer (unique: Position(0, 0), tags: {})
  └── PointerRange in item store
        └── Group pointer (unique: Position(10, 20), tags: {opacity: 0.5})
              └── TextBox pointer (unique: Position(5, 0), tags: {opacity: 1.0, font_size: 14})
                    └── TextBox primitive
```

At each step, the uniques are reduced:

1. Start: `Position(0, 0)`, `tags: {}`
2. After group pointer: `Position(0, 0) + Position(10, 20)` = `Position(10, 20)`, `tags: {} ∪ {opacity: 0.5}` = `{opacity: 0.5}`
3. After text box pointer: `Position(10, 20) + Position(5, 0)` = `Position(15, 20)`, `tags: {opacity: 0.5} ∪ {opacity: 1.0, font_size: 14}` = `{opacity: 1.0, font_size: 14}`

Positions add component-wise. Tags merge via right-biased set union: the text box's `opacity: 1.0` overwrites the group's `opacity: 0.5`, while `font_size: 14` is new and passes through unchanged.

## 5. Iterator yields (ItemPrimitive, reduced ItemUnique)

The caller receives a lazy iterator where each element is a tuple:

- **ItemPrimitive** — the concrete item (TextBox, Image, Vector, Shape)
- **ItemUnique** — the fully reduced unique (accumulated position, merged tags)

No data store content has been loaded yet. The iterator only touches the page store and item store.

## 6. Renderer draws primitives

The renderer iterates and draws each primitive at its reduced position. When it encounters a primitive that contains a [Handle](./PRIMITIVES.md#handles) — for example, an `Image` with a `data` handle or a `TextBox` with a `font` handle — it needs to load the referenced data.

## 7. Lazy data loading via deref_handle

The renderer calls:

```
reader.deref_handle(handle) → DataPrimitive
```

The handle is a `BackendPointer<DataPrimitive, ()>` pointing into the data store. The reader resolves it through the data store and backend, returning the `DataPrimitive` (e.g., `ImageData` with raw bytes, or `FontData` with font bytes).

This is lazy — data is only loaded when the renderer actually needs it. A page with ten images but only three visible in the viewport only loads three blobs.

## Summary of the delegation chain

```
Reader
  ├── pages segment → page entry → page store reference
  │
  ├── page store (AppendOnlyStore<ItemPointer, ()>)
  │   └── returns ItemPointer (BackendPointer<ItemPrimitive, ItemUnique>)
  │
  ├── item store via StoreExt::iter_rec → backend.iter_item_children_rec
  │   └── recursively follows refs, reduces uniques
  │   └── yields (ItemPrimitive, reduced ItemUnique) iterator
  │
  └── data store via deref_handle
      └── resolves Handle → DataPrimitive (font bytes, image bytes)
```

Each layer only knows about the layer directly below it: reader → store → backend. The reader never touches the backend directly, and the store never reads raw bytes — that is the backend's job.
