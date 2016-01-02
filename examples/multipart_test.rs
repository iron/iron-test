extern crate iron;
extern crate iron_test;
extern crate params;

use iron::{Handler, status};
use iron::prelude::*;

use params::Params;

struct MultipartHandler;

impl Handler for MultipartHandler {
    fn handle(&self, req: &mut Request) -> IronResult<Response> {
        let params = req.get_ref::<params::Params>()
            .expect("Expected to extract Params from the request.");
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

fn main() {
    Iron::new(MultipartHandler).http("localhost:3000").unwrap();
}

#[cfg(test)]
mod test {
    use iron::Headers;

    use iron_test::{request, response};

    use std::fs::File;
    use std::io::Write;
    use std::path::PathBuf;

    use super::MultipartHandler;

    #[test]
    fn test_post_key() {
        let mut body = request::MultipartBody::new();
        body.write("key".to_owned(), "my value".to_owned());

        let response = request::post_multipart("http://localhost:3000/multipart",
                                               Headers::new(),
                                               body,
                                               &MultipartHandler);
        let result = response::extract_body_to_bytes(response.unwrap());

        assert_eq!(result, b"my value");
    }

    #[test]
    fn test_post_file() {
        let mut body = request::MultipartBody::new();

        let path = PathBuf::from("/tmp/file.txt");
        let mut file = File::create(path.clone()).unwrap();
        file.write_all(b"Hello, world!").ok();

        body.upload("key".to_owned(), path);
        let response = request::post_multipart("http://localhost:3000",
                                               Headers::new(),
                                               body,
                                               &MultipartHandler);
        let result = response::extract_body_to_string(response.unwrap());

        assert_eq!(result, "file.txt");
    }
}
