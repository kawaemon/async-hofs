use core::pin::Pin;
use pin_project::pin_project;

macro_rules! ready {
    ($poll: expr) => {
        match $poll {
            core::task::Poll::Ready(r) => r,
            core::task::Poll::Pending => return core::task::Poll::Pending,
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
