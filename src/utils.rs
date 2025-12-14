use std::{io, path::PathBuf};

use lsp_types::Uri;

/// Utility functions for handling URI and path conversions
pub struct UriUtils;

impl UriUtils {
    pub fn file_path_to_uri<P: AsRef<str>>(path: P) -> io::Result<Uri> {
        let uri_string = format!("file://{}", path.as_ref());
        uri_string
            .parse()
            .map_err(|e| io::Error::other(format!("Failed to parse URI: {}", e)))
    }

    pub fn uri_to_file_path(uri: &Uri) -> io::Result<PathBuf> {
        let uri_str = uri.as_str();
        if let Some(stripped) = uri_str.strip_prefix("file://") {
            Ok(PathBuf::from(stripped))
        } else {
            Err(io::Error::other(format!(
                "URI is not a file URI: {}",
                uri_str
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_path_to_uri() {
        let path = "/home/user/test.txt";
        let uri = UriUtils::file_path_to_uri(path).unwrap();
        assert_eq!(uri.as_str(), "file:///home/user/test.txt");
    }

    #[test]
    fn test_uri_to_file_path() {
        let uri: Uri = "file:///home/user/test.txt".parse().unwrap();
        let path = UriUtils::uri_to_file_path(&uri).unwrap();
        assert_eq!(path, PathBuf::from("/home/user/test.txt"));
    }
}
