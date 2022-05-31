pub mod option {
    use async_trait::async_trait;
    use std::future::Future;

    #[async_trait]
    pub trait AsyncMapExt<T>: Send {
        async fn async_map<Fn, Fut, Out>(self, f: Fn) -> Option<Out>
        where
            Fn: Send + FnOnce(T) -> Fut,
            Fut: Future<Output = Out> + Send;
    }

    #[async_trait]
    impl<T: Send> AsyncMapExt<T> for Option<T> {
        async fn async_map<Fn, Fut, Out>(self, f: Fn) -> Option<Out>
        where
            Fn: Send + FnOnce(T) -> Fut,
            Fut: Future<Output = Out> + Send,
        {
            match self {
                Some(s) => Some(f(s).await),
                None => None,
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
        assert_eq!(None.async_map(|x: i32| async move { x + 1 }).await, None,);
    }
}

pub mod result {
    use async_trait::async_trait;
    use std::future::Future;

    #[async_trait]
    pub trait AsyncMapExt<T, E>: Send {
        async fn async_map<Fn, Fut, Out>(self, f: Fn) -> Result<Out, E>
        where
            Fn: Send + FnOnce(T) -> Fut,
            Fut: Future<Output = Out> + Send;
    }

    #[async_trait]
    impl<T: Send, E: Send> AsyncMapExt<T, E> for Result<T, E> {
        async fn async_map<Fn, Fut, Out>(self, f: Fn) -> Result<Out, E>
        where
            Fn: Send + FnOnce(T) -> Fut,
            Fut: Future<Output = Out> + Send,
        {
            match self {
                Ok(t) => Ok(f(t).await),
                Err(e) => Err(e),
            }
        }
    }

    #[cfg(test)]
    #[tokio::test]
    async fn test() {
        type R = Result<i32, ()>;
        assert_eq!(
            R::Ok(1).async_map(|x: i32| async move { x + 1 }).await,
            R::Ok(2),
        );
        assert_eq!(
            R::Err(()).async_map(|x: i32| async move { x + 1 }).await,
            R::Err(()),
        );
    }
}

mod async_util {
    use pin_project::pin_project;
    use std::pin::Pin;

    macro_rules! ready {
        ($poll: expr) => {
            match $poll {
                std::task::Poll::Ready(r) => r,
                std::task::Poll::Pending => return std::task::Poll::Pending,
            }
        };
    }

    pub(crate) use ready;

    #[pin_project(project = OptionPinnedProj)]
    pub(crate) enum OptionPinned<T> {
        Some(#[pin] T),
        None,
    }

    impl<'a, T> OptionPinnedProj<'a, T> {
        #[track_caller]
        pub(crate) fn unwrap(self) -> Pin<&'a mut T> {
            use OptionPinnedProj::*;
            match self {
                Some(t) => t,
                None => panic!("called `unwrap` on None"),
            }
        }
    }

    impl<T> OptionPinned<T> {
        pub(crate) fn is_some(&self) -> bool {
            use OptionPinned::*;
            match self {
                Some(_) => true,
                None => false,
            }
        }

        pub(crate) fn is_none(&self) -> bool {
            !self.is_some()
        }
    }
}

pub mod stream {
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
}

pub mod iter {
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

    #[pin_project]
    pub struct AsyncMap<TIter, TFn, TFuture> {
        #[pin]
        mapper_fut: OptionPinned<TFuture>,
        mapper: TFn,
        iter: TIter,
    }

    impl<TIter, TFn, TFuture> AsyncMap<TIter, TFn, TFuture> {
        fn new(iter: TIter, f: TFn) -> Self {
            Self {
                mapper_fut: OptionPinned::None,
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
            ctx: &mut Context<'_>,
        ) -> Poll<Option<<Self as Stream>::Item>> {
            let mut me = self.project();

            if me.mapper_fut.is_none() {
                let item = match me.iter.next() {
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
            vec![1, 2]
                .into_iter()
                .async_map(|x| async move { x + 1 })
                .collect::<Vec<_>>()
                .await,
        );
    }
}
