use std::io;

use lsp_server::{Message, Request, Response};
use lsp_types::{
    DeclarationCapability, ImplementationProviderCapability, InitializeParams, InitializeResult,
    ServerCapabilities, TextDocumentSyncCapability, TextDocumentSyncKind, WorkspaceFolder,
};

use crate::{logger::Logger, workspace::WorkspaceManager, LspServer};

pub struct InitializeHandler;

impl InitializeHandler {
    fn initialize_workspaces(params: &InitializeParams) -> WorkspaceManager {
        let tag_patterns = params
            .initialization_options
            .as_ref()
            .and_then(|options| options.get("tags"))
            .and_then(|tags| serde_json::from_value(tags.clone()).ok())
            .unwrap_or_else(|| vec!["tags".to_string()]);

        Logger::info(&format!("Initialize tag patterns: {:?}", tag_patterns));
        let mut manager = WorkspaceManager::new(tag_patterns);

        if let Some(folders) = params.workspace_folders.as_ref() {
            for folder in folders {
                manager.add_workspace(folder);
            }
        } else if let Some(root_uri) = params.root_uri.as_ref() {
            let folder = WorkspaceFolder {
                uri: root_uri.clone(),
                name: "root".to_string(),
            };
            manager.add_workspace(&folder);
        }
        manager
    }

    pub fn handle(&self, req: Request, server: &LspServer) -> io::Result<()> {
        Logger::info("Received initialize request");
        let params: InitializeParams = serde_json::from_value(req.params)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;

        let mut manager = server.workspace_manager.lock().unwrap();
        *manager = InitializeHandler::initialize_workspaces(&params);
        Logger::info(&format!(
            "Initializing {} workspaces",
            manager.workspaces.len()
        ));

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
