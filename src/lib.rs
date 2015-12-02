#![deny(missing_docs)]
#![deny(warnings)]

//! A set of convenience methods and constructors for making requests to Iron Handlers.

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
    use std::any::Any;
    use std::io::{Read, Write, Result};
    use std::net::SocketAddr;
    use std::time::Duration;

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

        fn set_read_timeout(&self, _: Option<Duration>) -> Result<()> {
            Ok(())
        }

        fn set_write_timeout(&self, _: Option<Duration>) -> Result<()> {
            Ok(())
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

    /// Contains convenience methods for making requests to Iron Handlers.
    pub mod request {
        use hyper::buffer::BufReader;
        use hyper::http::h1::HttpReader;
        use hyper::net::NetworkStream;

        use iron::prelude::*;
        use iron::request::Body;
        use iron::{Handler, headers, Headers, method, TypeMap, Url};

        use std::io::Cursor;

        use super::MockStream;

        /// Convenience method for making GET requests to Iron Handlers.
        pub fn get<H: Handler>(path: &str, headers: Headers, handler: H) -> IronResult<Response> {
            make_request(method::Get, path, "", headers, handler)
        }

        /// Convenience method for making POST requests with a body to Iron Handlers.
        pub fn post<H: Handler>(path: &str, headers: Headers, body: &str, handler: H) -> IronResult<Response> {
            make_request(method::Post, path, body, headers, handler)
        }

        /// Convenience method for making PATCH requests with a body to Iron Handlers.
        pub fn patch<H: Handler>(path: &str, headers: Headers, body: &str, handler: H) -> IronResult<Response> {
            make_request(method::Patch, path, body, headers, handler)
        }

        /// Convenience method for making PUT requests with a body to Iron Handlers.
        pub fn put<H: Handler>(path: &str, headers: Headers, body: &str, handler: H) -> IronResult<Response> {
            make_request(method::Put, path, body, headers, handler)
        }

        /// Convenience method for making DELETE requests to Iron Handlers.
        pub fn delete<H: Handler>(path: &str, headers: Headers, handler: H) -> IronResult<Response> {
            make_request(method::Delete, path, "", headers, handler)
        }

        /// Convenience method for making OPTIONS requests to Iron Handlers.
        pub fn options<H: Handler>(path: &str, headers: Headers, handler: H) -> IronResult<Response> {
            make_request(method::Options, path, "", headers, handler)
        }

        /// Convenience method for making HEAD requests to Iron Handlers.
        pub fn head<H: Handler>(path: &str, headers: Headers, handler: H) -> IronResult<Response> {
            make_request(method::Head, path, "", headers, handler)
        }

        fn make_request<H: Handler>(method: method::Method,
                                    path: &str,
                                    body: &str,
                                    mut headers: Headers,
                                    handler: H) -> IronResult<Response> {
            let data = Cursor::new(body.as_bytes().to_vec());
            let mut stream = MockStream::new(data);
            // Clone the stream so we can read the peer_addr off the original.
            let mut stream_clone = stream.clone();
            let mut reader = BufReader::new(&mut stream_clone as &mut NetworkStream);
            let reader = HttpReader::EofReader(&mut reader);

            let url = Url::parse(path).unwrap();
            let addr = stream.peer_addr().unwrap();

            headers.set(headers::UserAgent("iron-test".to_string()));

            let mut req = Request {
                method: method,
                url: url,
                body: Body::new(reader),
                local_addr: addr.clone(),
                remote_addr: addr,
                headers: headers,
                extensions: TypeMap::new()
            };

            handler.handle(&mut req)
        }
    }
}

#[cfg(test)]
mod test {
    mod request {
        extern crate router;
        extern crate urlencoded;

        use iron::headers::Headers;
        use iron::mime::Mime;
        use iron::prelude::*;
        use iron::response::{ResponseBody};
        use iron::{Handler, headers, status};

        use super::super::mock::request;

        use self::urlencoded::UrlEncodedBody;

        struct HelloWorldHandler;

        impl Handler for HelloWorldHandler {
            fn handle(&self, _: &mut Request) -> IronResult<Response> {
                Ok(Response::with((status::Ok, "Hello, world!")))
            }
        }

        struct RouterHandler;

        impl Handler for RouterHandler {
            fn handle(&self, req: &mut Request) -> IronResult<Response> {
                let params = req.extensions
                    .get::<router::Router>()
                    .expect("Expected to get a Router from the request extensions.");
                let id = params.find("id").unwrap();

                Ok(Response::with((status::Ok, id)))
            }
        }

        struct PostHandler;

        impl Handler for PostHandler {
            fn handle(&self, req: &mut Request) -> IronResult<Response> {
                let body = req.get_ref::<UrlEncodedBody>()
                    .expect("Expected to extract a UrlEncodedBody from the request");
                let first_name = body.get("first_name").unwrap()[0].to_owned();
                let last_name = body.get("last_name").unwrap()[0].to_owned();

                Ok(Response::with((status::Ok, first_name + " " + &last_name)))
            }
        }

        struct UpdateHandler;

        impl Handler for UpdateHandler {
            fn handle(&self, req: &mut Request) -> IronResult<Response> {
                let id = {
                    let params = req.extensions
                        .get::<router::Router>()
                        .expect("Expected to get a Router from request extensions.");
                    params.find("id").unwrap().parse::<String>().unwrap()
                };

                let body = req.get_ref::<UrlEncodedBody>()
                    .expect("Expected to extract a UrlEncodedBody from the request");
                let first_name = body.get("first_name").unwrap()[0].to_owned();
                let last_name = body.get("last_name").unwrap()[0].to_owned();

                Ok(Response::with((status::Ok, [first_name, last_name, id].join(" "))))
            }
        }

        struct OptionsHandler;

        impl Handler for OptionsHandler {
            fn handle(&self, _: &mut Request) -> IronResult<Response> {
                Ok(Response::with((status::Ok, "ALLOW: GET,POST")))
            }
        }

        struct HeadHandler;

        impl Handler for HeadHandler {
            fn handle(&self, _: &mut Request) -> IronResult<Response> {
                Ok(Response::with(status::Ok))
            }
        }

        #[test]
        fn test_get() {
            let response = request::get("http://localhost:3000", Headers::new(), HelloWorldHandler);
            let result = extract_body(response.unwrap());

            assert_eq!(result, b"Hello, world!");
        }

        #[test]
        fn test_post() {
            let mut headers = Headers::new();
            let mime: Mime = "application/x-www-form-urlencoded".parse().unwrap();
            headers.set(headers::ContentType(mime));
            let response = request::post("http://localhost:3000/users",
                                         headers,
                                         "first_name=Example&last_name=User",
                                         PostHandler);
            let result = extract_body(response.unwrap());

            assert_eq!(result, b"Example User");
        }

        #[test]
        fn test_patch() {
            let mut router = router::Router::new();
            router.patch("/users/:id", UpdateHandler);

            let mut headers = Headers::new();
            let mime: Mime = "application/x-www-form-urlencoded".parse().unwrap();
            headers.set(headers::ContentType(mime));
            let response = request::patch("http://localhost:3000/users/1",
                                         headers,
                                         "first_name=Example&last_name=User",
                                         router);
            let result = extract_body(response.unwrap());

            assert_eq!(result, b"Example User 1");
        }

        #[test]
        fn test_put() {
            let mut router = router::Router::new();
            router.put("/users/:id", UpdateHandler);

            let mut headers = Headers::new();
            let mime: Mime = "application/x-www-form-urlencoded".parse().unwrap();
            headers.set(headers::ContentType(mime));
            let response = request::put("http://localhost:3000/users/2",
                                         headers,
                                         "first_name=Example&last_name=User",
                                         router);
            let result = extract_body(response.unwrap());

            assert_eq!(result, b"Example User 2");
        }

        #[test]
        fn test_delete() {
            let mut router = router::Router::new();
            router.delete("/:id", RouterHandler);

            let response = request::delete("http://localhost:3000/1", Headers::new(), router);
            let result = extract_body(response.unwrap());

            assert_eq!(result, b"1");
        }


        #[test]
        fn test_options() {
            let response = request::options("http://localhost:3000/users/options", Headers::new(), OptionsHandler);
            let result = extract_body(response.unwrap());

            assert_eq!(result, b"ALLOW: GET,POST");
        }

        #[test]
        fn test_head() {
            let response = request::head("http://localhost:3000/users", Headers::new(), HeadHandler);
            let result = extract_body(response.unwrap());

            assert_eq!(result, []);
        }

        fn extract_body(response: Response) -> Vec<u8> {
            let mut result = Vec::new();

            {
                let mut response_body = ResponseBody::new(&mut result);
                match response.body {
                    Some(mut body) => body.write_body(&mut response_body).ok(),
                    None => None,
                };
            }

            result
        }
    }
}
