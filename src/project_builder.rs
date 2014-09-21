use std::io::fs::{mod, PathExtensions};
use std::io::{mod, IoResult};
use std::os;
use std::path::{Path, BytesContainer};
use std::vec::Vec;
use std::fmt::Show;
use uuid::Uuid;

static IRON_INTEGRATION_TEST_DIR : &'static str = "iron-integration-tests";

local_data_key!(task_id: Uuid)

#[deriving(PartialEq,Clone)]
struct FileBuilder {
    path: Path,
    body: String
}

impl FileBuilder {
    /// creates new instance of ProjectBuilder
    pub fn new(path: Path, body: &str) -> FileBuilder {
        FileBuilder { path: path, body: body.to_string() }
    }

    fn mk(&self) -> Result<(), String> {
        try!(mkdir_recursive(&self.dirname()));

        let mut file = try!(
            fs::File::create(&self.path)
                .with_err_msg(format!("Could not create file; path={}",
                                      self.path.display())));

        file.write_str(self.body.as_slice())
            .with_err_msg(format!("Could not write to file; path={}",
                                  self.path.display()))
    }

    fn dirname(&self) -> Path {
        Path::new(self.path.dirname())
    }
}


/// ProjectBuilder allows to incrementally build a project layout to be used in tests
#[deriving(PartialEq,Clone)]
pub struct ProjectBuilder {
    name: String,
    root: Path,
    files: Vec<FileBuilder>,
}

impl ProjectBuilder {
    /// ProjectBuilder constructor
    /// Creates a directory with name
    pub fn new(name: &str) -> ProjectBuilder {
        task_id.replace(Some(Uuid::new_v4()));
        debug!("path setup: root={}", root().display());
        root().rm_rf().unwrap();
        ProjectBuilder {
            name: name.to_string(),
            root: root().join(name),
            files: vec!(),
        }
    }

    /// Root of the project layout
    pub fn root(&self) -> Path {
        self.root.clone()
    }

    /// Adds new file to builder with given path and body
    pub fn file<B: BytesContainer, S: Str>(mut self, path: B, body: S) -> ProjectBuilder {
        self.files.push(FileBuilder::new(self.root.join(path), body.as_slice()));
        self
    }

    /// Creates the project layout, based on current state of the builder
    pub fn build(&self) -> &ProjectBuilder {
        match self.build_with_result() {
            Err(e) => fail!(e),
            _ => return self
        }
    }

    /// Creates the project layout, based on current state of the builder
    pub fn build_with_result(&self) -> Result<(), String> {

        for file in self.files.iter() {
            try!(file.mk());
        }

        Ok(())
    }
}

impl Drop for ProjectBuilder {
    fn drop(&mut self) {
        match self.root().dir_path().rm_rf() {
            Ok(_) => debug!("Successfully cleaned up the test directory; path = {}", self.root().dir_path().display()),
            Err(e) => debug!("Failed to cleanup the test directory; path = {}; {}", self.root().dir_path().display(), e)
        }
    }
}

/// recursively creates the directory with all subdirectories
pub fn mkdir_recursive(path: &Path) -> Result<(), String> {
    fs::mkdir_recursive(path, io::UserDir)
        .with_err_msg(format!("could not create directory; path={}",
                              path.display()))
}

/// Convenience methods to show errors
trait ErrMsg<T> {
    /// Convenience method on Result to to return either value on Ok,
    /// or value + error on Err
    fn with_err_msg(self, val: String) -> Result<T, String>;
}

impl<T, E: Show> ErrMsg<T> for Result<T, E> {
    fn with_err_msg(self, val: String) -> Result<T, String> {
        match self {
            Ok(val) => Ok(val),
            Err(err) => Err(format!("{}; original={}", val, err))
        }
    }
}

// Current test root path.
// Will be located in target/iron-integration-tests/test-<uuid>
fn root() -> Path {
    let my_id = *task_id.get().unwrap();
    integration_tests_dir().join(format!("test-{}", my_id))
}

fn integration_tests_dir() -> Path {
    os::self_exe_path().unwrap().join(IRON_INTEGRATION_TEST_DIR)
}


/// Convenience methods on Path
pub trait PathExt {
    /// Deletes directory in Path recursively
    fn rm_rf(&self) -> IoResult<()>;
}

impl PathExt for Path {
    fn rm_rf(&self) -> IoResult<()> {
        if self.exists() {
            fs::rmdir_recursive(self)
        } else {
            Ok(())
        }
    }
}
