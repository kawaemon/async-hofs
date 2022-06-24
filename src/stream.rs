use crate::async_util::{ready, OptionPinned};
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::Stream;
use pin_project::pin_project;

pub trait AsyncMapExt<T>: Sized {
    /// Basically same as [`StreamExt::map`], but it accepts closure that returns
    /// [`Future`] and creates new [`Stream`]
    ///
    /// [`Future`]: core::future::Future
    /// [`Stream`]: futures_core::Stream
    /// [`StreamExt::map`]: https://docs.rs/tokio-stream/0.1.9/tokio_stream/trait.StreamExt.html#method.map
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
    ///     tokio_stream::iter(vec![1, 2])
    ///         .async_map(|x| async move { x + 1 })
    ///         .collect::<Vec<_>>()
    ///         .await,
    ///     vec![2, 3],
    /// );
    /// # }
    fn async_map<TFn, TFuture, U>(self, f: TFn) -> AsyncMap<Self, TFn, TFuture>
    where
        TFn: FnMut(T) -> TFuture,
        TFuture: Future<Output = U>;
}

impl<TStream, T> AsyncMapExt<T> for TStream
where
    TStream: Stream<Item = T>,
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
pub struct AsyncMap<TStream, TFn, TFuture> {
    #[pin]
    stream: TStream,
    #[pin]
    mapper_future: OptionPinned<TFuture>,

    mapper: TFn,
}

impl<TStream, TFn, TFuture> AsyncMap<TStream, TFn, TFuture> {
    fn new(stream: TStream, f: TFn) -> Self {
        Self {
            stream,
            mapper_future: OptionPinned::None,
            mapper: f,
        }
    }
}

impl<TStream, TFn, T, U, TFuture> Stream for AsyncMap<TStream, TFn, TFuture>
where
    TFn: FnMut(T) -> TFuture,
    TStream: Stream<Item = T>,
    TFuture: Future<Output = U>,
{
    type Item = U;

    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<<Self as Stream>::Item>> {
        let mut me = self.project();

        if me.mapper_future.is_none() {
            let item = match ready!(me.stream.poll_next(cx)) {
                Some(item) => item,
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
        tokio_stream::iter(vec![1, 2])
            .async_map(|x| async move { x + 1 })
            .collect::<Vec<_>>()
            .await,
        vec![2, 3],
    );
}
