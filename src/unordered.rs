mod adapter;
mod future;
mod stream;
mod task;

pub use adapter::UnorderedTaskAdapter;

use std::{future::Future, pin::Pin};

use tokio::sync::mpsc;

use crate::{cond_pair::CondPair, BoxFuture};

use task::Task;

pub fn unordered_task<T>(limit: usize) -> UnorderedTask<T> {
    UnorderedTask::new(limit)
}

pub struct UnorderedTask<T> {
    futs: CondPair<Vec<Task<Pin<BoxFuture<Option<T>>>>>>,

    tx: mpsc::Sender<()>,
    rx: mpsc::Receiver<()>,
}

impl<T> UnorderedTask<T> {
    pub fn new(limit: usize) -> Self {
        let (tx, rx) = mpsc::channel(limit);

        Self {
            futs: CondPair::new(Vec::new(), Vec::new()),
            tx,
            rx,
        }
    }

    pub fn from_iter<I, F>(iter: I, limit: usize) -> Self
    where
        T: Send + Sync + Unpin + 'static,
        F: Future<Output = T> + Send + Sync + 'static,
        I: Iterator<Item = F>,
    {
        let mut this = Self::new(limit);

        for fut in iter {
            this.push(fut);
        }

        this
    }
}

impl<T> UnorderedTask<T>
where
    T: Send + Sync + Unpin + 'static,
{
    pub fn push<F>(&mut self, fut: F)
    where
        F: Future<Output = T> + Send + Sync + 'static,
    {
        let tx = self.tx.clone();
        let f = async move {
            // produce task
            if tx.send(()).await.is_ok() {
                Some(fut.await)
            } else {
                None
            }
        };

        self.futs.get_mut_false().push(Task::new(Box::pin(f)));
    }

    pub async fn pop(&mut self) -> Option<T> {
        self.await
    }

    pub fn len(&self) -> usize {
        let (t, f) = self.futs.get();
        t.len() + f.len()
    }

    pub fn is_empty(&self) -> bool {
        let (t, f) = self.futs.get();
        t.is_empty() && f.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use futures::StreamExt;
    use rand::{seq::SliceRandom, thread_rng, Rng};

    use super::*;

    async fn sleep(ret: usize, duration: Duration) -> usize {
        tokio::time::sleep(duration).await;
        ret
    }

    #[tokio::test]
    async fn base() {
        let mut rng = thread_rng();

        let mut xs = (0..300).collect::<Vec<_>>();

        xs.shuffle(&mut rng);

        let iter = xs
            .iter()
            .map(|x| sleep(*x, Duration::from_millis(rng.gen_range(0..10))));

        let start = Instant::now();

        let mut actual = iter.unordered_task(15).collect::<Vec<_>>().await;

        let end = start.elapsed().as_millis();

        println!("{}ms", end);

        let expected = (0..xs.len()).collect::<Vec<_>>();

        assert_ne!(actual, expected);

        actual.sort();

        assert_eq!(actual, expected);
    }

    #[tokio::test]
    async fn empty() {
        let mut rng = thread_rng();

        let mut tasker = unordered_task(15);

        let xs = (0..50).collect::<Vec<usize>>();

        for x in xs
            .iter()
            .map(|x| sleep(*x, Duration::from_millis(rng.gen_range(0..10))))
        {
            tasker.push(x);
        }

        let mut received = Vec::new();

        for _ in 0..xs.len() {
            let x = tasker.pop().await.unwrap();

            received.push(x);
        }

        assert_eq!(received.len(), xs.len());

        let r = tasker.pop().await;

        assert!(r.is_none());
    }
}
