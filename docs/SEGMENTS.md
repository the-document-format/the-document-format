# Segments

A TDF file is divided into three contiguous segments: the **header**, **meta**, and **pages**. Each segment has a fixed role and can be located independently once the header is read.

```
┌──────────┬──────────┬──────────┐
│  Header  │   Meta   │  Pages   │
└──────────┴──────────┴──────────┘
```

## Header

The header is **fixed-size** and is always read first. Because its size is known ahead of time, the reader can parse it in a single read without any seeking. It contains:

| Field | Description |
|-------|-------------|
| Magic bytes | Identifies the file as TDF |
| Version | Format version number |
| File length | Total byte length of the file |
| Compression | Compression scheme used for the other segments (if any) |
| Segment offsets | Byte offsets to the start of the meta and pages segments |
| Checksum | Rolling hash over all store contents (see [Store](./STORE.md) for how checksums are computed) |

The segment offsets are what make random access possible — once you have the header, you can jump directly to any segment without scanning through the file.

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
