use core::future::Future;

use crate::foo::{Foo, Id, MapOk};

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
    fn async_map<TFn, TFuture>(
        self,
        f: TFn,
    ) -> Foo<TFn, T, TFuture, MapOk<TFuture::Output, E>, Result<TFuture::Output, E>>
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
    fn async_and_then<U, TFn, TFuture>(
        self,
        f: TFn,
    ) -> Foo<TFn, T, TFuture, Id<TFuture::Output>, Result<U, E>>
    where
        TFn: FnOnce(T) -> TFuture,
        TFuture: Future<Output = Result<U, E>>;
}

impl<T, E> AsyncMapExt<T, E> for Result<T, E> {
    fn async_map<TFn, TFuture>(
        self,
        f: TFn,
    ) -> Foo<TFn, T, TFuture, MapOk<TFuture::Output, E>, Result<TFuture::Output, E>>
    where
        TFn: FnOnce(T) -> TFuture,
        TFuture: Future,
    {
        match self {
            Ok(v) => Foo::new(f, v),
            Err(e) => Foo::no_action(Err(e)),
        }
    }

    fn async_and_then<U, TFn, TFuture>(
        self,
        f: TFn,
    ) -> Foo<TFn, T, TFuture, Id<TFuture::Output>, Result<U, E>>
    where
        TFn: FnOnce(T) -> TFuture,
        TFuture: Future<Output = Result<U, E>>,
    {
        match self {
            Ok(v) => Foo::new(f, v),
            Err(e) => Foo::no_action(Err(e)),
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
