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
