use core::ops::{Deref, DerefMut};
use core::pin::Pin;
use core::task::{Context, Poll};

use futures_core::{FusedStream, Stream};
use pin_utils::unsafe_pinned;

use crate::request::{BaseRequest, Request};
use crate::response::Response;

/// A request able to send subsequent requests to enumerate the entire result.
pub trait PagedRequest: BaseRequest {
    /// Modify itself to retrive the next response, of return `false` if the
    /// given response was the last one.
    fn advance(&mut self, response: &Self::Ok) -> bool;

    fn paginate<C>(self, client: C) -> Paginator<C, Self>
    where
        Self: Request<C> + Sized,
    {
        Paginator::new(client, self)
    }
}

impl<P> PagedRequest for Pin<P>
where
    P: DerefMut,
    <P as Deref>::Target: PagedRequest + Unpin,
{
    fn advance(&mut self, response: &Self::Ok) -> bool {
        self.as_mut().get_mut().advance(response)
    }
}

/// A stream over the pages that consists the entire set from the request.
pub struct Paginator<C, R>
where
    R: PagedRequest + Request<C>,
{
    client: C,
    request: Option<R>,
    next: Option<R::Response>,
}

impl<C, R> Paginator<C, R>
where
    R: PagedRequest + Request<C>,
{
    unsafe_pinned!(request: Option<R>);

    unsafe_pinned!(next: Option<R::Response>);

    pub fn new(client: C, request: R) -> Self {
        Paginator {
            client,
            request: Some(request),
            next: None,
        }
    }
}

impl<C, R> Unpin for Paginator<C, R>
where
    C: Unpin,
    R: PagedRequest + Request<C> + Unpin,
{
}

impl<C, R> Paginator<C, R>
where
    C: Clone,
    R: PagedRequest + Request<C> + Unpin,
{
    fn poll_next(
        mut self: Pin<&mut Self>,
        ctx: &mut Context<'_>,
    ) -> Poll<Option<Result<R::Ok, R::Error>>> {
        if self.as_mut().next().is_none() {
            let client = self.client.clone();
            if let Some(request) = self.as_mut().request().as_pin_mut() {
                let next = request.send(client);
                self.as_mut().next().set(Some(next));
            } else {
                return Poll::Ready(None);
            }
        };

        assert!(self.as_mut().next().is_some());
        assert!(self.as_mut().request().is_some());

        let page = match self.as_mut().next().as_pin_mut().unwrap().try_poll(ctx) {
            Poll::Pending => return Poll::Pending,
            Poll::Ready(Ok(x)) => x,
            Poll::Ready(Err(e)) => {
                self.as_mut().next().set(None);
                return Poll::Ready(Some(Err(e.into())));
            }
        };
        self.as_mut().next().set(None);

        let advanced = if let Some(mut r) = self.as_mut().request().as_pin_mut() {
            r.advance(&page)
        } else {
            true
        };
        if !advanced {
            self.as_mut().request().set(None);
        }

        Poll::Ready(Some(Ok(page)))
    }
}

impl<C, R> Stream for Paginator<C, R>
where
    C: Clone,
    R: PagedRequest + Request<C> + Unpin,
{
    type Item = Result<R::Ok, R::Error>;

    fn poll_next(self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Paginator::poll_next(self, ctx)
    }
}

impl<C, R> FusedStream for Paginator<C, R>
where
    C: Clone,
    R: PagedRequest + Request<C> + Unpin,
{
    fn is_terminated(&self) -> bool {
        self.next.is_none()
    }
}

#[cfg(feature = "alloc")]
mod feature_alloc {
    use alloc::boxed::Box;

    use super::*;

    impl<R> PagedRequest for Box<R>
    where
        R: PagedRequest,
    {
        fn advance(&mut self, response: &Self::Ok) -> bool {
            (**self).advance(response)
        }
    }
}
