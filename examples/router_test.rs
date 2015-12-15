extern crate iron;
extern crate iron_test;
extern crate router;

use iron::{Handler, status};
use iron::prelude::*;

use router::Router;

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

#[cfg(test)]
mod test {
    use iron::Headers;
    use iron::prelude::*;
    use iron::response::ResponseBody;

    use iron_test::mock::request;

    use router::Router;

    use super::RouterHandler;

    #[test]
    fn test_router() {
        let mut router = Router::new();
        router.get("/:id", RouterHandler);

        let response = request::get("http://localhost:3000/1",
                                    Headers::new(),
                                    router);
        let result = extract_body(response.unwrap());

        assert_eq!(result, b"1");
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
