#![license = "MIT"]
#![deny(missing_docs)]
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
        use std::path::BytesContainer;
        use iron::{Request, TypeMap, Url};
        use http::method;
        use http::headers::request::HeaderCollection;

        /// Create a new request at `/` on the specified host with the
        /// specified method.
        pub fn new<S: Str>(method: method::Method, host: S) -> Request {
            let mut url_str = "http://".to_string();
            url_str.push_str(host.as_slice());
            let url = Url::parse(url_str.as_slice()).unwrap();
            at(method, url)
        }

        /// Create a new request at the specific Url with the specified method.
        pub fn at(method: method::Method, path: Url) -> Request {
            at_with(method, path, "")
        }

        /// Create a new request at the specified Url with the specified method
        /// and the specified content as the body of the request.
        pub fn at_with<B: BytesContainer>(method: method::Method, path: Url, body: B) -> Request {
            Request {
                url: path,
                body: body.container_as_bytes().to_vec(),
                method: method,
                remote_addr: None,
                headers: HeaderCollection::new(),
                extensions: TypeMap::new()
            }
        }
    }

    /// Contains constructors for mocking Iron Responses.
    #[deprecated = "Replaced by Response modifiers and constructors in Iron itself."]
    pub mod response {
        use iron::{Response, status};

        /// Create a new, blank, response.
        pub fn new() -> Response {
            panic!("Use iron::Response::new() instead.");
        }

        /// Create a new response with the specified body and status.
        pub fn with<B>(_: status::Status, _: B) -> Response {
            panic!("Use iron::Response::new() and modifiers instead.");
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
            assert_eq!(req.body.as_slice(), b"Hello Google!");
        }
    }
}

