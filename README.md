Iron Test
=========
[![Build Status](https://secure.travis-ci.org/reem/iron-test.svg?branch=master)](https://travis-ci.org/reem/iron-test)
[![Crates.io Status](http://meritbadge.herokuapp.com/iron-test)](https://crates.io/crates/iron-test)
[![Documentation](https://docs.rs/iron-test/badge.svg)](https://docs.rs/iron-test)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](https://raw.githubusercontent.com/reem/iron-test/master/LICENSE)

> A suite of convenience methods and constructors for making requests to Iron Handlers.

## Example

```rust
extern crate iron;
extern crate iron_test;

use iron::prelude::*;
use iron::{Handler, Headers, status};
use iron_test::{request, response};

struct HelloWorldHandler;

impl Handler for HelloWorldHandler {
    fn handle(&self, _: &mut Request) -> IronResult<Response> {
        Ok(Response::with((status::Ok, "Hello, world!")))
    }
}

#[test]
fn test_hello_world() {
    let response = request::get("http://localhost:3000/hello",
                                Headers::new(),
                                &HelloWorldHandler).unwrap();
    let result_body = response::extract_body_to_bytes(response);

    assert_eq!(result_body, b"Hello, world!");
}
```

## API

### request
The request API implements convenience methods for all the major HTTP verbs
except CONNECT and TRACE. They're broken down as follows.

```Rust
// Generates empty body
request::get<H: Handler>(path: &str, headers: Headers, handler: &H) -> IronResult<Response>
request::options<H: Handler>(path: &str, headers: Headers, handler: &H) -> IronResult<Response>
request::delete<H: Handler>(path: &str, headers: Headers, handler: &H) -> IronResult<Response>
request::head<H: Handler>(path: &str, headers: Headers, handler: &H) -> IronResult<Response>

// Accepts a `&str` body
request::post<H: Handler>(path: &str, headers: Headers, body: &str, handler: &H) -> IronResult<Response>
request::patch<H: Handler>(path: &str, headers: Headers, body: &str, handler: &H) -> IronResult<Response>
request::put<H: Handler>(path: &str, headers: Headers, body: &str, handler: &H) -> IronResult<Response>
```

The requests that it makes sense for accept a `&str` body, while the other
requests generate an empty body for you. The request is passed directly to 
the `handle` call on the Handler, and the raw result is returned to you.

For examples of testing different handlers, head over to the [examples
directory](https://github.com/reem/iron-test/tree/master/examples).

### Creating project layout for tests

Sometimes it is useful to have a predefined directory layout with specific
files in it. You can easily create a simple project directory using a
ProjectBuilder.

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

### Installation
If you're using Cargo, just add iron-test to your Cargo.toml, and point it at
the git url.
```Rust
[dependencies]

iron-test = { git = "https://github.com/reem/iron-test" }
```

### Author

Jonathan Reem

### Get Help

Come find us on `#iron` or `#rust` on `irc.mozilla.net`

### License

MIT
