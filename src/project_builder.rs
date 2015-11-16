use std::fs;
use std::io::{self, Write};
use std::env;
use std::path::{Path, PathBuf};
use std::fmt::Debug;
use uuid::Uuid;
use std::sync::{Mutex, Arc};

static IRON_INTEGRATION_TEST_DIR : &'static str = "iron-integration-tests";

#[derive(Debug, PartialEq, Clone)]
struct FileBuilder {
    path: PathBuf,
    body: Vec<u8>
}

impl FileBuilder {
    /// creates new instance of FileBuilder
    pub fn new<P: AsRef<Path>, B: Into<Vec<u8>>>(path: P, body: B) -> FileBuilder {
        FileBuilder { path: path.as_ref().to_path_buf(), body: body.into() }
    }

    fn mk(&self) -> Result<(), String> {
        try!(mkdir_recursive(&self.dirname()));

        let mut file = try!(
            fs::File::create(&self.path)
                .with_err_msg(format!("Could not create file; path={}",
                                      self.path.display())));

        file.write_all(&self.body)
            .with_err_msg(format!("Could not write to file; path={}",
                                  self.path.display()))
    }

    // FIXME: Panics if there's no parent
    fn dirname(&self) -> &Path {
        &self.path.parent().expect("parent directory")
    }
}


/// An RAII guard that controls a temporary directory of test files.
///
/// It is also a builder and is used to build up the temporary files,
/// which are then deleted on drop.
#[derive(Debug, Clone)]
pub struct ProjectBuilder {
    name: String,
    root: PathBuf,
    files: Vec<FileBuilder>,
    lock: Arc<Mutex<()>>,
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
            lock: Arc::new(Mutex::new(())),
        }
    }

    /// Get the root path of the temporary directory.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Add a new file to the temporary directory with the given contents.
    pub fn file<P, B>(mut self, path: P, body: B) -> ProjectBuilder
    where P: AsRef<Path>, B: Into<Vec<u8>> {
        self.files.push(FileBuilder::new(self.root.join(&path), body));
        self
    }

    /// Creates the project layout, based on current state of the builder
    pub fn build(&self) -> &ProjectBuilder {
        self.build_with_result().map(|_| self).unwrap()
    }

    /// Creates the project layout, based on current state of the builder
    pub fn build_with_result(&self) -> Result<(), String> {
        let _lock = self.lock.lock().unwrap();
        for file in self.files.iter() {
            try!(file.mk());
        }

        Ok(())
    }
}

impl PartialEq for ProjectBuilder {
    fn eq(&self, other: &ProjectBuilder) -> bool {
        self.name.eq(&other.name) &&
            self.root.eq(&other.root) &&
            self.files.eq(&other.files)
    }
}

impl Drop for ProjectBuilder {
    fn drop(&mut self) {
        let _lock = self.lock.lock().unwrap();
        match self.root().parent().map(BuilderPathExt::rm_rf) {
            Some(Ok(_)) => debug!("Successfully cleaned up the test directory; path = {:?}", self.root().parent().unwrap()),
            Some(Err(e)) => debug!("Failed to cleanup the test directory; path = {:?}; {}", self.root().parent().unwrap(), e),
            None => debug!("Failed to cleanup the test directory; no parent")
        }
    }
}

// Recursively creates the directory with all subdirectories
fn mkdir_recursive(path: &Path) -> Result<(), String> {
    fs::create_dir_all(path)
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
fn root(id: Uuid) -> PathBuf {
    integration_tests_dir().join(&format!("test-{}", id))
}

fn integration_tests_dir() -> PathBuf {
    env::current_exe()
        .map(|mut p| { p.pop(); p.join(IRON_INTEGRATION_TEST_DIR) })
        .unwrap()
}


/// Convenience methods on Path
pub trait BuilderPathExt {
    /// Deletes directory in Path recursively
    fn rm_rf(&self) -> io::Result<()>;
}

impl BuilderPathExt for Path {
    fn rm_rf(&self) -> io::Result<()> {
        if let Ok(_) = fs::metadata(self) {
            fs::remove_dir_all(self)
        } else {
            Ok(())
        }
    }
}
