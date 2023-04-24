use std::future::Future;

use super::OrderedTask;

pub trait OrderedTaskAdapter<T, F>
where
    T: Send + Sync + Unpin + 'static,
    F: Future<Output = T> + Send + Sync + 'static,
    Self: Iterator<Item = F> + Sized,
{
    fn ordered_task(self, limit: usize) -> OrderedTask<T> {
        OrderedTask::from_iter(self, limit)
    }
}

impl<I, F, T> OrderedTaskAdapter<T, F> for I
where
    I: Iterator<Item = F>,
    F: Future<Output = T> + Send + Sync + 'static,
    T: Send + Sync + Unpin + 'static,
{
}
