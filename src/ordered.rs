mod adapter;
mod future;
mod stream;
mod task;

pub use adapter::OrderedTaskAdapter;

use std::{collections::BinaryHeap, future::Future, pin::Pin};

use tokio::sync::mpsc;

use crate::{cond_pair::CondPair, BoxFuture};

use task::Task;

pub struct OrderedTask<T> {
    futs: CondPair<BinaryHeap<Task<Pin<BoxFuture<Option<T>>>>>>,
    rets: BinaryHeap<Task<T>>,

    tx: mpsc::Sender<()>,
    rx: mpsc::Receiver<()>,

    ret_pos: usize,
}

/* fn raw_waker(ptr: *const ()) -> RawWaker {
    fn wake(_: *const ()) {
        // println!("wake {ptr:?}");
    }
    fn wake_by_ref(_: *const ()) {
        // println!("wake_by_ref {ptr:?}");
    }
    fn drop(_: *const ()) {
        // println!("drop {ptr:?}");
    }
    fn clone(ptr: *const ()) -> RawWaker {
        // println!("clone {ptr:?}");
        raw_waker(ptr)
    }

    let empty_raw_waker_vtable = &RawWakerVTable::new(clone, wake, wake_by_ref, drop);
    RawWaker::new(ptr, empty_raw_waker_vtable)
}

fn waker(ptr: *const ()) -> Waker {
    unsafe { Waker::from_raw(raw_waker(ptr)) }
} */

impl<T> OrderedTask<T> {
    pub fn new(limit: usize) -> Self {
        let (tx, rx) = mpsc::channel(limit);

        Self {
            futs: CondPair::new(BinaryHeap::new(), BinaryHeap::new()),
            rets: BinaryHeap::new(),
            tx,
            rx,
            ret_pos: 0,
        }
    }

    pub fn from_iter<I, F>(iter: I, limit: usize) -> Self
    where
        T: Send + Sync + 'static + Unpin,
        F: Future<Output = T> + Send + Sync + 'static,
        I: Iterator<Item = F>,
    {
        let mut this = Self::new(limit);

        for (i, fut) in iter.enumerate() {
            let tx = this.tx.clone();
            let f = async move {
                // produce task
                if tx.send(()).await.is_ok() {
                    Some(fut.await)
                } else {
                    None
                }
            };

            this.futs.get_mut_false().push(Task::new(i, Box::pin(f)));
        }

        this
    }

    pub fn f_len(&self) -> usize {
        let (t, f) = self.futs.get();
        t.len() + f.len()
    }

    pub fn r_len(&self) -> usize {
        self.rets.len()
    }

    pub fn is_empty(&self) -> bool {
        let (t, f) = self.futs.get();
        t.is_empty() && f.is_empty() && self.rets.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use std::{
        sync::{Arc, Mutex},
        time::{Duration, Instant},
    };

    use futures::{StreamExt, TryStreamExt};
    use rand::{thread_rng, Rng};

    use super::OrderedTaskAdapter;

    async fn sleep(ret: i32, duration: Duration) -> i32 {
        tokio::time::sleep(duration).await;
        ret
    }

    #[tokio::test]
    async fn base() {
        let mut rng = thread_rng();
        let it = (0..300).map(|x| sleep(x, Duration::from_millis(rng.gen_range(0..10))));

        let start = Instant::now();

        let r = it.ordered_task(15).collect::<Vec<_>>().await;

        let end = start.elapsed().as_millis();

        println!("{}ms", end);

        assert_eq!(r, (0..300).collect::<Vec<_>>());
    }

    #[tokio::test]
    async fn empty() {
        let it = (0..0).map(|x| sleep(x, Duration::from_millis(0)));

        let r = it.ordered_task(5).collect::<Vec<_>>().await;

        assert!(r.is_empty());
    }

    #[tokio::test]
    async fn cancel() {
        async fn error(x: i32, dur: Duration, cnt: Arc<Mutex<i32>>) -> Result<i32, ()> {
            tokio::time::sleep(dur).await;

            {
                let mut cnt = cnt.lock().unwrap();
                *cnt += 1;
            }

            if x >= 10 {
                Err(())
            } else {
                Ok(x)
            }
        }

        let a_cnt = Arc::new(Mutex::new(0));
        let b_cnt = Arc::new(Mutex::new(0));

        let mut rng = thread_rng();

        let sorted_tasks =
            (0..15).map(|x| error(x, Duration::from_millis((x as u64) * 15), a_cnt.clone()));
        let rand_tasks = (0..15).map(|x| {
            error(
                x,
                Duration::from_millis(rng.gen_range(if x >= 14 { 1000..1001 } else { 0..10 })),
                b_cnt.clone(),
            )
        });

        let a = sorted_tasks.ordered_task(5).try_collect::<Vec<_>>().await;
        let b = rand_tasks.ordered_task(5).try_collect::<Vec<_>>().await;

        println!("a.strong_count {}", Arc::strong_count(&a_cnt));
        println!("b.strong_count {}", Arc::strong_count(&b_cnt));

        let a_cnt = Arc::try_unwrap(a_cnt).unwrap().into_inner().unwrap();
        let b_cnt = Arc::try_unwrap(b_cnt).unwrap().into_inner().unwrap();

        println!("a.cnt {}", a_cnt);
        println!("b.cnt {}", b_cnt);

        assert_eq!(a, Err(()));
        assert_eq!(a_cnt, 11);
        assert_eq!(b, Err(()));
        assert!(b_cnt <= 14, "b_cnt {b_cnt}");
    }
}
