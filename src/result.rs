use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use pin_project::pin_project;

pub trait AsyncMapExt<T, E> {
    /// Basically same as [`Result::map`], but it accepts closure that returns [`Future`]
    ///
    /// [`Result::map`]: core::result::Result::map
    /// [`Future`]: core::future::Future
    ///
    /// # Examples
    ///
    /// ```
    /// # #[tokio::main]
    /// # async fn main() {
    /// use async_hofs::prelude::*;
    ///
    /// type Result = core::result::Result<i32, i32>;
    ///
    /// assert_eq!(
    ///     Result::Ok(1).async_map(|x: i32| async move { x + 1 }).await,
    ///     Result::Ok(2),
    /// );
    /// assert_eq!(
    ///     Result::Err(4).async_map(|x: i32| async move { x + 1 }).await,
    ///     Result::Err(4),
    /// );
    /// # }
    /// ```
    fn async_map<TFn, TFuture>(self, f: TFn) -> AsyncMap<T, E, TFn, TFuture>
    where
        TFn: FnOnce(T) -> TFuture,
        TFuture: Future;
}

#[doc(hidden)]
#[pin_project(project = AsyncMapProj)]
pub enum AsyncMap<T, E, TFn, TFuture> {
    Err(Option<E>),
    Pending(Option<(T, TFn)>),
    Polling(#[pin] TFuture),
}

impl<T, U, E, TFn, TFuture> Future for AsyncMap<T, E, TFn, TFuture>
where
    TFn: FnOnce(T) -> TFuture,
    TFuture: Future<Output = U>,
{
    type Output = Result<U, E>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        use AsyncMapProj::*;

        match self.as_mut().project() {
            Err(e) => Poll::Ready(Result::Err(e.take().expect("AsyncMap::Err polled twice"))),

            Pending(payload) => {
                let (x, f) = payload.take().expect("AsyncMap::Pending polled twice");
                let future = f(x);
                self.set(AsyncMap::Polling(future));
                self.poll(cx)
            }

            Polling(future) => future.poll(cx).map(Ok),
        }
    }
}

impl<T, E> AsyncMapExt<T, E> for Result<T, E> {
    fn async_map<TFn, TFuture>(self, f: TFn) -> AsyncMap<T, E, TFn, TFuture>
    where
        TFn: FnOnce(T) -> TFuture,
        TFuture: Future,
    {
        match self {
            Ok(v) => AsyncMap::Pending(Some((v, f))),
            Err(e) => AsyncMap::Err(Some(e)),
        }
    }
}

#[cfg(test)]
#[tokio::test]
async fn test() {
    type Result = core::result::Result<i32, i32>;
    assert_eq!(
        Result::Ok(1).async_map(|x: i32| async move { x + 1 }).await,
        Result::Ok(2),
    );
    assert_eq!(
        Result::Err(4)
            .async_map(|x: i32| async move { x + 1 })
            .await,
        Result::Err(4),
    );
}
