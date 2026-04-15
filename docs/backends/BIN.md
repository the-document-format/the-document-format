# Binary Backend

The binary backend stores TDF documents as a compact, seekable binary file. Unlike [VecBackend](./VEC.md), it supports direct page access by byte offset, is safe to load on machines of any endianness (including big-endian and wasm), and is significantly more compact than the JSON format.

The first implementation loads the entire file into a `Vec<u8>` and resolves pointers by slicing into that buffer. A future streaming implementation will use `Read + Seek` directly without loading the full file.

## File layout

```
+------------------------------------------+
|  Header  (u32 length prefix + bincode)   |
+------------------------------------------+
|  Meta    (u32 length prefix + bincode)   |
+------------------------------------------+
|  Pages   (u32 length prefix + bincode)   |
+------------------------------------------+
|  Page store  (back-to-back bincode)      |
+------------------------------------------+
|  Item store  (back-to-back bincode)      |
+------------------------------------------+
|  Data store  (back-to-back bincode)      |
+------------------------------------------+
|  Sig store   (back-to-back bincode)      |
+------------------------------------------+
```

All multi-byte integers are **little-endian** throughout. This ensures compatibility across x86, ARM, big-endian hosts, and wasm (all of which are either natively LE or handle LE explicitly).

## Header

The header always comes first in the file. It is bincode-encoded with a `u32 LE` length prefix, the same framing used for the meta and pages segments. Its fields, in order:

| Field               | Type      | Description                                    |
|---------------------|-----------|------------------------------------------------|
| `magic`             | `[u8; 6]` | Always `b"TREVDF"`                             |
| `version`           | `u8`      | Format version, currently `1`                  |
| `file_len`          | `u64`     | Total file length in bytes                     |
| `meta_offset`       | `u64`     | Byte offset of the meta segment                |
| `pages_offset`      | `u64`     | Byte offset of the pages segment               |
| `page_store_offset` | `u64`     | Byte offset of the page store region           |
| `item_store_offset` | `u64`     | Byte offset of the item store region           |
| `data_store_offset` | `u64`     | Byte offset of the data store region           |
| `sig_store_offset`  | `u64`     | Byte offset of the sig store region            |

Readers must validate:
1. `magic == b"TREVDF"` -- otherwise the file is not a TDF binary document
2. `version == 1` -- otherwise the reader does not know the format

## Meta segment

Located at `meta_offset`. Format:

```
[ u32 LE: byte length of encoded body ][ bincode-encoded MetaSegment ]
```

The `u32 LE` length prefix allows the reader to slice exactly the right number of bytes before passing them to bincode. The bincode encoding uses `bincode::config::standard()` (little-endian integers, varint length prefixes for collections).

## Pages segment

Located at `pages_offset`. Format:

```
[ u32 LE: byte length of encoded body ][ bincode-encoded PagesSegment<BinaryTypes> ]
```

The body is a bincode-encoded `Vec<PageStorePointer<BinaryTypes>>`. Each element is a `BackendPointer` using `BinaryTypes` pointer types -- specifically, a byte offset into the page store.

> **TODO: fixed-size page entries**
>
> The current format deserializes all page pointers at once. A future revision will replace the length-prefixed Vec with a contiguous array of fixed-size page entries, enabling O(1) *lazy* random page access by page number. Because page entries contain only fixed-width fields (integer dimensions, a fixed-width byte offset pointer), they can be encoded with a fixed-width bincode configuration so that every entry is the same serialized size. This makes page N directly addressable without scanning. The header already has a dedicated `pages_offset` field, so no other changes to the header are needed.

## Backend stores

The four store regions (page, item, data, sig) are contiguous byte buffers. Their start offsets are in the header; their end offsets are the start of the next region (or `file_len` for the sig store).

Each store is a sequence of **back-to-back bincode-encoded `StoreItemCell` values** with no separator or index between them. Records are never scanned sequentially -- they are always accessed by byte offset using a pointer. To read a record:

```rs
(cell, _bytes_consumed) = bincode::serde::decode_from_slice(&buf[offset..], config)
```

Bincode determines the end of each record implicitly from the type structure. The `_bytes_consumed` return value is available if needed (e.g. for building an index), but is not required for pointer-based access.

Each cell stores only the **primitive** value (`StoreItemCell::StorePrimitive(primitive)`). The unique value for each item is stored in the pointer, not in the cell -- the same design as VecBackend.

Primitives may themselves contain pointers to other stores. For example, an `Image` primitive holds a `DataStorePointer<BinaryTypes>` referencing its pixel data in the data store, and a `PageStorePrimitive` holds an `ItemPointer<BinaryTypes>` referencing its items in the item store. These embedded pointers are `BinaryTypes` pointer values and are bincode-encoded inline as part of the cell record, just like any other field. Following a pointer means decoding the cell to extract the embedded pointer, then using its byte offset to read from the target store (and also relative to the store of the thing being pointed at).

## Pointer types

`BinaryBackend` uses `BinaryTypes` as its `BackendTypes`. All byte offsets in the binary backend are wrapped in a newtype to prevent accidental mixing with other integer values:

```rs
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, Constructor)]
struct Offset(u64);
```

`BinaryTypes` defines two concrete pointer shapes:

### BinarySinglePointer

```rs
BinarySinglePointer<S> {
    offset: Offset    // byte offset into the store buffer
    unique: S::Unique // inline unique data for this item
}
```

`offset` is the byte position of the record within the store's region of the file. To resolve the pointer, slice `buf[store_region_start + offset.0..]` and decode one record with bincode.

### BinaryGroupPointer

```rs
BinaryGroupPointer<S> {
    start:   Offset         // byte offset of the first item in the group
    len:     u32            // number of items in the group
    uniques: Vec<S::Unique> // one unique value per item, in order
}
```

Grouped items must be **contiguous** in the store buffer -- pushed back-to-back with no other records interleaved between them. The builder enforces this the same way as `VecBackend`: all items for a group are pushed before `group_together` is called. To expand a group, decode `len` records starting from `start`, advancing through the buffer by the number of bytes each decode consumes. When you push we just append even if it means that we will have duplicates -- this is something we will not bother to optimize for now.

Both pointer types are serde-serialized. They appear embedded inside segments (e.g. `PagesSegment` holds `Vec<PageStorePointer<BinaryTypes>>`) and inside item primitives (e.g. `Image.data` is a `DataStorePointer<BinaryTypes>`).

## Cache

`BinaryBackend` owns a `TdfBinCache` that avoids re-deserializing records on repeated access and having to clone over and over again. The cache uses interior mutability so that `get_cells` can take `&self` while still populating the cache on misses. We will use a mutex.

### StoreKind

An enum identifying which of the four stores a cache entry belongs to:

```rs
enum StoreKind { PageStore, ItemStore, DataStore, SigStore }
```

### BackendCacheKey

The lookup key. Uniqueness is (store, offset) -- the unique value is intentionally excluded from the key because the same record may be referenced with different unique values by different pointers.

```rs
#[derive(Hash, Eq, PartialEq)]
struct BackendCacheKey {
    store:  StoreKind,
    offset: Offset,
}
```

### BackendCacheValue

A typed union of the four concrete cell types. Each variant carries the fully deserialized `StoreItemCell` for its store:

```rs
enum BackendCacheValue {
    PageStore(StoreItemCell<PageTypes<BinaryTypes>, BinaryTypes>),
    ItemStore(StoreItemCell<ItemTypes<BinaryTypes>, BinaryTypes>),
    DataStore(StoreItemCell<DataTypes, BinaryTypes>),
    SigStore(StoreItemCell<SignatureTypes, BinaryTypes>),
}
```

Rather than four named accessors, `BackendCacheValue` exposes a single generic method via a sealed `BinaryCacheExtract` trait. Each concrete store type implements the trait by pattern-matching on its own variant:

```rs
trait BinaryCacheExtract: StoreTypes {
    fn extract(value: &BackendCacheValue) 
        -> Option<&StoreItemCell<Self, BinaryTypes>>;
}

impl BackendCacheValue {
    fn as_store<S: BinaryCacheExtract>(&self) 
        -> Option<&StoreItemCell<S, BinaryTypes>> 
    {
        S::extract(self)
    }
}
```

The type parameter is usually inferred from context. Where it is not, the call site is `value.as_store::<PageTypes<BinaryTypes>>()`.

There is an implicit contract that a value stored under a `PageStore` key is always the `PageStore` variant. The `BackendAccess` impls uphold this contract. A `debug_assert` inside each `BinaryCacheExtract` impl confirms it at runtime in debug builds.

### TdfBinCache

A thin wrapper around `schnellru::LruMap<BackendCacheKey, BackendCacheValue>` with three operations:

```rs
impl TdfBinCache {
    fn get(&self, key: &BackendCacheKey) -> Option<&BackendCacheValue>;
    fn insert(&self, key: BackendCacheKey, value: BackendCacheValue);
    fn evict(&self, key: &BackendCacheKey);
}
```

We should have a test that checks that when you ask to use the cache we get back a Cow that is Borrowed and that when you ask to not use the cache we get back a Cow that is Owned.

### CacheHints and cell access methods

```rs
enum CacheHints { Cache, NoCache }
```

`BackendAccess` exposes two methods:

```rust
// Default -- no cache involvement. Returns Cow::Owned for BinaryBackend,
// Cow::Borrowed for VecBackend (which is always in memory).
fn get_cells<'a>(
    &'a self,
    pointer: &BackendPointer<S, B::Types>,
) -> Result<Vec<Cow<'a, StoreItemCell<S, B::Types>>>, TdfBinaryError>;

// Cache-aware variant. Caller opts in explicitly.
fn get_cells_with_cache_hints<'a>(
    &'a self,
    pointer: &BackendPointer<S, B::Types>,
    hints: CacheHints,
) -> Result<Vec<Cow<'a, StoreItemCell<S, B::Types>>>, TdfBinaryError>;
```

`get_cells` is equivalent to calling `get_cells_with_cache_hints` with `CacheHints::NoCache`. It exists so that callers that do not need caching do not have to import or think about `CacheHints`.

**With `CacheHints::Cache`:** check the cache first. On a hit, call `.as_store::<S>()`, `debug_assert` it is `Some`, and return `Cow::Borrowed` pointing into the cache-owned value -- no deserialization, no allocation of the cell itself. On a miss, decode from the byte buffer, insert into the cache, and return `Cow::Borrowed` to the newly inserted entry.

**With `CacheHints::NoCache`:** skip the cache entirely. Decode from the byte buffer and return `Cow::Owned`. Nothing is inserted into or read from the cache.

**VecBackend** ignores `CacheHints` entirely -- both methods always return `Cow::Borrowed` directly from the `Vec`, since the data is already in memory.

A test verifies that `CacheHints::Cache` produces `Cow::Borrowed` and `CacheHints::NoCache` produces `Cow::Owned`.

## Error type

Binary I/O operations use a dedicated error type:

| Variant                    | Cause                                              |
|----------------------------|----------------------------------------------------|
| `InvalidMagic`             | Magic bytes do not match `b"TREVDF"`               |
| `UnsupportedVersion(u8)`   | Version byte is not a known format version         |
| `Encode(EncodeError)`      | Bincode failed to serialize a value                |
| `Decode(DecodeError)`      | Bincode failed to deserialize a value              |
| `BadOffset(u64)`           | A pointer byte offset is out of bounds             |
| `InvalidPointerRef`        | A pointer was followed but no record exists at that location |
| `Io(std::io::Error)`       | Underlying I/O failure                             |

`TdfBinaryError` is defined with `thiserror`. At the `DocumentWrite` and `ManifestRead` trait boundaries (which use `std::io::Result`), binary errors are converted to `std::io::Error` via `std::io::Error::other`.

## Serialization library choices

Everything -- header, meta, pages, and all four store records -- uses `bincode 2` via its serde compatibility module (`bincode::serde::encode_to_vec` / `decode_from_slice`). This means no `#[derive(Encode, Decode)]` is required on any type; serde's `Serialize`/`Deserialize` derives are sufficient.

Bincode configuration used everywhere: `bincode::config::standard()`, which is little-endian with varint-encoded integers. One config, applied uniformly.

**Why not postcard:** same varint approach as bincode standard config, but less commonly used and offers no advantage here.

**Why not rkyv:** rkyv uses native byte order by default. Making it endian-safe requires replacing every integer field with `rend::LittleEndian<u32>` etc. throughout the entire data model. Given that TDF files are written on mixed-endian machines and read in wasm (which is always LE), this is impractical.

## New Cargo dependencies

Add to `tdf-engine/Cargo.toml`:

```toml
bincode    = { version = "2", features = ["serde"] }
schnellru  = "0.2"
```

## Future TODOs

| Item | Description |
|------|-------------|
| Fixed-size pages segment | Replace length-prefixed Vec with fixed-stride contiguous entries; enables O(1) page seek |
| Streaming backend | Replace full-file `Vec<u8>` load with `Read + Seek`; pointers become file seeks |
| Generic builder | Unify `VecBackend` and `BinaryBackend` builders into `TDFBuilder<B: Backend>` |
