use hyper::header::ContentType;
use hyper::mime::{Mime, TopLevel, SubLevel, Attr, Value};

use iron::Headers;

use rand::Rng;
use rand;

use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use super::RequestBody;

pub const BOUNDARY_LENGTH: usize = 32;

/// A trait for describing the different types of entries that a multipart
/// body can have, and implementing the functions needed to be able to write
/// each kind of entry to the body.
pub trait MultipartEntry {
    /// A required method for MultipartEntry that returns the headers to write
    /// to the multipart body.
    fn headers(&self) -> String;

    /// A required method for MultipartEntry that returns the value to write
    /// to the multipart body.
    fn value(&self) -> String;

    /// A method that takes a boundary and the Entry's headers, and writes them
    /// to a MultipartBody.
    fn write_headers(&self, body: &mut MultipartBody) {
        let boundary = body.full_boundary();
        body.parts.push(boundary);
        body.parts.push(self.headers());
    }

    /// A method that takes a boundary and the Entry's value, and writes them
    /// to a MultipartBody.
    fn write_value(&self, body: &mut MultipartBody) {
        body.parts.push(self.value())
    }
}

/// A struct representing a simple key:value pair in a multipart body.
pub struct MultipartTextEntry {
    key: String,
    value: String,
}

impl MultipartEntry for MultipartTextEntry {
    fn headers(&self) -> String {
        "Content-Disposition: form-data; name=\"".to_owned() + &self.key + "\"\r\n"
    }

    fn value(&self) -> String {
        self.value.clone()
    }
}

/// A struct representing a key:file pair in a multipart body. Contains a `key`
/// field representing the key that the file was uploaded at, and a `path` field
/// representing where the file is on the file system.
pub struct MultipartFileEntry {
    key: String,
    path: PathBuf,
}

impl MultipartEntry for MultipartFileEntry {
    fn headers(&self) -> String {
        let filename = self.path.file_name().unwrap();
        "Content-Disposition: form-data; name=\"".to_owned() + &self.key + "\"; filename = \"" + filename.to_str().unwrap() + "\"\r\n"
    }

    fn value(&self) -> String {
        let mut file_body = String::new();
        let file_result = File::open(self.path.clone());

        match file_result {
            Ok(mut file) => {
                file.read_to_string(&mut file_body).ok();
                file_body
            },
            // Since this is used in tests, just panic with the error if
            // we had problems opening the file.
            Err(err) => panic!("{}", err),
        }
    }
}

/// A struct representing the parts of a Multipart request body. Contains a
/// `boundary` field that is the raw generated boundary used to separate the
/// request body, and a Vector of `parts`, that represent the different entries
/// and boundaries in the request.
pub struct MultipartBody {
    /// A field holding the raw `boundary` separator used in the request body.
    pub boundary: String,
    parts: Vec<String>,
}

impl MultipartBody {
    /// Initializes a new MultipartBody with a randomly generated `boundary`,
    /// and an empty Vector of parts.
    pub fn new() -> Self {
        MultipartBody {
            boundary: MultipartBody::generate_boundary(),
            parts: Vec::new(),
        }
    }

    /// Writes a key:value pair to the MultipartBody, pushing the appropriate
    /// boundaries, headers, and value.
    ///
    /// # Examples
    ///
    /// ```
    /// use iron_test::request::MultipartBody;
    ///
    /// let mut body = MultipartBody::new();
    /// body.write("key".to_owned(), "value".to_owned());
    /// ```
    ///
    pub fn write(&mut self, key: String, value: String) {
        let entry = MultipartTextEntry {
            key: key,
            value: value,
        };

        entry.write_headers(self);
        entry.write_value(self);
    }

    /// 'Uploads' a key:file pair to the MultipartBody, pulling the file body
    /// from the given path, and pushing the appropriate boundaries, headers,
    /// and file body.
    ///
    /// # Examples
    ///
    /// ```
    /// use iron_test::request::MultipartBody;
    /// use std::fs::File;
    /// use std::io::Write;
    /// use std::path::PathBuf;
    ///
    /// let mut body = MultipartBody::new();
    ///
    /// let path = PathBuf::from("/tmp/file.txt");
    /// let mut file = File::create(path.clone()).unwrap();
    /// file.write_all(b"Hello, world!").ok();
    ///
    /// body.upload("key".to_owned(), path);
    /// ```
    ///
    pub fn upload(&mut self, key: String, path: PathBuf) {
        let entry = MultipartFileEntry {
            key: key,
            path: path,
        };

        entry.write_headers(self);
        entry.write_value(self);
    }

    fn full_boundary(&self) -> String {
        "--".to_owned() + &self.boundary.clone()
    }

    fn generate_boundary() -> String {
        rand::thread_rng().gen_ascii_chars().take(BOUNDARY_LENGTH).collect()
    }
}

impl RequestBody for MultipartBody {
    /// Builds the final body for use in an Iron::Request. Adds a closing
    /// boundary to the parts Vector, and then joins everything on a newline.
    fn for_request(&mut self) -> String {
        let closing_boundary = self.full_boundary() + "--";
        self.parts.push(closing_boundary);
        self.parts.join("\r\n")
    }

    /// Set the Content-Type as multipart/form-data; boundary=<boundary>
    fn set_headers(&self, headers: &mut Headers) {
        headers.set(ContentType(Mime(TopLevel::Multipart,
                                     SubLevel::FormData,
                                     vec![(Attr::Boundary, Value::Ext(self.boundary.clone()))])));
    }
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use super::{MultipartFileEntry, MultipartEntry};

    #[test]
    #[should_panic(expected = "No such file or directory (os error 2)")]
    fn test_invalid_file() {
        let entry = MultipartFileEntry {
            key: "key".to_owned(),
            path: PathBuf::from("/invalid/file")
        };

        entry.value();
    }
}
