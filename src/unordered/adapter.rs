use std::future::Future;

use super::UnorderedTask;

pub trait UnorderedTaskAdapter<T, F>
where
    T: Send + Sync + Unpin + 'static,
    F: Future<Output = T> + Send + Sync + 'static,
    Self: Iterator<Item = F> + Sized,
{
    fn unordered_task(self, limit: usize) -> UnorderedTask<T> {
        UnorderedTask::from_iter(self, limit)
    }
}

impl<I, F, T> UnorderedTaskAdapter<T, F> for I
where
    I: Iterator<Item = F>,
    F: Future<Output = T> + Send + Sync + 'static,
    T: Send + Sync + Unpin + 'static,
{
}
