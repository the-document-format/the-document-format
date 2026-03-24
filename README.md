# The Document Format

TDF is a document format designed for efficient, composable document storage and retrieval. Documents are organized into segments — header, metadata, pages, page store, data store, footer, and suffix — that can be read independently and lazily. The format is backend-agnostic: the same document structure works whether data lives in a local vector, on IPFS, or anywhere else. Built-in support for data deduplication, cryptographic signing, and lazy loading means documents stay compact and fast to access without loading everything into memory.

## Structure

- [Reader](docs/READER.md) — The high-level interface for reading TDF documents, delegating to stores for data retrieval.
- [Stores](docs/STORE.md) — Thin wrappers over backends that manage typed primitives: item, data, and signature stores.
- [Frontends](docs/FRONTEND.md) — Store implementations that enforce additional invariants like append-only ordering or deduplication.
- [Backends](docs/BACKEND.md) — Low-level data structures for grouped, ordered, key-value storage (e.g. VecBackend, IPFSBackend).