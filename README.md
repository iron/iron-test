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

### Author

Jonathan Reem

### Get Help

Come find us on `#iron` or `#rust` on `irc.mozilla.net`

### License

MIT

