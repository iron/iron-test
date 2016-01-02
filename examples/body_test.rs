extern crate iron;
extern crate iron_test;
extern crate mime;
extern crate params;

use iron::{Handler, status};
use iron::prelude::*;

struct BodyHandler;

impl Handler for BodyHandler {
    fn handle(&self, req: &mut Request) -> IronResult<Response> {
        let params = req.get_ref::<params::Params>()
            .expect("Expected to extract a UrlEncodedBody from the request");
        let first_name = get_params_value(params, "first_name");
        let last_name = get_params_value(params, "last_name");

        Ok(Response::with((status::Ok, first_name + " " + &last_name)))
    }
}

fn get_params_value<'a>(params: &params::Map, key: &'a str) -> String {
    let value = params.get(key);

    // destructure the Value enum to get the param value out
    match value {
        Some(&params::Value::String(ref string)) => string.clone(),
        _ => String::new(),
    }
}

fn main() {
    Iron::new(BodyHandler).http("localhost:3000").unwrap();
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
