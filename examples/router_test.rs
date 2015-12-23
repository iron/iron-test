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

fn app_router() -> Router {
    let mut router = Router::new();
    router.get("/:id", RouterHandler);
    router
}

fn main() {
    Iron::new(app_router()).http("localhost:3000").ok();
}

#[cfg(test)]
mod test {
    use iron::Headers;
    use iron::prelude::*;

    use iron_test::{request, response};

    use router::Router;

    use super::{app_router, RouterHandler};

    #[test]
    fn test_router() {
        let response = request::get("http://localhost:3000/1",
                                    Headers::new(),
                                    &app_router());
        let result = extract_body_to_bytes(response.unwrap());

        assert_eq!(result, b"1");
    }
}
