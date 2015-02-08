use std::old_io::fs::{self, PathExtensions};
use std::old_io::IoResult;
use std::old_io as io;
use std::env;
use std::old_path::{Path, BytesContainer};
use std::fmt::Debug;
use uuid::Uuid;

static IRON_INTEGRATION_TEST_DIR : &'static str = "iron-integration-tests";

#[derive(Debug, PartialEq, Clone)]
struct FileBuilder {
    path: Path,
    body: Vec<u8>
}

impl FileBuilder {
    /// creates new instance of ProjectBuilder
    pub fn new(path: Path, body: &[u8]) -> FileBuilder {
        FileBuilder { path: path, body: body.to_vec() }
    }

    fn mk(&self) -> Result<(), String> {
        try!(mkdir_recursive(&self.dirname()));

        let mut file = try!(
            fs::File::create(&self.path)
                .with_err_msg(format!("Could not create file; path={}",
                                      self.path.display())));

        file.write_all(self.body.as_slice())
            .with_err_msg(format!("Could not write to file; path={}",
                                  self.path.display()))
    }

    fn dirname(&self) -> Path {
        Path::new(self.path.dirname())
    }
}


/// An RAII guard that controls a temporary directory of test files.
///
/// It is also a builder and is used to build up the temporary files,
/// which are then deleted on drop.
#[derive(Debug, PartialEq, Clone)]
pub struct ProjectBuilder {
    name: String,
    root: Path,
    files: Vec<FileBuilder>,
}

impl ProjectBuilder {
    /// Create a ProjectBuilder that will manage a new temporary directory
    /// making use of the current name.
    pub fn new(name: &str) -> ProjectBuilder {
        let id = Uuid::new_v4();
        let path = root(id);

        // Clear out the temp directory.
        path.rm_rf().unwrap();

        ProjectBuilder {
            name: name.to_string(),
            root: path.join(name),
            files: vec!(),
        }
    }

    /// Get the root path of the temporary directory.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Add a new file to the temporary directory with the given contents.
    pub fn file<B, C>(mut self, path: B, body: C) -> ProjectBuilder
    where B: BytesContainer, C: BytesContainer {
        self.files.push(FileBuilder::new(self.root.join(path), body.container_as_bytes()));
        self
    }

    /// Creates the project layout, based on current state of the builder
    pub fn build(&self) -> &ProjectBuilder {
        self.build_with_result().map(|_| self).unwrap()
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

// Recursively creates the directory with all subdirectories
fn mkdir_recursive(path: &Path) -> Result<(), String> {
    fs::mkdir_recursive(path, io::USER_DIR)
        .with_err_msg(format!("could not create directory; path={}",
                              path.display()))
}

/// Convenience methods to show errors
trait ErrMsg<T> {
    /// Convenience method on Result to to return either value on Ok,
    /// or value + error on Err
    fn with_err_msg(self, val: String) -> Result<T, String>;
}

impl<T, E: Debug> ErrMsg<T> for Result<T, E> {
    fn with_err_msg(self, val: String) -> Result<T, String> {
        match self {
            Ok(val) => Ok(val),
            Err(err) => Err(format!("{}; original={:?}", val, err))
        }
    }
}

// Current test root path.
// Will be located in target/iron-integration-tests/test-<uuid>
fn root(id: Uuid) -> Path {
    integration_tests_dir().join(format!("test-{}", id))
}

fn integration_tests_dir() -> Path {
    env::current_exe()
        .map(|mut p| { p.pop(); p.join(IRON_INTEGRATION_TEST_DIR) })
        .unwrap()
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

