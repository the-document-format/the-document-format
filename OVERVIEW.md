# The Document Format (TDF)

TDF aims to be the blazingly fast 🔥, lightweight, and simple defacto document format of the future.

## General Summary

TDF is a dramatically more efficient replacement for the PDF, aiming for smaller file size and a much simpler file format. Unlike the PDF, it will be easy to parse, create, and write software to modify TDFs. We aim for an open standard that is simple and efficient, something that PDF has struggled to achieve. Additionally, TDF aims to resolve a number of pain points, such as slow loads and modifications, with existing PDFs. The ultimate goal is a more universal, exchangeable, and consistent format with clearly defined and predictable behavior.

## Scope and Background

The current defacto standard for immutable, reliable documents is the Portable Document Format Standard (PDF), a complicated and dated file format built upon technologies of the past. Here is a [link of all the PDF standards](https://pdfa.org/resource/pdf-specification-index/) in its entirety. A byproduct of being an extremely complicated standard is that Adobe has a monopoly over fully compliant PDF viewers and editors, which is increasingly worrying for what is meant to be a free exchange format for the world. The current standard, PDF, does not prioritize speed or file size when representing documents, while including completely irrelevant features such as flash support (which has been since removed) or JavaScript support. Such features have resulted in PDF viewer exploits before for arbitrary code execution. TDFs, by comparison, aim to be static pieces of information with the goal of being printed or displayed. Rather than being bulk compressed text or XML-like structures, TDF is a custom small binary format designed to be fast to read and minimal in size. As a team, we have practical experience utilizing typesetting tools such as docs, quarto, typst, and markdown, and have found dealing with oddities of typesetting increasingly frustrating. The goal is a format that is as flexible and smaller than PDF, but easy to create ourselves, allowing for creativity in the typesetting space as a whole.

## Objectives and Technical Overview

We aim for TDF to be versatile enough to support many mediums including books, technical documents, reports, SVG images, graphics, and even other vector-heavy information like laser cutting. TDF is a simple document format for any static graphical information, at full vector quality, lending itself for use within scalable graphics and prints. The scope of the format has deliberately been cut down relative to PDFs, as exemplified by the lack of support for JavaScript and other dynamic content. The TDF file format is self-contained and self-describing, with possible changes in the future for additional features. This allows for consistent rendering across platforms, especially around fonts.

To be a smaller, faster, and more optimized alternative to the PDF, we are using a binary-first format for file compactness and faster reading. This allows us to even further expand the use cases of PDF while remaining true to our original cause, serving as a portable static graphical format for documents and vector graphics. Our binary format is designed to:

- Allow for parsing from an ongoing stream, before the entirety has been received.
- Allow arbitrary page items to be interned across the entire document, minimizing file size.
- Support both item compression and document-wide compression.
- Support random page reading and manipulation without reading the full document, allowing for renderers to minimize time-to-screen when a document is opened.
- Build on the PDF by solving some weak points in its data model of visual page data. For example, our text structure allows for the flow of text to be stored across line breaks, allowing for search and copying of long sections of text, a common issue with PDFs.
- Allow cheap basic manipulation of the document such as page reordering or rotation that doesn't incur a costly re-encoding step.

Additionally, if time allows, we would like to define an intermediate TDFI format, which aims to be an editable, intermediary text version of TDFs for other programs to interface with in an easier manner without the need for consumers to implement their own binary parser and writer. This decision allows even wider adoption of our format, including on the web, where it is significantly easier to deal with text than binary, and utilize our TDFI to TDF renderer, reducing the need for applications to reimplement features.

In order to increase adoption of our format, on top of defining TDFI, we would also like to create interfaces for building documents that follow already established document building interfaces for the purpose of being easy to swap components out in the backends of other projects. We specifically aim to implement the krilla API, a popular rust based crate and interface that is used in popular document definition and rendering library typst. This would gain us easy adoption from typst and a direct way to create TDF documents.

Finally, as our primary product is a document format, we intend to also produce minimal tooling around the document format to allow for the creation and usage of such tools. The minimum product is a TDF renderer, which will use our primary TDF library for the reading and deserialization of TDF files and simply draw the contents to the screen. The renderer will serve as a minimal proof-of-concept for the unique objectives described above. A long-term stretch goal, if time permits, is to extend the renderer into a lightweight editor that allows for modification of the TDF files as well.

## Comparing TDF to PDF

TDFs have a lot of advantages compared to PDFs, but many features of PDFs are intentionally not supported.

| Feature | TDF | PDF |
|---------|-----|-----|
| File Format | Binary (optimized) | Compressed text/XML-like |
| File Size | Designed to be as small as possible | Larger |
| Parsing Complexity | Simple, easy to parse using TDFI intermediate format | Extremely complicated |
| Performance | Fast rendering and loading | Slow rendering |
| Text Search | Effective across line breaks | Struggles with line breaks |
| Interactivity | Static documents only (intentional limitation) | Interactive (JavaScript, forms, etc.) |
| Scripting Support | Not supported | Supported |
| Vendor Lock-in | Open standard, easy to implement | Adobe monopoly on full compliance |
| Page Reordering | Native support (easy) | Difficult |
| Free Scrubbing | Yes (any page lookup) with indexing | No |
| Form Support | Planned (fillable forms) | Yes, but generally hard to parse |
| Cryptographic Signing | Planned, designed to be easy to use | Yes |
| Implementation Complexity | Low (by design) | Very high |

## Technical Decisions

To keep files compact, TDF uses a DAG-based data structure for all item, data, and signature information. This allows deduplication of information as it can be referenced instead of directly stored. Each page is an item; items can reference other items; items can be primitive objects. Primitive objects are visual pieces that represent things like text, shapes, images, etc. and are able to reference data from the data store for things like large texts, image data, fonts, and other pieces of information. This allows the file format to be smaller as well as theoretically faster as we can cache information and rendered outputs to speed up time to first page.

During the rendering process of a TDF, due to this structure, we need not load the entire document into memory at once and rather load only the relevant information needed for the rendered pages. This also allows us to selectively cache information depending on its use later for lower memory usage. The flexible structure of our DAG stores also allows us to pre-cache information we may deem to be needed soon (like next pages) or signature information.

As an abstraction over the raw format of the file, we define interfaces that may produce and write TDF information on the fly. This allows for selective file compression as an abstraction of the raw information within the file, while remaining TDF compliant. This allows us to both support various compression strategies as well as be more versatile and upgradable over time with minimal changes to existing readers and writers.

## Potential Extensions

In the process of designing the TDF file format, we came up with many additional ideas to extend the file format down the line. These ideas include:

- Shaders, or arbitrary kernels to apply to regions or entire TDF pages
- Making an intermediate format that is easily diffable, so that you can safely commit your TDFs to git and be able to easily observe changes over time
- Designing an algorithm to intern arbitrary patterns of items using a Huffman-encoding-like algorithm
- Being able to print out a TDF by exporting to PDF or generating Postscript/bitmaps
- Adding support for form fillables and cryptographic signing with included ink signatures
- Full-text search with document indexing ahead of time
- Accessibility features (screen reader optimization, WCAG compliance)
- A suite of manipulation tools for transforming, rotating, redacting, and combining TDF files
- A TDF-to-text converter for LLM ingestion without the usual line break and ordering issues

## Deliverables

The project produces the following core components:

1. **TDF specification**: A comprehensive technical document detailing the binary file structure of a TDF and the steps required to deconstruct and render one.
2. **TDF Rust library**: A crate that can incrementally parse and read TDF files by page, output raster previews at arbitrary resolution, and produce useful metadata like table of contents.
3. **TDF renderer**: A web application using wgpu/vello (compiled to WASM) that displays TDF files in an HTML canvas with zoom, pan, and metadata display.
4. **TDFI tooling** (stretch): A human-readable intermediate format and a converter from TDFI to finalized TDF binary.

## Team

- **Trevor**: Underlying store data structure, lazy reading, automatic data interning
- **Eric**: Renderer, core primitives for page and data items, native and browser renderer binary
- **Wolf**: Lazy TDF reader, serializer/deserializer, binary file format design, supporting Trevor and Eric as needed
