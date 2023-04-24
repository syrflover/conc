pub struct Task<T> {
    pub(crate) item: T,
}

impl<T> Task<T> {
    pub fn new(item: T) -> Self {
        Self { item }
    }
}
