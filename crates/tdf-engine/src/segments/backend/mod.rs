

pub trait BackendPointer<T, U>: Ord {}

pub trait Backend<Pointer: BackendPointer<T, U>> {
    fn get_item(pointer: Pointer) -> BackendItem
}
