# TDF File Format

This module contains all the utilities necessary to parse TDF files.

# File Layout

We break down the general layout of a TDF file into the concept of "segments"s. All TDF files have the following segments:

1. **Header**: All *fixed-sized* file level metadata. Includes the size of the file, relevant offsets, the compression algorithm used, and similar.
1. **Meta**: All *variable-size* document metadata lives. The title, offsets of the other segments, etc.
1. **Pages**: This is where all of the document pages live. These are all fixed sized segments with metadata about the actual page objects, and the **header** states how many of them there are.
1. **Page Store**: Where all the actual data that shows visibly on pages live. This is stored as a DAG where the leaves are primative data items (like images, fonts, etc).
1. **Data Store**: Similar to the **page store**, but only includes data that does not show up visibly on pages. Uses the same underlying DAG structure.
1. **Footer**: This segments has full-document **variable-sized** trailing metadata like signatures.
1. **Suffix**: File level metadata that must come at the end, like a checksum.

Every segments (besides the store) is (de)serializable via Serde, so if we are able to read the full contents of a given segments, we can (de)serialize it. For example, we do not need to read the entire file from disc in order to be able to de(serialize) just the **Header**.

The store is special. It is structured on disc as an array, and we have a special data structure that abstracts accessing random indexes. For now, we load the entire store into memory in our reader, but eventually we would like to be able to lazily read index ranges from disc.

# The Store

The store is a DAG that contains all of the actual data that lives inside a TDF. Every page has a list of pointers to items in this DAG that belong on the page. When we follow the pointers, we may find additional pointers if the items have been interned.

The store is generic on T and U, where:
- **T**: Non-internable metadata associated with each store item
- **U**: The enum of all actual primitive data types (fonts, text, images, shapes, etc.)

The actual way that we store this data involves:
- **a "store" container**: This is the highest level container that stores everything. It is a vector of references to store items. We have a **store reader** that is able to (eventually, right now not implemented) lazily load arbitrary store items.
- **a "store item" container**: Every item in the store contains the actual store item reference, which is either a *primative* or a pointer to another item in the store which is one, and in the higher level store array, an item of T that stores non-internable metadata.
- **the "primative" items**: This is a piece of actual data of type U. It could be a font, segments of text, image, shape, or really anything.


We store a little bit of higher-level transformation metadata in these pointers that lives above the actual primative data.

Consider a page that has a few images on it, followed by another page with more images, where one image on the second page is the exact same image, except maybe in a slightly different position.
