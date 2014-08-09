#![license = "MIT"]
#![deny(missing_doc)]
#![deny(warnings)]

//! A set of constructors for mocking Iron objects.

extern crate iron;
extern crate http;
extern crate url;

/// Contains tooling for mocking various Iron objects.
pub mod mock {
    /// Contains constructors for mocking Iron Requests.
    pub mod request {
        use url;
        use iron::{Request, Alloy};
        use http::method;
        use http::headers::request::HeaderCollection;

        /// Create a new request at `/` on the specified host with the
        /// specified method.
        pub fn new<S: Str>(method: method::Method, host: S) -> Request {
            let url = url::Url::parse("http://".to_string().append(host.as_slice()).as_slice()).unwrap();
            at(method, url)
        }

        /// Create a new request at the specific Url with the specified method.
        pub fn at(method: method::Method, path: url::Url) -> Request {
            at_with(method, path, "")
        }

        /// Create a new request at the specified Url with the specified method
        /// and the specified content as the body of the request.
        pub fn at_with<S: Str>(method: method::Method, path: url::Url, body: S) -> Request {
            Request {
                url: path,
                body: body.as_slice().to_string(),
                method: method,
                remote_addr: None,
                headers: box HeaderCollection::new(),
                alloy: Alloy::new()
            }
        }
    }

    /// Contains constructors for mocking Iron Responses.
    pub mod response {
        use iron::Response;
        use http::status;
        use http::headers::response::HeaderCollection;

        use std::path::BytesContainer;
        use std::io::MemReader;

        /// Create a new, blank, response.
        pub fn new() -> Response {
            with("")
        }

        /// Create a new response with the specified body.
        pub fn with<B: BytesContainer>(body: B) -> Response {
            Response {
                body: box MemReader::new(body.container_as_bytes().to_vec()) as Box<Reader>,
                headers: box HeaderCollection::new(),
                status: None
            }
        }

        /// Create a new response with the specified body and status.
        pub fn with_status(status: status::Status) -> Response {
            Response {
                body: box MemReader::new(vec![]) as Box<Reader>,
                headers: box HeaderCollection::new(),
                status: Some(status)
            }
        }

        /// Create a new response with the specified body and status.
        pub fn with_body_and_status<B: BytesContainer>(body: B, status: status::Status) -> Response {
            Response {
                body: box MemReader::new(body.container_as_bytes().to_vec()) as Box<Reader>,
                headers: box HeaderCollection::new(),
                status: Some(status)
            }
        }
    }
}

#[cfg(test)]
mod test {
    mod request {
        use super::super::mock::request;
        use http::method;
        use url::Url;

        #[test] fn test_new() {
            let req = request::new(method::Get, "localhost:3000");
            assert_eq!(req.method, method::Get);
            assert_eq!(req.url.serialize().as_slice(), "http://localhost:3000/");
        }

        #[test] fn test_at() {
            let req = request::at(method::Post, Url::parse("http://www.google.com/").unwrap());
            assert_eq!(req.method, method::Post);
            assert_eq!(req.url.serialize().as_slice(), "http://www.google.com/");
        }

        #[test] fn test_at_with() {
            let req = request::at_with(method::Put, Url::parse("http://www.google.com/").unwrap(), "Hello Google!");
            assert_eq!(req.method, method::Put);
            assert_eq!(req.url.serialize().as_slice(), "http://www.google.com/");
            assert_eq!(req.body.as_slice(), "Hello Google!");
        }
    }
}

