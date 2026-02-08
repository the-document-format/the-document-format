# TDF File Format

This module contains all the utilities necessary to parse TDF files.

# File Layout

We break down the general layout of a TDF file into the concept of "chunk"s. All TDF files have the following chunks:

1. **Prefix**: File level metadata, like the version number and magic bits.
1. **Header**: This is where all document metadata lives. The title, offsets of the other chunks, etc.
1. **Pages**: This is where all of the document pages live. This chunk contains pointers to offsets in the next chunk.
1. **Store**: Where all the actual data in the document lives. This is stored as a DAG where the leaves are primative data items (like images, fonts, etc).
1. **Trailer**: This chunk has full-document trailing metadata like signatures.
1. **Suffix**: File level metadata that must come at the end, like a checksum.

Every chunk is (de)serializable, so if we are able to read the full contents of a given chunk, we can (de)serialize it. For example, we do not need to read the entire file from disc in order to be able to de(serialize) just the **Header**.

# The Store

The store is a DAG that contains all of the actual data that lives inside a TDF. Every page has a list of pointers to items in this DAG that belong on the page. When we follow the pointers, we may find additional pointers if the items have been interned.

The actual way that we store this data involves:
- **a "store" container**: This is the highest level container that stores everything. It is a vector of references to store items.
- **a "store item" container**: This is a container for a piece of data. There are two flavors of **store item**s -- **page items**, which are real pieces of data that show up on the page, or **meta items**, which are things like fonts, tags, or other pieces of metadata that do not actually appear directly on the page. For the sake of interning, it is important that we store the additional *transformation* data, like where a given **primative item** is on the page, inside this **"store item" container**. In addition to storing the kind of store item it is, and, depending on the kind, extra metadata like the position, a **store item** always stores either a **store pointer**, which points to another spot in the **store container** where you can find a **store item** that is not a pointer, or an actual **primative** item. 
- **the "primative" items**: This is a piece of actual data. It could be a font, chunk of text, image, shape, or really anything.

We store a little bit of higher-level transformation metadata in these pointers that lives above the actual primative data.

Consider a page that has a few images on it, followed by another page with more images, where one image on the second page is the exact same image, except maybe in a slightly different position.
