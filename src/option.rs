use core::future::Future;

use crate::foo::{Foo, Id, MapSome};

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
    fn async_map<TFn, TFuture>(
        self,
        f: TFn,
    ) -> Foo<TFn, T, TFuture, MapSome<TFuture::Output>, Option<TFuture::Output>>
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
    fn async_and_then<U, TFn, TFuture>(
        self,
        f: TFn,
    ) -> Foo<TFn, T, TFuture, Id<TFuture::Output>, TFuture::Output>
    where
        TFn: FnOnce(T) -> TFuture,
        TFuture: Future<Output = Option<U>>;
}

impl<T> AsyncMapExt<T> for Option<T> {
    fn async_map<TFn, TFuture>(
        self,
        f: TFn,
    ) -> Foo<TFn, T, TFuture, MapSome<TFuture::Output>, Option<TFuture::Output>>
    where
        TFn: FnOnce(T) -> TFuture,
        TFuture: Future,
    {
        match self {
            Some(v) => Foo::new(f, v),
            None => Foo::no_action(None),
        }
    }

    fn async_and_then<U, TFn, TFuture>(
        self,
        f: TFn,
    ) -> Foo<TFn, T, TFuture, Id<TFuture::Output>, TFuture::Output>
    where
        TFn: FnOnce(T) -> TFuture,
        TFuture: Future<Output = Option<U>>,
    {
        match self {
            Some(v) => Foo::new(f, v),
            None => Foo::no_action(None),
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
