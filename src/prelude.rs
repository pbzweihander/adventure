//! A prelude of the `adventure` for the crate which want to try with it.
//!
//! This module is intended to be included by `use adventure::prelude::*;`,
//! to access the various traits and methods mostly will be used.

pub use crate::request::{PagedRequest, RepeatableRequest, Request, RetriableRequest};
pub use crate::response::Response;
pub use crate::util::ResponseExt;
