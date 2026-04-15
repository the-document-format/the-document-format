# Manifest

The **manifest** is the structured portion of a TDF file that describes the document independently of its backend storage. In code it is represented as `TDFManifest<B>`, which is generic over the backend type `B`. It contains two segments: **meta** and **pages**.

Every TDF file also begins with a **header** that precedes the manifest. The header is backend-specific: its layout, field types, and byte size depend on the concrete backend in use. The binary backend header is defined in [docs/backends/BIN.md](./backends/BIN.md). The header's job is to carry the byte offsets needed to locate the manifest segments and the backend stores within the file.

## Meta

The meta segment contains document-level metadata. None of this data is needed to render pages; it exists for search, display, and organizational purposes.

| Field | Description |
|-------|-------------|
| Title | Document title |
| Search index | Pre-built index for full-text search within the document |
| Table of contents | Structural outline of the document |
| Tags | Key-value metadata — authors, creation date, etc. |

## Pages

The pages segment contains **fixed-size page metadata** for every page in the document. Each page entry includes:

| Field | Description |
|-------|-------------|
| Page tags | Per-page metadata (e.g. page dimensions, labels) |
| Page store reference | A reference into the [page store](./BACKEND.md) that links this page to its top-level items |

Because page entries are fixed-size, the reader can seek directly to any page by number: `pages_offset + page_number * page_entry_size`. The page store reference is an entry in the page store (one of the four stores in the [backend](./BACKEND.md)), which in turn points to item store entries that hold the actual page content.

See [Flow](./FLOW.md) for a complete walkthrough of how a page read proceeds from the pages segment through the stores and backend.
