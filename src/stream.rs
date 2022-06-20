use crate::async_util::{ready, OptionPinned};
use futures_core::Stream;
use pin_project::pin_project;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

pub trait AsyncMapExt<T>: Sized {
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

#[pin_project]
pub struct AsyncMap<TStream, TFn, TFuture> {
    #[pin]
    stream: TStream,
    #[pin]
    mapper_fut: OptionPinned<TFuture>,

    mapper: TFn,
}

impl<TStream, TFn, TFuture> AsyncMap<TStream, TFn, TFuture> {
    fn new(stream: TStream, f: TFn) -> Self {
        Self {
            stream,
            mapper_fut: OptionPinned::None,
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
        ctx: &mut Context<'_>,
    ) -> Poll<Option<<Self as Stream>::Item>> {
        let mut me = self.project();

        if me.mapper_fut.is_none() {
            let item = match ready!(me.stream.poll_next(ctx)) {
                Some(item) => item,
                None => return Poll::Ready(None),
            };

            let fut = (me.mapper)(item);
            me.mapper_fut.set(OptionPinned::Some(fut));
        }

        let fut = me.mapper_fut.as_mut().project().unwrap();
        let output = ready!(fut.poll(ctx));

        me.mapper_fut.set(OptionPinned::None);

        Poll::Ready(Some(output))
    }
}

#[cfg(test)]
#[tokio::test]
async fn test() {
    use tokio_stream::StreamExt;
    assert_eq!(
        vec![2, 3],
        tokio_stream::iter(vec![1, 2])
            .async_map(|x| async move { x + 1 })
            .collect::<Vec<_>>()
            .await,
    );
}
