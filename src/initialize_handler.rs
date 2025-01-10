use std::{io, path::Path};

use lsp_server::{Message, Request, Response};
use lsp_types::{
    DeclarationCapability, ImplementationProviderCapability, InitializeParams, InitializeResult,
    ServerCapabilities, TextDocumentSyncCapability, TextDocumentSyncKind,
};

use crate::{logger::Logger, utils::Workspace, LspServer};

pub struct InitializeHandler;

impl InitializeHandler {
    fn initialize_workspaces(params: InitializeParams, workspaces: &mut Vec<Workspace>) {
        let default_tags = vec!["tags".to_string()];
        let tags = if let Some(options) = params.initialization_options {
            if let Some(tags) = options.get("tags") {
                serde_json::from_value(tags.clone()).unwrap_or(default_tags)
            } else {
                default_tags
            }
        } else {
            default_tags
        };

        workspaces.clear();

        for folder in params.workspace_folders.unwrap_or_default() {
            let folder_path = folder.uri.to_file_path().unwrap();
            let mut tag_file_path = None;
            for tag in &tags {
                let tags_path = format!("{}/{}", folder_path.display(), tag);
                if Path::new(&tags_path).exists() {
                    tag_file_path = Some(tags_path);
                    break;
                }
            }
            workspaces.push(Workspace {
                folder,
                tag_file_path,
            });
        }
    }

    pub fn handle_request(&self, req: Request, server: &LspServer) -> io::Result<()> {
        Logger::info("Received initialize request");
        let params: InitializeParams = serde_json::from_value(req.params)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;

        InitializeHandler::initialize_workspaces(
            params,
            server.workspaces.lock().unwrap().as_mut(),
        );

        // check if the workspace has a tags file
        let workspaces = server.workspaces.lock().unwrap();
        let has_tags = workspaces
            .iter()
            .any(|workspace| workspace.tag_file_path.is_some());
        if !has_tags {
            Logger::error("No tags file found in workspace");
        }

        let kind = TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL);
        let server_capabilities = ServerCapabilities::default();
        let capabilities = ServerCapabilities {
            text_document_sync: Some(kind),
            definition_provider: Some(lsp_types::OneOf::Left(true)),
            declaration_provider: Some(DeclarationCapability::Simple(true)),
            implementation_provider: Some(ImplementationProviderCapability::Simple(true)),
            ..server_capabilities
        };

        let initialize_result = InitializeResult {
            capabilities,
            server_info: None,
        };

        let resp = Response::new_ok(req.id.clone(), initialize_result);
        server
            .connection
            .sender
            .send(Message::Response(resp))
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
        Ok(())
    }
}
