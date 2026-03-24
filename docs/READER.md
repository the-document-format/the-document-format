# The `Reader`

The `Reader` is the highest level interface that users use to read a TDF document. It provides various methods, like querying for an iterable of all the items on some specific page. Under the hood, it delegates to a respective `Store` (for example, if I am asking for an image, it will query the `DataStore`), and the stores know how to talk to some backend to actually retrieve the data. The querying of the backend is done by the `Store`, not the `Reader`.

The reader has the following operations:

- Get a lazy iterator of all items on some page given the page number. This function returns `ItemPrimitive` instances -- items that are allowed to exist on a page, like vectors or image containers.
- Dereference a container. You provide the container primitive, which is a kind of `ItemPrimitive`, and in response you get a `DataStorePrimitive` which is automatically recursed and directly returned.
- Various getters for the headers that we already read.
