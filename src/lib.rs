#![deny(missing_docs)]
#![deny(warnings)]
#![feature(std_misc, path_ext)]

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
    use hyper::net::NetworkStream;
    use std::net::SocketAddr;
    use std::io::{Read, Write, Result};
    use std::any::Any;

    /// A mock network stream
    #[derive(Clone)]
    pub struct MockStream<T> {
        data: T
    }

    impl<T> MockStream<T> {
        /// Create a new mock stream that reads from the given data
        pub fn new(data: T) -> MockStream<T> {
            MockStream { data: data }
        }
    }

    impl<T: Send + Read + Write + Clone + Any> NetworkStream for MockStream<T> {
        fn peer_addr(&mut self) -> Result<SocketAddr> {
            Ok("127.0.0.1:3000".parse().unwrap())
        }
    }

    impl<T: Read> Read for MockStream<T> {
        fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
            self.data.read(buf)
        }
    }

    impl<T: Write> Write for MockStream<T> {
        fn write(&mut self, buf: &[u8]) -> Result<usize> {
            self.data.write(buf)
        }

        fn flush(&mut self) -> Result<()> {
            self.data.flush()
        }
    }

    /// Contains constructors for mocking Iron Requests.
    pub mod request {
        use iron::{Request, TypeMap, Headers, Url};
        use iron::request::Body;
        use iron::{method, headers};

        use hyper::http::HttpReader;
        use hyper::buffer::BufReader;
        use hyper::net::NetworkStream;

        use std::net::SocketAddr;

        /// Create a new mock Request with the given method, url, and data.
        pub fn new<'a, 'b>(method: method::Method, path: Url,
                           reader: &'a mut BufReader<&'b mut NetworkStream>) -> Request<'a, 'b> {
            let reader = HttpReader::EofReader(reader);
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
        use super::super::mock::MockStream;
        use iron::method;
        use iron::Url;
        use std::io::{Read, Cursor};
        use hyper::buffer::BufReader;
        use hyper::net::NetworkStream;

        #[test] fn test_request() {
            let data = Cursor::new("Hello Google!".to_string().into_bytes());
            let mut stream = MockStream::new(data);
            let mut reader = BufReader::new(&mut stream as &mut NetworkStream);
            let mut req = request::new(method::Get, Url::parse("http://localhost:3000").unwrap(),
                                       &mut reader);
            assert_eq!(req.method, method::Get);
            assert_eq!(&format!("{}", req.url)[..], "http://localhost:3000/");

            let mut body_buf = Vec::new();
            req.body.read_to_end(&mut body_buf).ok().unwrap();
            assert_eq!(&*body_buf, b"Hello Google!");
        }
    }
}
