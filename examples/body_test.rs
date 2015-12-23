extern crate iron;
extern crate iron_test;
extern crate mime;
extern crate urlencoded;

use iron::{Handler, status};
use iron::prelude::*;

use urlencoded::UrlEncodedBody;

struct BodyHandler;

impl Handler for BodyHandler {
    fn handle(&self, req: &mut Request) -> IronResult<Response> {
        let body = req.get_ref::<UrlEncodedBody>()
            .expect("Expected to extract a UrlEncodedBody from the request");
        let first_name = body.get("first_name").unwrap()[0].to_owned();
        let last_name = body.get("last_name").unwrap()[0].to_owned();

        Ok(Response::with((status::Ok, first_name + " " + &last_name)))
    }
}

fn main() {
    Iron::new(BodyHandler).http("localhost:3000").ok();
}

#[cfg(test)]
mod test {
    use iron::Headers;
    use iron::headers::ContentType;
    use iron::prelude::*;

    use iron_test::{request, response};

    use mime::Mime;

    use super::BodyHandler;

    #[test]
    fn test_body() {
        let mut headers = Headers::new();
        let mime: Mime = "application/x-www-form-urlencoded".parse().unwrap();
        headers.set(ContentType(mime));
        let response = request::post("http://localhost:3000/users",
                                     headers,
                                     "first_name=Example&last_name=User",
                                     &BodyHandler);
        let result = response::extract_body_to_bytes(response.unwrap());

        assert_eq!(result, b"Example User");
    }
}
