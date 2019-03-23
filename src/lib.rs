//! A general method of the common pattern for network requests.
//!
//! This crate defines a general interface of the [request-response pattern]
//! like HTTP request, and provides a number of composable constructs to work
//! with it at the high-level.
//!
//! [request-response pattern]: https://en.wikipedia.org/wiki/Request%E2%80%93response
#![cfg_attr(feature = "std-future", feature(futures_api))]
#![deny(rust_2018_idioms)]

pub mod paginator;
pub mod prelude;
pub mod repeat;
pub mod request;
pub mod response;
pub mod task;
pub mod util;

#[cfg(feature = "backoff")]
pub mod retry;

mod paginator;

#[cfg(test)]
mod test;

#[doc(inline)]
pub use crate::{
    paginator::{PagedRequest, Paginator},
    request::Request,
    response::Response,
};
