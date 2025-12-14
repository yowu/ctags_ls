use std::{io, path::PathBuf};

use lsp_types::Uri;
use url::Url;

pub struct UriUtils;

impl UriUtils {
    /// Convert a file path to a URI using url::Url::from_file_path
    pub fn file_path_to_uri<P: AsRef<std::path::Path>>(path: P) -> io::Result<Uri> {
        let url = Url::from_file_path(path)
            .map_err(|_| io::Error::other("Failed to convert path to URI"))?;

        url.as_str()
            .parse()
            .map_err(|e| io::Error::other(format!("Failed to parse URI: {}", e)))
    }

    /// Convert a URI to a file path using url::Url::to_file_path
    pub fn uri_to_file_path(uri: &Uri) -> io::Result<PathBuf> {
        let url = Url::parse(uri.as_str())
            .map_err(|e| io::Error::other(format!("Failed to parse URI: {}", e)))?;

        url.to_file_path()
            .map_err(|_| io::Error::other("Failed to convert URI to file path"))
    }
}
