# VecBackend

`VecBackend` is the simplest concrete backend implementation. It stores all four stores in memory as plain Rust `Vec`s and serializes the entire document to JSON. It exists for testing, tooling, and as a reference implementation -- not for production document storage.

## Structure

Each of the four stores is its own typed `Vec`:

```
page_store:  Vec<StoreItemCell<PageTypes<VecTypes>, VecTypes>>
item_store:  Vec<StoreItemCell<ItemTypes<VecTypes>, VecTypes>>
data_store:  Vec<StoreItemCell<DataTypes, VecTypes>>
sig_store:   Vec<StoreItemCell<SignatureTypes, VecTypes>>
```

Unlike a byte-array backend, there is no shared underlying buffer. The stores are independent and address their elements by integer index, not byte offset.

## Pointer types

`VecBackend` uses `VecTypes` as its `BackendTypes`. `VecTypes` defines two concrete pointer shapes:

### VecSinglePointer

```
VecSinglePointer<S> {
    index:  usize       -- absolute index into the store's Vec
    unique: S::Unique   -- inline unique data for this item
}
```

`index` is the position of the cell in the store's `Vec` at the time of insertion. Because items are only ever appended (never reordered), the index is stable for the lifetime of the backend.

### VecGroupPointer

```
VecGroupPointer<S> {
    range:   VecRange { start: usize, len: usize }   -- contiguous slice of the Vec
    uniques: Vec<S::Unique>                           -- one unique value per item in range
}
```

Groups require that all grouped items occupy a **contiguous range** in the store's `Vec`. This is enforced by the builder, which pushes all items for a group before calling `group_together`. There is no indirection table -- a group is just a start index and a count.

## BackendAccess behavior

### push_cell

Appends a `StoreItemCell::StorePrimitive(primitive)` to the relevant store and returns a `BackendPointer::Single` pointing to the new entry:

```
offset = store.len()
store.push(StoreItemCell::StorePrimitive(primitive))
return BackendPointer::Single(VecSinglePointer { index: offset, unique })
```

The unique value is stored in the pointer, not in the cell. Cells only hold the primitive.

### get_cells

For a `Single` pointer, indexes directly into the `Vec`:

```
store.get(pointer.index)  ->  Some(&cell) or None
```

For a `Group` pointer, takes a contiguous slice:

```
store.get(range.start .. range.start + range.len)  ->  Some(&[cell, ...]) or None
```

### group_together

Expects all input pointers to be `Single` pointers that were pushed contiguously. Records the start index of the first item and the count, plus all the uniques:

```
start   = items[0].index
len     = items.len()
uniques = [items[0].unique, items[1].unique, ...]
return BackendPointer::Group(VecGroupPointer { range: VecRange { start, len }, uniques })
```

Nested groups (a group containing another group) are not currently supported.

### expand_group

Reconstructs individual `Single` pointers from a group by pairing each index in the range with its corresponding unique:

```
for i in 0..group.len:
    BackendPointer::Single(VecSinglePointer { index: group.start + i, unique: group.uniques[i] })
```

## Serialization format

The entire `VecBackend` is serde-serialized as a single JSON value containing all four stores. Each store is a JSON array of `StoreItemCell` values. The complete document wire format is:

```
<ASCII decimal length><JSON-encoded HeaderSegment>
<ASCII decimal length><JSON-encoded MetaSegment>
<ASCII decimal length><JSON-encoded PagesSegment>
<ASCII decimal length><JSON-encoded VecBackend>
```

Each section is prefixed with its byte length written as ASCII decimal digits (no separator between the length and the JSON body). To read section N, the reader scans ASCII digits until the first non-digit, parses the count, then reads exactly that many bytes as JSON.

There is no seeking. Reading a single page requires deserializing the entire file.

## Limitations

**No random access.** To read page 40, the entire backend must be deserialized into memory first. There is no way to seek to a specific page or item.

**Verbose on disk.** JSON encodes integers as decimal strings and includes all field names. A document with many items is significantly larger on disk than a compact binary encoding.

**No byte-offset pointers.** Pointers are `usize` indices, not byte offsets. This means the format cannot be memory-mapped, streamed, or accessed lazily -- the full document must be loaded before any pointer can be resolved.

**Contiguity requirement for groups.** `group_together` requires that items are already adjacent in the Vec. The builder enforces this by pushing items in the right order, but it is an implicit invariant rather than an enforced one.

These limitations are acceptable for testing and development. The [binary backend](./BIN.md) is designed to address all of them.
