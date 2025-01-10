use std::{
    io::{self},
    process::Command,
};

use crate::{logger::Logger, utils::Workspace};

#[derive(Debug)]
pub struct CtagsEntry {
    pub name: String,
    pub file: String,
    pub pattern: String,
    pub kind: String,
}

pub struct CtagsHandler;

impl CtagsHandler {
    pub fn query_ctags(workspaces: &Vec<Workspace>, symbol: &str) -> io::Result<Vec<CtagsEntry>> {
        for workspace in workspaces {
            if let Some(tags_path) = &workspace.tag_file_path {
                let output = Command::new("readtags")
                    .arg("-t")
                    .arg(tags_path)
                    .arg("-e")
                    .arg(symbol)
                    .output()
                    .map_err(|e| {
                        Logger::error(&format!("Failed to execute readtags: {:?}", e));
                        io::Error::new(io::ErrorKind::Other, "Failed to execute readtags")
                    })?;
                let stdout = String::from_utf8(output.stdout).map_err(|e| {
                    Logger::error(&format!("Invalid UTF-8 in readtags output: {:?}", e));
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Invalid UTF-8 in readtags output",
                    )
                })?;
                let entries: Vec<CtagsEntry> = stdout
                    .lines()
                    .filter_map(|line| CtagsHandler::parse_tag(line, workspace))
                    .collect();

                return Ok(entries);
            }
        }

        Ok(Vec::new())
    }

    fn parse_tag(line: &str, workspace: &Workspace) -> Option<CtagsEntry> {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() < 4 {
            return None;
        }

        let pattern =
            parts[2].trim_matches(|c| c == '/' || c == '^' || c == '$' || c == ';' || c == '"');

        let kind_parts: Vec<&str> = parts[3].split(':').collect();
        if kind_parts.len() < 2 {
            return None;
        }

        Some(CtagsEntry {
            name: parts[0].to_string(),
            file: format!(
                "{}/{}",
                workspace.folder.uri.to_file_path().unwrap().display(),
                parts[1].to_string()
            ),
            pattern: pattern.to_string(),
            kind: kind_parts[1].to_string(),
        })
    }
}
