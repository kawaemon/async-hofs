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

    /// Basically same as [`Option::and_then`], but it accepts closure that returns [`Future`]
    ///
    /// [`Option::and_then`]: core::option::Option::and_then
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
    ///     Some(1)
    ///         .async_and_then(|x: i32| async move { Some(x + 1) })
    ///         .await,
    ///     Some(2),
    /// );
    ///
    /// assert_eq!(
    ///     Some(1)
    ///         .async_and_then(|x: i32| async move { Option::<i32>::None })
    ///         .await,
    ///     None
    /// );
    ///
    /// assert_eq!(
    ///     None.async_and_then(|x: i32| async move { Some(x + 1) })
    ///         .await,
    ///     None
    /// );
    /// # }
    /// ```
    fn async_and_then<U, TFn, TFuture>(self, f: TFn) -> AsyncAndThen<T, TFn, TFuture>
    where
        TFn: FnOnce(T) -> TFuture,
        TFuture: Future<Output = Option<U>>;
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

#[doc(hidden)]
#[pin_project(project = AsyncAndThenProj)]
pub enum AsyncAndThen<T, TFn, TFuture> {
    None,
    Pending(Option<(T, TFn)>),
    Polling(#[pin] TFuture),
}

impl<T, U, TFn, TFuture> Future for AsyncAndThen<T, TFn, TFuture>
where
    TFn: FnOnce(T) -> TFuture,
    TFuture: Future<Output = Option<U>>,
{
    type Output = Option<U>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        use AsyncAndThenProj::*;

        match self.as_mut().project() {
            None => Poll::Ready(Option::None),

            Pending(payload) => {
                let (x, f) = payload.take().expect("AsyncMap::Pending polled twice");
                let future = f(x);
                self.set(AsyncAndThen::Polling(future));
                self.poll(cx)
            }

            Polling(future) => match future.poll(cx) {
                Poll::Ready(Option::Some(d)) => Poll::Ready(Option::Some(d)),
                Poll::Ready(Option::None) => Poll::Ready(Option::None),
                Poll::Pending => Poll::Pending,
            },
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

    fn async_and_then<U, TFn, TFuture>(self, f: TFn) -> AsyncAndThen<T, TFn, TFuture>
    where
        TFn: FnOnce(T) -> TFuture,
        TFuture: Future<Output = Option<U>>,
    {
        match self {
            Some(v) => AsyncAndThen::Pending(Some((v, f))),
            None => AsyncAndThen::None,
        }
    }
}

#[cfg(test)]
mod test {
    use super::AsyncMapExt;

    #[tokio::test]
    async fn map() {
        assert_eq!(
            Some(1).async_map(|x: i32| async move { x + 1 }).await,
            Some(2),
        );

        assert_eq!(None.async_map(|x: i32| async move { x + 1 }).await, None);
    }

    #[tokio::test]
    async fn and_then() {
        assert_eq!(
            Some(1)
                .async_and_then(|x: i32| async move { Some(x + 1) })
                .await,
            Some(2),
        );

        assert_eq!(
            Some(1)
                .async_and_then(|_: i32| async move { Option::<i32>::None })
                .await,
            None
        );

        assert_eq!(
            None.async_and_then(|x: i32| async move { Some(x + 1) })
                .await,
            None
        );
    }
}
