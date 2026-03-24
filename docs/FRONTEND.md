# `Frontend`s

A frontend is a container to store raw data that is just one level above the backend. It defines contracts for how the data actually gets stored.

In reality, a store is just a trait. A `Frontend` is a implementation of a `Store`, but still specific to some type of data (also usually is some `enum`) for type `T`. The purpose of a `Frontend` is to maintain some additional invariants about how the data is stored. We plan on defining the following types of `Frontend`s:

## `AppendOnlyStore`

The `AppendOnlyStore` is a frontend that makes sure that the data being stored is sequential (pointers are ordered). That way when you sign the document, there is a guarantee that the items you are signing all come before the last item.

## `OptimizedStore`

The `OptimizedStore` is a frontend that automatically groups items and interns identical items. It is used for the `DataStore` and `ItemStore` where we are storing potentially duplicate data. It takes advantage of an abstract "pointer + group" layout (more on this in [Backends](BACKEND.md)).
