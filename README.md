# The Document Format

## Abstract

We present TDF (The Document Format), an open binary document format designed as a simpler, smaller, and faster alternative to the PDF. TDF targets static graphical content (documents, vector graphics, print media) and deliberately excludes scripting and interactivity in favor of a minimal, easy-to-implement specification. The format uses a binary-first layout divided into fixed-size segments that enable random page access and streaming reads without full deserialization. Document content is organized into a directed acyclic graph of typed stores, allowing identical items to be interned across the entire file for compact storage while supporting lazy, page-at-a-time loading for fast time-to-screen. Large data such as fonts and images is referenced indirectly and resolved on demand, and all references are hashable to support consumer-side caching. The store layer is abstract over a backend trait, decoupling logical document structure from physical storage so that the same reader works against in-memory vectors, byte arrays, IPFS, or other media. Two classes of store frontends enforce distinct invariants: append-only stores preserve insertion order for cryptographic signing, while optimized stores intern and group items for size efficiency. Text is stored with flow information preserved across line breaks, enabling reliable full-text search and copy. The format supports cheap page reordering and manipulation without re-encoding, self-contained font embedding for cross-platform consistency, and an append-only signature chain for tamper-evident document history. A companion intermediate text format (TDFI) is planned to lower the barrier for tooling that produces TDF files. Reference tooling includes a Rust parsing and rendering library and a portable web-based viewer built on wgpu and vello.

## Documentation

- [Overview](OVERVIEW.md) — Project goals, scope, background, and comparison to PDF.
- [Segments](docs/SEGMENTS.md) — File-level layout: header, meta, and pages segments.
- [Reader](docs/READER.md) — The high-level interface for reading TDF documents, generic over a backend.
- [Stores](docs/STORE.md) — The trait that sits between reader and backend, managing typed items via BackendView.
- [Primitives](docs/PRIMITIVES.md) — All primitive and unique type definitions for the four stores.
- [Frontends](docs/FRONTEND.md) — Store implementations: AppendOnlyStore and OptimizedStore.
- [Backends](docs/BACKEND.md) — The trait for physical storage, BackendPointer, BackendView, and unique reduction.
- [Flow](docs/FLOW.md) — End-to-end read flow from opening a file to rendering a page.

## General flow

Reading a TDF document follows this path:

1. The [reader](docs/READER.md) opens the file and reads the fixed-size [header](docs/SEGMENTS.md#header), learning segment offsets and the checksum.
2. To read a page, the reader looks up the page entry in the [pages segment](docs/SEGMENTS.md#pages) and gets an `ItemPointer` from the [page store](docs/PRIMITIVES.md#page-store-types).
3. The item store recursively dereferences the pointer, [reducing uniques](docs/BACKEND.md#unique-reduction) along the way, and returns an iterator of `(ItemPrimitive, reduced ItemUnique)`.
4. The renderer draws each primitive. When it encounters a [Handle](docs/PRIMITIVES.md#handles) (a lazy reference to the data store), it calls `reader.deref_handle()` to load the data on demand.

See [Flow](docs/FLOW.md) for the full detailed walkthrough.
