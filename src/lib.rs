#![license = "MIT"]
#![deny(missing_doc)]
#![deny(warnings)]

//! Crate comment goes here

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

    }
}

#[cfg(test)]
mod test {

}

