# Iron Test

> A suite of constructors for Iron Request's and Response's for testing.

## Example

```rust
#[test] fn test_new() {
    let req = request::new(method::Get, "localhost:3000");
    assert_eq!(req.method, method::Get);
    assert_eq!(req.url.serialize().as_slice(), "http://localhost:3000/");
}
```

## API

### Request

> `request::new<S: Str>(method: method::Method, host: S) -> Request`

Create a new request at `/` on the specified host with the specified method.

Ex:

```rust
let req = request::new(method::Get, "localhost:3000");
assert_eq!(req.method, method::Get);
assert_eq!(req.url.serialize().as_slice(), "http://localhost:3000/");
```

> `request::at(method: method::Method, path: url::Url) -> Request`

Create a new request at the specific Url with the specified method.

Ex:

```rust
let req = request::at(method::Post, Url::parse("http://www.google.com/").unwrap());
assert_eq!(req.method, method::Post);
assert_eq!(req.url.serialize().as_slice(), "http://www.google.com/");
```

> `request::at_with<S: Str>(method: method::Method, path: url::Url, body: S) -> Request`

Create a new request at the specified Url with the specified method
and the specified content as the body of the request.

Ex:

```rust
let req = request::at_with(method::Put, Url::parse("http://www.google.com/").unwrap(), "Hello Google!");
assert_eq!(req.method, method::Put);
assert_eq!(req.url.serialize().as_slice(), "http://www.google.com/");
assert_eq!(req.body.as_slice(), "Hello Google!");
```

### Response

> `response::new() -> Response`

Create a new, blank, response.

Ex:

```rust
let mut res = response::new();
assert_eq!(res.status, None);
assert_eq!(res.body.read_to_string().unwrap().as_slice(), "");
```

> `response::with<B: BytesContainer>(status: status::Status, body: B) -> Response`

Create a new response with the specified body and status.

Ex:

```rust
let mut res = response::with(status::Ok, "Hello World!");
assert_eq!(res.status, Some(status::Ok));
assert_eq!(res.body.read_to_string().unwrap().as_slice(), "Hello World!");
```

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

