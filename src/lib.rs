#![allow(unstable)]
#![deny(missing_docs)]
#![deny(warnings)]

//! A set of constructors for mocking Iron objects.

extern crate iron;
extern crate hyper;
extern crate url;
extern crate uuid;
extern crate intovec;

#[macro_use]
extern crate log;

pub use project_builder::ProjectBuilder;

mod project_builder;

/// Contains tooling for mocking various Iron objects.
pub mod mock {
    /// Contains constructors for mocking Iron Requests.
    pub mod request {
        use intovec::IntoVec;
        use iron::{Request, TypeMap, Url};
        use hyper::method;
        use hyper::header::Headers;

        use std::io::net::ip::ToSocketAddr;

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
        pub fn at_with<I: IntoVec<u8>>(method: method::Method, path: Url, body: I) -> Request {
            Request {
                url: path,
                body: body.into_vec(),
                method: method,
                remote_addr: "127.0.0.1:3000".to_socket_addr().unwrap(),
                headers: Headers::new(),
                extensions: TypeMap::new()
            }
        }
    }
}

#[cfg(test)]
mod test {
    mod request {
        use super::super::mock::request;
        use iron::method;
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

