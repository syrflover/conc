use std::{
    pin::Pin,
    task::{Context, Poll},
};

use futures::Stream;

use super::{OrderedTask, Task};

impl<T> Stream for OrderedTask<T>
where
    T: Send + Sync + Unpin + 'static,
{
    type Item = T;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if self.is_empty() {
            return Poll::Ready(None);
        }

        let this = Pin::get_mut(self);

        this.futs.update_cond(|t, _| t.is_empty());

        while let Some(Task { id, item: mut fut }) = this.futs.get_mut_false().pop() {
            match fut.as_mut().poll(cx) {
                Poll::Ready(Some(ret)) => {
                    // consume task
                    let _ = this.rx.poll_recv(cx);

                    this.rets.push(Task::new(id, ret));

                    cx.waker().wake_by_ref();
                }
                // returns None if closed channel
                Poll::Ready(None) => {
                    return Poll::Ready(None);
                }
                Poll::Pending => {
                    this.futs.get_mut_true().push(Task::new(id, fut));
                }
            }
        }

        if let Some(Task { id, .. }) = this.rets.peek() {
            if id == &this.ret_pos {
                this.ret_pos += 1;

                let r = this.rets.pop().unwrap();

                Poll::Ready(Some(r.item))
            } else {
                Poll::Pending
            }
        } else {
            Poll::Pending
        }
    }
}
