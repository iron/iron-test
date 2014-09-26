#![license = "MIT"]
#![deny(missing_doc)]
#![deny(warnings)]
#![feature(phase)]

//! A set of constructors for mocking Iron objects.

extern crate iron;
extern crate http;
extern crate url;
extern crate uuid;

#[phase(plugin, link)]
extern crate log;

pub use project_builder::ProjectBuilder;

mod project_builder;

/// Contains tooling for mocking various Iron objects.
pub mod mock {
    /// Contains constructors for mocking Iron Requests.
    pub mod request {
        use iron::{Request, TypeMap, Url};
        use http::method;
        use http::headers::request::HeaderCollection;

        /// Create a new request at `/` on the specified host with the
        /// specified method.
        pub fn new<S: Str>(method: method::Method, host: S) -> Request {
            let url = Url::parse("http://".to_string().push_str(host.as_slice()).to_string().as_slice()).unwrap();
            at(method, url)
        }

        /// Create a new request at the specific Url with the specified method.
        pub fn at(method: method::Method, path: Url) -> Request {
            at_with(method, path, "")
        }

        /// Create a new request at the specified Url with the specified method
        /// and the specified content as the body of the request.
        pub fn at_with<S: Str>(method: method::Method, path: Url, body: S) -> Request {
            Request {
                url: path,
                body: body.as_slice().to_string(),
                method: method,
                remote_addr: None,
                headers: HeaderCollection::new(),
                extensions: TypeMap::new()
            }
        }
    }

    /// Contains constructors for mocking Iron Responses.
    pub mod response {
        use iron::{Response, TypeMap};
        use http::status;
        use http::headers::response::HeaderCollection;

        use std::path::BytesContainer;
        use std::io::MemReader;

        /// Create a new, blank, response.
        pub fn new() -> Response {
            Response {
                body: None,
                headers: HeaderCollection::new(),
                status: None,
                extensions: TypeMap::new()
            }
        }

        /// Create a new response with the specified body and status.
        pub fn with<B: BytesContainer>(status: status::Status, body: B) -> Response {
            Response {
                body: Some(box MemReader::new(body.container_as_bytes().to_vec()) as Box<Reader + Send>),
                headers: HeaderCollection::new(),
                status: Some(status),
                extensions: TypeMap::new()
            }
        }
    }
}

#[cfg(test)]
mod test {
    mod request {
        use super::super::mock::request;
        use http::method;
        use iron::Url;

        #[test] fn test_new() {
            let req = request::new(method::Get, "localhost:3000");
            assert_eq!(req.method, method::Get);
            assert_eq!(format!("{}", req.url).as_slice(), "http://localhost:3000/");
        }

        #[test] fn test_at() {
            let req = request::at(method::Post, Url::parse("http://www.google.com/").unwrap());
            assert_eq!(req.method, method::Post);
            assert_eq!(format!("{}", req.url).as_slice(), "http://www.google.com:80/");
        }

        #[test] fn test_at_with() {
            let req = request::at_with(method::Put, Url::parse("http://www.google.com/").unwrap(), "Hello Google!");
            assert_eq!(req.method, method::Put);
            assert_eq!(format!("{}", req.url).as_slice(), "http://www.google.com:80/");
            assert_eq!(req.body.as_slice(), "Hello Google!");
        }
    }

    mod response {
        use super::super::mock::response;
        use http::status;

        #[test] fn test_new() {
            let res = response::new();
            assert_eq!(res.status, None);
            assert!(res.body.is_none());
        }

        #[test] fn test_with() {
            let res = response::with(status::Ok, "Hello World!");
            assert_eq!(res.status, Some(status::Ok));
            assert_eq!(res.body.unwrap().read_to_string().unwrap().as_slice(), "Hello World!");
        }
    }
}

