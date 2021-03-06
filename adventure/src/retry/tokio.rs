use core::pin::Pin;
use core::task::{Context, Poll};
use std::time::{Duration, Instant};

use futures_util::compat::Compat01As03 as Compat;
use tokio_timer::Delay as DelayImpl;

use super::{RetryError, Timer};
use crate::response::Response;

/// Provides a delayed response using [`tokio_timer`] crate.
#[derive(Clone, Default)]
pub struct TokioTimer;

/// A response that completes at a specified instant in time.
#[must_use = "responses do nothing unless polled"]
pub struct Delay {
    inner: Compat<DelayImpl>,
}

impl Timer for TokioTimer {
    type Delay = Delay;

    fn expires_in(&mut self, duration: Duration) -> Self::Delay {
        let deadline = Instant::now() + duration;
        let delay = DelayImpl::new(deadline);
        Delay {
            inner: Compat::new(delay),
        }
    }
}

impl Response for Delay {
    type Ok = ();
    type Error = RetryError;

    fn try_poll(
        mut self: Pin<&mut Self>,
        ctx: &mut Context<'_>,
    ) -> Poll<Result<Self::Ok, Self::Error>> {
        let r = match Response::try_poll(Pin::new(&mut self.inner), ctx) {
            Poll::Pending => {
                return Poll::Pending;
            }
            Poll::Ready(Err(ref e)) if e.is_shutdown() => Err(RetryError::shutdown()),
            Poll::Ready(_) => Ok(()),
        };
        Poll::Ready(r)
    }
}
