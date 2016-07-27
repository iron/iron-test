/// Contains convenience methods for making requests to Iron Handlers.
use hyper::buffer::BufReader;
use hyper::http::h1::HttpReader;
use hyper::net::NetworkStream;

use iron::prelude::*;
use iron::request::Body;
use iron::{Handler, headers, Headers, method, TypeMap, Url};

use std::io::Cursor;

use super::mock_stream::MockStream;

/// Convenience method for making GET requests to Iron Handlers.
pub fn get<H: Handler>(path: &str, headers: Headers, handler: &H) -> IronResult<Response> {
    request(method::Get, path, "", headers, handler)
}

/// Convenience method for making POST requests with a body to Iron Handlers.
pub fn post<H: Handler>(path: &str, headers: Headers, body: &str, handler: &H) -> IronResult<Response> {
    request(method::Post, path, body, headers, handler)
}

/// Convenience method for making PATCH requests with a body to Iron Handlers.
pub fn patch<H: Handler>(path: &str, headers: Headers, body: &str, handler: &H) -> IronResult<Response> {
    request(method::Patch, path, body, headers, handler)
}

/// Convenience method for making PUT requests with a body to Iron Handlers.
pub fn put<H: Handler>(path: &str, headers: Headers, body: &str, handler: &H) -> IronResult<Response> {
    request(method::Put, path, body, headers, handler)
}

/// Convenience method for making DELETE requests to Iron Handlers.
pub fn delete<H: Handler>(path: &str, headers: Headers, handler: &H) -> IronResult<Response> {
    request(method::Delete, path, "", headers, handler)
}

/// Convenience method for making OPTIONS requests to Iron Handlers.
pub fn options<H: Handler>(path: &str, headers: Headers, handler: &H) -> IronResult<Response> {
    request(method::Options, path, "", headers, handler)
}

/// Convenience method for making HEAD requests to Iron Handlers.
pub fn head<H: Handler>(path: &str, headers: Headers, handler: &H) -> IronResult<Response> {
    request(method::Head, path, "", headers, handler)
}

/// Constructs an Iron::Request from the given parts and passes it to the
/// `handle` method on the given Handler.
pub fn request<H: Handler>(method: method::Method,
                            path: &str,
                            body: &str,
                            mut headers: Headers,
                            handler: &H) -> IronResult<Response> {
    let content_length = body.len() as u64;
    let data = Cursor::new(body.as_bytes().to_vec());
    let mut stream = MockStream::new(data);
    // Clone the stream so we can read the peer_addr off the original.
    let mut stream_clone = stream.clone();
    let mut reader = BufReader::new(&mut stream_clone as &mut NetworkStream);
    let reader = HttpReader::SizedReader(&mut reader, content_length);

    let url = Url::parse(path).unwrap();
    let addr = stream.peer_addr().unwrap();

    if !headers.has::<headers::UserAgent>() {
        headers.set(headers::UserAgent("iron-test".to_string()));
    }
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

#[cfg(test)]
mod test {
    extern crate router;
    extern crate urlencoded;

    use iron::headers::Headers;
    use iron::mime::Mime;
    use iron::prelude::*;
    use iron::{Handler, headers, status};

    use response::{extract_body_to_bytes, extract_body_to_string};

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

    struct UserAgentHandler;

    impl Handler for UserAgentHandler {
        fn handle(&self, req: &mut Request) -> IronResult<Response> {
            let user_agent = req.headers.get::<headers::UserAgent>().unwrap();
            Ok(Response::with((status::Ok, user_agent.to_string())))
        }
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
                            "first_name=Example&last_name=User",
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
                             "first_name=Example&last_name=User",
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
                           "first_name=Example&last_name=User",
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

    #[test]
    fn test_user_agent_not_provided() {
        let headers = Headers::new();
        let response = get("http://localhost:3000/", headers, &UserAgentHandler);
        let result = extract_body_to_string(response.unwrap());

        assert_eq!(result, "iron-test");
    }

    #[test]
    fn test_user_agent_provided() {
        let mut headers = Headers::new();
        headers.set(headers::UserAgent("CustomAgent/1.0".to_owned()));
        let response = get("http://localhost:3000/", headers, &UserAgentHandler);
        let result = extract_body_to_string(response.unwrap());

        assert_eq!(result, "CustomAgent/1.0");
    }
}
