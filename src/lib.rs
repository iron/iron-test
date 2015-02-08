#![deny(missing_docs)]
#![deny(warnings)]
#![feature(core, io, env, path)]

//! A set of constructors for mocking Iron objects.

extern crate iron;
extern crate hyper;
extern crate url;
extern crate uuid;

#[macro_use]
extern crate log;

pub use project_builder::ProjectBuilder;

mod project_builder;

/// Contains tooling for mocking various Iron objects.
pub mod mock {
    /// Contains constructors for mocking Iron Requests.
    pub mod request {
        use iron::{Request, TypeMap, Headers, Url};
        use iron::request::Body;
        use iron::{method, headers};

        use hyper::http::HttpReader;

        use std::old_io::net::ip::ToSocketAddr;

        /// Create a new mock Request with the given method, url, and data.
        pub fn new<'a, R>(method: method::Method, path: Url,
                          data: &'a mut R) -> Request<'a>
        where R: Reader {
            let reader = HttpReader::EofReader(data as &'a mut Reader);
            let addr = "127.0.0.1:3000".to_socket_addr().unwrap();

            let mut headers = Headers::new();
            let host = Url::parse("http://127.0.0.1:3000").unwrap()
                .into_generic_url()
                .serialize_host().unwrap();

            headers.set(headers::Host {
                hostname: host,
                port: Some(3000),
            });

            headers.set(headers::UserAgent("iron-test".to_string()));

            Request {
                method: method,
                url: path,
                body: Body::new(reader),
                local_addr: addr.clone(),
                remote_addr: addr,
                headers: headers,
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

