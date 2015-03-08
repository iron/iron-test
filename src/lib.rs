#![deny(missing_docs)]
#![deny(warnings)]
#![feature(core, io, path, std_misc, net, path_ext)]

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

        use std::io::Read;
        use std::net::SocketAddr;

        /// Create a new mock Request with the given method, url, and data.
        pub fn new<'a, R>(method: method::Method, path: Url,
                          data: &'a mut R) -> Request<'a>
        where R: Read {
            let reader = HttpReader::EofReader(data as &'a mut Read);
            let addr: SocketAddr = "127.0.0.1:3000".parse().unwrap();

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
        use std::io::{Read,Cursor};

        #[test] fn test_request() {
            let ref mut data = Cursor::new("Hello Google!".as_bytes());
            let mut req = request::new(method::Get, Url::parse("http://localhost:3000").unwrap(), data);
            assert_eq!(req.method, method::Get);
            assert_eq!(format!("{}", req.url).as_slice(), "http://localhost:3000/");

            let mut body_buf = Vec::new();
            req.body.read_to_end(&mut body_buf).ok().unwrap();
            assert_eq!(&*body_buf, b"Hello Google!");
        }
    }
}

