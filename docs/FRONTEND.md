# Frontends

A frontend is an implementation of the [Store](./STORE.md) trait that enforces additional invariants about how data is organized. The store trait defines *what* operations are available; the frontend defines *how* items are laid out and what guarantees hold. The actual byte layout is not guaranteed by the backend but the ordering and grouping is.

Every store in TDF uses one of two frontends:

| Frontend | Used by | Key guarantee |
|----------|---------|---------------|
| AppendOnlyStore | Page store, Signature store | Insertion order is preserved and meaningful |
| OptimizedStore | Item store, Data store | Identical primitives are interned; items are grouped |

## AppendOnlyStore

```
AppendOnlyStore<Primitive, Unique>
```

An append-only frontend guarantees that pointers are **sequential and ordered**. You can only add items to the end — never insert, reorder, or delete. This means:

- Everything before pointer N was written before N.
- The ordering of pointers reflects the temporal ordering of writes.

This property is critical for two use cases:

**Signature store**: Signatures sign over all prior content in the document. The append-only guarantee means that when you verify signature N, you know that all content referenced by pointers < N existed at the time of signing. Without this, a signature could be invalidated by reordering.

**Page store**: Pages are stored as `AppendOnlyStore<ItemPointer, ()>`. The append-only ordering ensures pages are numbered sequentially and can be looked up by index. The unique type is `()` because pages carry no unique data — they are purely structural pointers into the item store.

See [Primitives](./PRIMITIVES.md) for the concrete types used by each store.

## OptimizedStore

```
OptimizedStore<Primitive, Unique>
```

An optimized frontend groups items together and **interns identical primitives** — if two items have the same primitive data, they share a single copy in storage. This is the frontend for stores where deduplication matters:

**Item store**: Multiple pages may reference the same vector graphic or text style. The optimized store ensures these are stored once and referenced by pointer, taking advantage of the [backend's](./BACKEND.md) pointer and pointer-group layout.

**Data store**: Font data and image data are often shared across many items. Interning prevents storing the same 500KB font file for every text box that uses it.

The optimized store makes **no ordering guarantees**. Items may be rearranged internally to improve grouping and deduplication. If you need ordering, use `AppendOnlyStore`.
