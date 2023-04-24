use std::{
    pin::Pin,
    task::{Context, Poll},
};

use futures::Stream;

use super::{Task, UnorderedTask};

impl<T> Stream for UnorderedTask<T>
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

        while let Some(Task { item: mut fut }) = this.futs.get_mut_false().pop() {
            match fut.as_mut().poll(cx) {
                Poll::Ready(Some(ret)) => {
                    // consume task
                    let _ = this.rx.poll_recv(cx);

                    return Poll::Ready(Some(ret));
                }
                // if closed channel
                Poll::Ready(None) => {
                    return Poll::Ready(None);
                }
                Poll::Pending => {
                    this.futs.get_mut_true().push(Task::new(fut));
                }
            }
        }

        Poll::Pending
    }
}
