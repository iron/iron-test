#![deny(missing_docs)]
#![deny(warnings)]

//! A set of convenience methods and constructors for making requests to Iron Handlers.

extern crate iron;
extern crate hyper;
extern crate url;
extern crate uuid;
extern crate rand;

#[macro_use]
extern crate log;

pub use project_builder::ProjectBuilder;

/// Set of convenience methods for making requests to Iron Handlers.
pub mod request;

/// Set of helper methods for interacting with Iron Responses.
pub mod response;

/// Tooling for mocking a Stream.
pub mod mock_stream;

mod project_builder;
