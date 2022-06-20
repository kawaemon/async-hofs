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
