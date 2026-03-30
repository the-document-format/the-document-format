# The `Backend`s

A backend is a data structure to store grouped, ordered, key-value data. It has no knowledge of the actual data being stored and there is an underlying assumption that the data is faster to query for some notion of a "group." Backends are generic on a "pointer type" and a "pointer group type," and the backend trait lets you query a concrete backend given one of these two reference types.

An example of a backend is the `VecBackend`:

A `VecBackend` defines its pointer as a usize index, and a pointer group as a usize index and usize length. The `VecBackend` stores all of its data in a single vector, and the pointer group is used to reference contiguous slices of the vector. This is a very simple dumb backend for tests.

A more complicated example is the `IPFSBackend`. This backend defines its pointer as a CID, and its pointer group is a vec of CIDs. The `IPFSBackend` stores all of its data in IPFS, and the pointer group is used to reference a group of data from IPFS. In this case we do get our performance gain from using a group by virtue of the fact that we can fetch all of the data in a group with a single IPFS request.

Some of the various backends we hope to define include:

- A IPFS backend
- A bytearray backend
- A vector backend
- A json backend
