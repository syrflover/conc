pub struct Task<T> {
    pub(crate) id: usize,
    pub(crate) item: T,
}

impl<T> Task<T> {
    pub fn new(id: usize, item: T) -> Self {
        Self { id, item }
    }
}

impl<T> PartialEq for Task<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id.eq(&other.id)
    }
}

impl<T> Eq for Task<T> {}

impl<T> PartialOrd for Task<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        other.id.partial_cmp(&self.id)
    }
}

impl<T> Ord for Task<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.id.cmp(&self.id)
    }
}
