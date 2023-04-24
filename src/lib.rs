mod cond_pair;
mod ordered;
mod unordered;

pub use cond_pair::CondPair;
pub use ordered::{OrderedTask, OrderedTaskAdapter};
pub use unordered::{unordered_task, UnorderedTask};

pub type BoxFuture<T> = Box<dyn std::future::Future<Output = T> + Send + Sync + 'static>;
