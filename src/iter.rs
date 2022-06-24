use crate::async_util::{ready, OptionPinned};
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::Stream;
use pin_project::pin_project;

pub trait AsyncMapExt<T>: Sized {
    /// Basically same as [`Iterator::map`], but it accepts closure that returns
    /// [`Future`] and creates new [`Stream`] instead of [`Iterator`].
    ///
    /// [`Iterator`]: core::iter::Iterator
    /// [`Iterator::map`]: core::iter::Iterator::map
    /// [`Future`]: core::future::Future
    /// [`Stream`]: futures_core::Stream
    ///
    /// # Examples
    ///
    /// ```
    /// # #[tokio::main]
    /// # async fn main() {
    /// use async_hofs::prelude::*;
    /// use tokio_stream::StreamExt; // for .collect
    ///
    /// assert_eq!(
    ///     vec![1, 2]
    ///         .into_iter()
    ///         .async_map(|x| async move { x + 1 })
    ///         .collect::<Vec<_>>()
    ///         .await,
    ///     vec![2, 3],
    /// );
    /// # }
    /// ```
    fn async_map<TFn, TFuture, U>(self, f: TFn) -> AsyncMap<Self, TFn, TFuture>
    where
        TFn: FnMut(T) -> TFuture,
        TFuture: Future<Output = U>;
}

impl<TIter, T> AsyncMapExt<T> for TIter
where
    TIter: Iterator<Item = T>,
{
    fn async_map<TFn, TFuture, U>(self, f: TFn) -> AsyncMap<Self, TFn, TFuture>
    where
        TFn: FnMut(T) -> TFuture,
        TFuture: Future<Output = U>,
    {
        AsyncMap::new(self, f)
    }
}

#[doc(hidden)]
#[pin_project]
pub struct AsyncMap<TIter, TFn, TFuture> {
    #[pin]
    mapper_future: OptionPinned<TFuture>,
    mapper: TFn,
    iter: TIter,
}

impl<TIter, TFn, TFuture> AsyncMap<TIter, TFn, TFuture> {
    fn new(iter: TIter, f: TFn) -> Self {
        Self {
            mapper_future: OptionPinned::None,
            mapper: f,
            iter,
        }
    }
}

impl<TIter, TFn, T, U, TFuture> Stream for AsyncMap<TIter, TFn, TFuture>
where
    TFn: FnMut(T) -> TFuture,
    TIter: Iterator<Item = T>,
    TFuture: Future<Output = U>,
{
    type Item = U;

    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<<Self as Stream>::Item>> {
        let mut me = self.project();

        if me.mapper_future.is_none() {
            let item = match me.iter.next() {
                Some(x) => x,
                None => return Poll::Ready(None),
            };

            let future = (me.mapper)(item);
            me.mapper_future.set(OptionPinned::Some(future));
        }

        let future = me.mapper_future.as_mut().project().unwrap();
        let output = ready!(future.poll(cx));

        me.mapper_future.set(OptionPinned::None);

        Poll::Ready(Some(output))
    }
}

#[cfg(test)]
#[tokio::test]
async fn test() {
    use tokio_stream::StreamExt;

    assert_eq!(
        vec![1, 2]
            .into_iter()
            .async_map(|x| async move { x + 1 })
            .collect::<Vec<_>>()
            .await,
        vec![2, 3],
    );
}
