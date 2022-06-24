use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use pin_project::pin_project;

pub trait AsyncMapExt<T> {
    /// Basically same as [`Option::map`], but it accepts closure that returns [`Future`]
    ///
    /// [`Option::map`]: core::option::Option::map
    /// [`Future`]: core::future::Future
    ///
    /// # Examples
    ///
    /// ```
    /// # #[tokio::main]
    /// # async fn main() {
    /// use async_hofs::prelude::*;
    ///
    /// assert_eq!(
    ///     Some(1).async_map(|x: i32| async move { x + 1 }).await,
    ///     Some(2),
    /// );
    ///
    /// assert_eq!(
    ///     None.async_map(|x: i32| async move { x + 1 }).await,
    ///     None
    /// );
    /// # }
    /// ```
    fn async_map<TFn, TFuture>(self, f: TFn) -> AsyncMap<T, TFn, TFuture>
    where
        TFn: FnOnce(T) -> TFuture,
        TFuture: Future;
}

#[doc(hidden)]
#[pin_project(project = AsyncMapProj)]
pub enum AsyncMap<T, TFn, TFuture> {
    None,
    Pending(Option<(T, TFn)>),
    Polling(#[pin] TFuture),
}

impl<T, U, TFn, TFuture> Future for AsyncMap<T, TFn, TFuture>
where
    TFn: FnOnce(T) -> TFuture,
    TFuture: Future<Output = U>,
{
    type Output = Option<U>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        use AsyncMapProj::*;

        match self.as_mut().project() {
            None => Poll::Ready(Option::None),

            Pending(payload) => {
                let (x, f) = payload.take().expect("AsyncMap::Pending polled twice");
                let future = f(x);
                self.set(AsyncMap::Polling(future));
                self.poll(cx)
            }

            Polling(future) => future.poll(cx).map(Some),
        }
    }
}

impl<T> AsyncMapExt<T> for Option<T> {
    fn async_map<TFn, TFuture>(self, f: TFn) -> AsyncMap<T, TFn, TFuture>
    where
        TFn: FnOnce(T) -> TFuture,
        TFuture: Future,
    {
        match self {
            Some(v) => AsyncMap::Pending(Some((v, f))),
            None => AsyncMap::None,
        }
    }
}

#[cfg(test)]
#[tokio::test]
async fn test() {
    assert_eq!(
        Some(1).async_map(|x: i32| async move { x + 1 }).await,
        Some(2),
    );
    assert_eq!(None.async_map(|x: i32| async move { x + 1 }).await, None);
}
