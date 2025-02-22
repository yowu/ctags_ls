use std::path::Path;

use crate::logger::Logger;
use lsp_types::WorkspaceFolder;

#[derive(Debug)]
pub struct Workspace {
    pub folder: WorkspaceFolder,
    pub tag_file_path: Option<String>,
}

#[derive(Clone)]
pub struct WorkspaceConfig {
    pub tag_file_patterns: Vec<String>,
}

pub struct WorkspaceManager {
    pub workspaces: Vec<Workspace>,
    pub config: WorkspaceConfig,
}

impl WorkspaceManager {
    pub fn new(tag_patterns: Vec<String>) -> Self {
        Self {
            workspaces: Vec::new(),
            config: WorkspaceConfig {
                tag_file_patterns: tag_patterns,
            },
        }
    }

    pub fn add_workspace(&mut self, folder: &WorkspaceFolder) {
        if self.workspaces.iter().any(|w| w.folder.uri == folder.uri) {
            Logger::info(&format!("Workspace already exists: {}", folder.uri));
            return;
        }

        let folder_path = if let Ok(path) = folder.uri.to_file_path() {
            path
        } else {
            return;
        };

        let mut tag_file_path = None;
        for pattern in &self.config.tag_file_patterns {
            let tags_path = format!("{}/{}", folder_path.display(), pattern);
            if Path::new(&tags_path).exists() {
                tag_file_path = Some(tags_path);
                break;
            }
        }

        Logger::info(&format!(
            "Adding workspace: {:?} with tag file: {:?}",
            folder, tag_file_path
        ));

        self.workspaces.push(Workspace {
            folder: folder.clone(),
            tag_file_path,
        });
    }

    pub fn remove_workspace(&mut self, folder: &WorkspaceFolder) {
        self.workspaces.retain(|w| w.folder != *folder);
    }
}
