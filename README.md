# Iron Test

> A suite of convenience methods and constructors for making requests to Iron Handlers.

## Example

```rust
extern crate iron;
extern crate iron_test;

use iron::prelude::*;
use iron::{Handler, Headers, status};
use iron::response::ResponseBody;
use iron_test::mock::request;

struct HelloWorldHandler;

impl Handler for HelloWorldHandler {
  fn handle(&self, _: &mut Request) -> IronResult<Response> {
    Ok(Response::with((status::Ok, "Hello, world!")))
  }
}

#[test]
fn test_hello_world() {
    let response = request::get("http://localhost:3000/hello", Headers::new(), HelloWorldHandler).unwrap();
    let mut result_body = Vec::new();

    {
      let mut response_body = ResponseBody::new(&mut result_body);
      match response.body {
        Some(mut body) => body.write_body(&mut response_body).ok(),
        None => None,
      };
    }

    assert_eq!(response.status.unwrap(), status::Ok);
    assert_eq!(result_body, b"Hello, world!");
}
```

## API

### request
The request API implements convenience methods for all the major HTTP verbs
except CONNECT and TRACE. They're broken down as follows.

```Rust
// Generates empty body
request::get<H: Handler>(path: &str, headers: Headers, handler: H) -> IronResult<Response>
request::options<H: Handler>(path: &str, headers: Headers, handler: H) -> IronResult<Response>
request::delete<H: Handler>(path: &str, headers: Headers, handler: H) -> IronResult<Response>
request::head<H: Handler>(path: &str, headers: Headers, handler: H) -> IronResult<Response>

// Accepts a `&str` body
request::post<H: Handler>(path: &str, headers: Headers, body: &str, handler: H) -> IronResult<Response>
request::patch<H: Handler>(path: &str, headers: Headers, body: &str, handler: H) -> IronResult<Response>
request::put<H: Handler>(path: &str, headers: Headers, body: &str, handler: H) -> IronResult<Response>
```

The requests that it makes sense for accept a `&str` body, while the other
requests generate an empty body for you. The request is passed directly to 
the `handle` call on the Handler, and the raw result is returned to you.

### Creating project layout for tests

Sometimes it is useful to have a predefined directory layout with specific files in
it. You can easily create a simple project directory using a ProjectBuilder.

Ex:

```rust
use iron_test::ProjectBuilder;

#[test]
fn test_a() {
  let builder = ProjectBuilder::new("foo")
    .file("index.html", "<html><h2>hello</h2></html>")
    .file("main.css", "body{font-family: Verdana}");
  builder.build();

  // At this point you will have your project directory in:
  // target/iron-integration-tests/test-<N>/foo/
}
```
To access current project root, use `p.root()`.

ProjectBuilder implements Drop and will clean up the project when it is dropped.

### Author

Jonathan Reem

### Get Help

Come find us on `#iron` or `#rust` on `irc.mozilla.net`

### License

MIT
