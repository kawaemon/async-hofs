use core::future::Future;
use core::marker::PhantomData;
use core::pin::Pin;
use core::task::Context;
use core::task::Poll;

use pin_project::pin_project;

use crate::async_util::ready;

pub(crate) trait PollMapper {
    type In;
    type Out;
    fn map(i: Self::In) -> Self::Out;
}

#[pin_project(project = StateProj)]
enum State<TFn, TFnArg, TFuture, TOutput> {
    NoAction(Option<TOutput>),
    Pending(Option<(TFn, TFnArg)>),
    Polling(#[pin] TFuture),
}

#[pin_project]
pub struct Foo<TFn, TFnArg, TFuture, TPollMapper, TOutput> {
    #[pin]
    state: State<TFn, TFnArg, TFuture, TOutput>,
    _poll_mapper: PhantomData<fn() -> TPollMapper>,
}

impl<TFn, TFnArg, TFuture, TPollMapper, TOutput> Foo<TFn, TFnArg, TFuture, TPollMapper, TOutput> {
    pub(crate) fn no_action(v: TOutput) -> Self {
        Self {
            state: State::NoAction(Some(v)),
            _poll_mapper: PhantomData,
        }
    }

    pub(crate) fn new(f: TFn, x: TFnArg) -> Self {
        Self {
            state: State::Pending(Some((f, x))),
            _poll_mapper: PhantomData,
        }
    }
}

impl<TFn, TFnArg, TFuture, TPollMapper, TOutput> Future
    for Foo<TFn, TFnArg, TFuture, TPollMapper, TOutput>
where
    TFn: FnOnce(TFnArg) -> TFuture,
    TFuture: Future,
    TPollMapper: PollMapper<In = TFuture::Output, Out = TOutput>,
{
    type Output = TOutput;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        use StateProj::*;

        let mut state = self.project().state;

        match state.as_mut().project() {
            NoAction(v) => return Poll::Ready(v.take().expect("State::NoAction polled twice")),

            Pending(payload) => {
                let (f, x) = payload.take().expect("State::Pending polled twice");
                let future = f(x);
                state.set(State::Polling(future));
            }

            _ => {}
        }

        if let Polling(future) = state.project() {
            let output = ready!(future.poll(cx));
            Poll::Ready(TPollMapper::map(output))
        } else {
            unreachable!()
        }
    }
}

pub struct MapSome<T>(PhantomData<fn() -> T>);

impl<T> PollMapper for MapSome<T> {
    type In = T;
    type Out = Option<T>;

    #[inline(always)]
    fn map(i: Self::In) -> Self::Out {
        Some(i)
    }
}

pub struct MapOk<T, E>(PhantomData<fn() -> (T, E)>);

impl<T, E> PollMapper for MapOk<T, E> {
    type In = T;
    type Out = Result<T, E>;

    #[inline(always)]
    fn map(i: Self::In) -> Self::Out {
        Ok(i)
    }
}

pub struct Id<T>(PhantomData<fn() -> T>);

impl<T> PollMapper for Id<T> {
    type In = T;
    type Out = T;

    #[inline(always)]
    fn map(i: Self::In) -> Self::Out {
        i
    }
}
