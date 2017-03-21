use std::path::{Path, PathBuf};
use std::io::ErrorKind::NotFound;
use std::fs;

use hyper::method::Method::{Get, Head};
use hyper::header::Connection;

use status::StatusCode;
use request::Request;
use response::Response;
use middleware::{Middleware, MiddlewareResult};

// this should be much simpler after unboxed closures land in Rust.

#[derive(Clone)]
pub struct StaticFilesHandler {
    root_path: PathBuf
}

impl<D> Middleware<D> for StaticFilesHandler {
    fn invoke<'a>(&self, req: &mut Request<D>, res: Response<'a, D>)
            -> MiddlewareResult<'a, D> {
        match req.origin.method {
            Get | Head => self.with_file(self.extract_path(req), res),
            _ => res.next_middleware()
        }
    }
}

impl StaticFilesHandler {
    /// Create a new middleware to serve files from within a given root directory.
    /// The file to serve will be determined by combining the requested Url with
    /// the provided root directory.
    ///
    ///
    /// # Examples
    /// ```{rust}
    /// use nickel::{Nickel, StaticFilesHandler};
    /// let mut server = Nickel::new();
    ///
    /// server.utilize(StaticFilesHandler::new("/path/to/serve/"));
    /// ```
    pub fn new<P: AsRef<Path>>(root_path: P) -> StaticFilesHandler {
        StaticFilesHandler {
            root_path: root_path.as_ref().to_path_buf()
        }
    }

    fn extract_path<'a, D>(&self, req: &'a mut Request<D>) -> Option<&'a str> {
        req.path_without_query().map(|path| {
            debug!("{:?} {:?}{:?}", req.origin.method, self.root_path.display(), path);

            match path {
                "/" => "index.html",
                path => &path[1..],
            }
        })
    }

    fn get_mime_by_filename<P: AsRef<Path>>(&self, path : P) -> &'static str {
        let _path = path.as_ref().to_str().unwrap();
        if _path.ends_with(".jpg") || _path.ends_with(".jpeg"){
            "image/jpeg"
        }else if _path.ends_with(".png"){
            "image/png"
        }else if _path.ends_with(".gif"){
            "image/gif"
        }else if _path.ends_with(".pdf"){
            "application/pdf"
        }else if _path.ends_with(".rtf"){
            "application/rtf"
        }else if _path.ends_with(".json"){
            "application/json"
        }else if _path.ends_with(".zip"){
            "application/x-zip-compressed"
        }else if _path.ends_with(".rar"){
            "application/x-compressed"
        }else if _path.ends_with(".gz") || _path.ends_with(".gzip"){
            "application/x-gzip"
        }else if _path.ends_with(".doc") || _path.ends_with(".docx"){
            "application/msword"
        }else if _path.ends_with(".xls") || _path.ends_with(".xlsx") || _path.ends_with(".xlt"){
            "application/excel"
        }else if _path.ends_with(".xml"){
            "text/xml"
        }else if _path.ends_with(".mov"){
            "video/quicktime"
        }else if _path.ends_with(".mp3"){
            "audio/mp3"
        }else{
            "application/octet-stream"
        }
    }

    fn with_file<'a, 'b, D, P>(&self,
                            relative_path: Option<P>,
                            mut res: Response<'a, D>)
            -> MiddlewareResult<'a, D> where P: AsRef<Path> {

        if let Some(path) = relative_path {
            let path = path.as_ref();
            if !safe_path(path) {
                let log_msg = format!("The path '{:?}' was denied access.", path);
                return res.error(StatusCode::BadRequest, log_msg);
            }

            let path = self.root_path.join(path);
            match fs::metadata(&path) {
                Ok(ref attr) if attr.is_file() => {

                    // res.set(MediaType::new("image".to_string(), "jpeg".to_string()));

                    let mime = self.get_mime_by_filename(&path);

                    res.headers_mut().set_raw("Content-Type", vec![mime.as_bytes().to_vec()]);

                    res.set(Connection::close());
                    return res.send_file(&path);
                },
                Err(ref e) if e.kind() != NotFound => debug!("Error getting metadata \
                                                              for file '{:?}': {:?}",
                                                              path, e),
                _ => {}
            }
        };

        res.next_middleware()
    }
}

/// Block paths from accessing the parent directory
fn safe_path<P: AsRef<Path>>(path: P) -> bool {
    use std::path::Component;

    path.as_ref().components().all(|c| match c {
        // whitelist non-suspicious in case new things get added in future
        Component::CurDir | Component::Normal(_) => true,
        _ => false
    })
}

#[test]
fn bad_paths() {
    let bad_paths = &[
        "foo/bar/../baz/index.html",
        "foo/bar/../baz",
        "../bar/",
        "..",
        "/" // Root path should be handled already
    ];

    for &path in bad_paths {
        assert!(!safe_path(path), "expected {:?} to be suspicious", path);
    }
}

#[test]
fn valid_paths() {
    let good_paths = &[
        "foo/bar/./baz/index.html",
        "foo/bar/./baz",
        "./bar/",
        ".",
        "index.html"
    ];

    for &path in good_paths {
        assert!(safe_path(path), "expected {:?} to not be suspicious", path);
    }
}
