/// Contains convenience methods for making requests to Iron Handlers.
use hyper::buffer::BufReader;
use hyper::http::h1::HttpReader;
use hyper::net::NetworkStream;
use hyper::header::ContentType;
use hyper::mime::{Mime, TopLevel, SubLevel, Attr, Value};

use iron::prelude::*;
use iron::request::Body;
use iron::{Handler, headers, Headers, method, TypeMap, Url};

use std::io::Cursor;

use super::mock_stream::MockStream;

mod multipart;
pub use self::multipart::MultipartBody;

/// Convenience method for making GET requests to Iron Handlers.
pub fn get<H: Handler>(path: &str, headers: Headers, handler: &H) -> IronResult<Response> {
    request(method::Get, path, StringBody::new(""), headers, handler)
}

/// Convenience method for making POST requests with a body to Iron Handlers.
pub fn post<H: Handler, B: RequestBody>(path: &str, headers: Headers, body: B, handler: &H) -> IronResult<Response> {
    request(method::Post, path, body, headers, handler)
}

/// Convenience method for POSTing multipart/form-data requests to Iron Handlers.
/// It takes a MultiPart Body for the body, which is used to build the multipart
/// request body.
pub fn post_multipart<H: Handler>(path: &str, mut headers: Headers, mut body: MultipartBody, handler: &H) -> IronResult<Response> {
    let request_body = body.for_request();
    headers.set(ContentType(Mime(TopLevel::Multipart, SubLevel::FormData, vec![(Attr::Boundary, Value::Ext(body.boundary))])));
    request(method::Post, path, request_body, headers, handler)
}

/// Convenience method for making PATCH requests with a body to Iron Handlers.
pub fn patch<H: Handler, B: RequestBody>(path: &str, headers: Headers, body: B, handler: &H) -> IronResult<Response> {
    request(method::Patch, path, body, headers, handler)
}

/// Convenience method for making PUT requests with a body to Iron Handlers.
pub fn put<H: Handler, B: RequestBody>(path: &str, headers: Headers, body: B, handler: &H) -> IronResult<Response> {
    request(method::Put, path, body, headers, handler)
}

/// Convenience method for making DELETE requests to Iron Handlers.
pub fn delete<H: Handler>(path: &str, headers: Headers, handler: &H) -> IronResult<Response> {
    request(method::Delete, path, StringBody::new(""), headers, handler)
}

/// Convenience method for making OPTIONS requests to Iron Handlers.
pub fn options<H: Handler>(path: &str, headers: Headers, handler: &H) -> IronResult<Response> {
    request(method::Options, path, StringBody::new(""), headers, handler)
}

/// Convenience method for making HEAD requests to Iron Handlers.
pub fn head<H: Handler>(path: &str, headers: Headers, handler: &H) -> IronResult<Response> {
    request(method::Head, path, StringBody::new(""), headers, handler)
}

/// Constructs an Iron::Request from the given parts and passes it to the
/// `handle` method on the given Handler.
pub fn request<H: Handler, B: RequestBody>(method: method::Method,
                            path: &str,
                            body: B,
                            mut headers: Headers,
                            handler: &H) -> IronResult<Response> {
    body.set_headers(&headers);
    let body = body.for_request();
    let content_length = body.len() as u64;
    let data = Cursor::new(body.as_bytes().to_vec());
    let mut stream = MockStream::new(data);
    // Clone the stream so we can read the peer_addr off the original.
    let mut stream_clone = stream.clone();
    let mut reader = BufReader::new(&mut stream_clone as &mut NetworkStream);
    let reader = HttpReader::SizedReader(&mut reader, content_length);

    let url = Url::parse(path).unwrap();
    let addr = stream.peer_addr().unwrap();

    headers.set(headers::UserAgent("iron-test".to_string()));
    headers.set(headers::ContentLength(content_length));

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

/// A trait describing the interface a request body must implement.
pub trait RequestBody {
    /// The final body to write to the request, in the form of a string.
    fn for_request(&self) -> &str;

    /// Set any appropriate headers for the request e.g. Content-Type
    fn set_headers(&self, headers: &Headers);
}

/// A simple string request body.
pub struct StringBody {
    string: String,
}

impl StringBody {
    /// Initialize a new StringBody with the given string.
    pub fn new(body: &str) -> StringBody {
        StringBody {
            string: body.to_owned()
        }
    }
}

impl RequestBody for StringBody {
    fn for_request(&self) -> &str {
        &self.string
    }

    fn set_headers(&self, _: &Headers) {
        ()
    }
}

#[cfg(test)]
mod test {
    extern crate params;
    extern crate router;
    extern crate urlencoded;

    use iron::headers::Headers;
    use iron::mime::Mime;
    use iron::prelude::*;
    use iron::{Handler, headers, status};

    use response::extract_body_to_bytes;
    use response::extract_body_to_string;

    use std::fs::File;
    use std::io::Write;
    use std::path::PathBuf;

    use self::urlencoded::UrlEncodedBody;

    use super::*;

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

    struct MultipartFormHandler;

    impl Handler for MultipartFormHandler {
        fn handle(&self, req: &mut Request) -> IronResult<Response> {
            let params = req.get_ref::<params::Params>().expect("Params");
            let value = params.get("key");

            match value {
                Some(&params::Value::String(ref string)) => {
                    Ok(Response::with((status::Ok, string.to_owned())))
                },
                Some(&params::Value::File(ref file)) => {
                    Ok(Response::with((status::Ok, file.filename().unwrap())))
                },
                _ => Ok(Response::with(status::Ok)),
            }
        }
    }

    #[test]
    fn test_post_multipart_text() {
        let mut body = super::MultipartBody::new();
        body.write("key".to_owned(), "my_song".to_owned());
        let response = post_multipart("http://localhost:3000", Headers::new(), body, &MultipartFormHandler);
        let result = extract_body_to_string(response.unwrap());

        assert_eq!(result, "my_song");
    }

    #[test]
    fn test_post_multipart_file() {
        let mut body = super::MultipartBody::new();

        let path = PathBuf::from("/tmp/file.txt");
        let mut file = File::create(path.clone()).unwrap();
        file.write_all(b"Hello, world!").ok();

        body.upload("key".to_owned(), path);
        let response = post_multipart("http://localhost:3000", Headers::new(), body, &MultipartFormHandler);
        let result = extract_body_to_string(response.unwrap());

        assert_eq!(result, "file.txt");
    }

    #[test]
    fn test_get() {
        let response = get("http://localhost:3000", Headers::new(), &HelloWorldHandler);
        let result = extract_body_to_bytes(response.unwrap());

        assert_eq!(result, b"Hello, world!");
    }

    #[test]
    fn test_post() {
        let mut headers = Headers::new();
        let mime: Mime = "application/x-www-form-urlencoded".parse().unwrap();
        headers.set(headers::ContentType(mime));
        let response = post("http://localhost:3000/users",
                            headers,
                            StringBody::new("first_name=Example&last_name=User"),
                            &PostHandler);
        let result = extract_body_to_bytes(response.unwrap());

        assert_eq!(result, b"Example User");
    }

    #[test]
    fn test_patch() {
        let mut router = router::Router::new();
        router.patch("/users/:id", UpdateHandler);

        let mut headers = Headers::new();
        let mime: Mime = "application/x-www-form-urlencoded".parse().unwrap();
        headers.set(headers::ContentType(mime));
        let response = patch("http://localhost:3000/users/1",
                             headers,
                             StringBody::new("first_name=Example&last_name=User"),
                             &router);
        let result = extract_body_to_bytes(response.unwrap());

        assert_eq!(result, b"Example User 1");
    }

    #[test]
    fn test_put() {
        let mut router = router::Router::new();
        router.put("/users/:id", UpdateHandler);

        let mut headers = Headers::new();
        let mime: Mime = "application/x-www-form-urlencoded".parse().unwrap();
        headers.set(headers::ContentType(mime));
        let response = put("http://localhost:3000/users/2",
                           headers,
                           StringBody::new("first_name=Example&last_name=User"),
                           &router);
        let result = extract_body_to_bytes(response.unwrap());

        assert_eq!(result, b"Example User 2");
    }

    #[test]
    fn test_delete() {
        let mut router = router::Router::new();
        router.delete("/:id", RouterHandler);

        let response = delete("http://localhost:3000/1", Headers::new(), &router);
        let result = extract_body_to_bytes(response.unwrap());

        assert_eq!(result, b"1");
    }


    #[test]
    fn test_options() {
        let response = options("http://localhost:3000/users/options", Headers::new(), &OptionsHandler);
        let result = extract_body_to_bytes(response.unwrap());

        assert_eq!(result, b"ALLOW: GET,POST");
    }

    #[test]
    fn test_head() {
        let response = head("http://localhost:3000/users", Headers::new(), &HeadHandler);
        let result = extract_body_to_bytes(response.unwrap());

        assert_eq!(result, []);
    }
}
