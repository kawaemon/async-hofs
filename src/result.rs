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

    /// Basically same as [`Result::and_then`], but it accepts closure that returns [`Future`]
    ///
    /// [`Result::and_then`]: core::result::Result::and_then
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
    ///     Result::Ok(1)
    ///         .async_and_then(|x: i32| async move { Ok(x + 1) })
    ///         .await,
    ///     Result::Ok(2),
    /// );
    ///
    /// assert_eq!(
    ///     Result::Ok(1)
    ///         .async_and_then(|x: i32| async move { Err(x + 1) })
    ///         .await,
    ///     Result::Err(2),
    /// );
    ///
    /// assert_eq!(
    ///     Result::Err(4)
    ///         .async_and_then(|x: i32| async move { Ok(x + 1) })
    ///         .await,
    ///     Result::Err(4),
    /// );
    /// # }
    /// ```
    fn async_and_then<U, TFn, TFuture>(self, f: TFn) -> AsyncAndThen<T, E, TFn, TFuture>
    where
        TFn: FnOnce(T) -> TFuture,
        TFuture: Future<Output = Result<U, E>>;
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

#[doc(hidden)]
#[pin_project(project = AsyncAndThenProj)]
pub enum AsyncAndThen<T, E, TFn, TFuture> {
    Err(Option<E>),
    Pending(Option<(T, TFn)>),
    Polling(#[pin] TFuture),
}

impl<T, U, E, TFn, TFuture> Future for AsyncAndThen<T, E, TFn, TFuture>
where
    TFn: FnOnce(T) -> TFuture,
    TFuture: Future<Output = Result<U, E>>,
{
    type Output = Result<U, E>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        use AsyncAndThenProj::*;

        match self.as_mut().project() {
            Err(e) => Poll::Ready(Result::Err(
                e.take().expect("AsyncAndThen::Err polled twice"),
            )),

            Pending(payload) => {
                let (x, f) = payload.take().expect("AsyncAndThen::Pending polled twice");
                let future = f(x);
                self.set(AsyncAndThen::Polling(future));
                self.poll(cx)
            }

            Polling(future) => match future.poll(cx) {
                Poll::Ready(Result::Ok(v)) => Poll::Ready(Result::Ok(v)),
                Poll::Ready(Result::Err(e)) => Poll::Ready(Result::Err(e)),
                Poll::Pending => Poll::Pending,
            },
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

    fn async_and_then<U, TFn, TFuture>(self, f: TFn) -> AsyncAndThen<T, E, TFn, TFuture>
    where
        TFn: FnOnce(T) -> TFuture,
        TFuture: Future<Output = Result<U, E>>,
    {
        match self {
            Ok(v) => AsyncAndThen::Pending(Some((v, f))),
            Err(e) => AsyncAndThen::Err(Some(e)),
        }
    }
}

#[cfg(test)]
mod test {
    use super::AsyncMapExt;

    type Result = core::result::Result<i32, i32>;

    #[tokio::test]
    async fn map() {
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

    #[tokio::test]
    async fn and_then() {
        assert_eq!(
            Result::Ok(1)
                .async_and_then(|x: i32| async move { Ok(x + 1) })
                .await,
            Result::Ok(2),
        );

        assert_eq!(
            Result::Ok(1)
                .async_and_then(|x: i32| async move { Err(x + 1) })
                .await,
            Result::Err(2),
        );

        assert_eq!(
            Result::Err(4)
                .async_and_then(|x: i32| async move { Ok(x + 1) })
                .await,
            Result::Err(4),
        );
    }
}
