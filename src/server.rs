use std::{
    io,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
};

use lsp_server::{Connection, Message, Request, Response};
use lsp_types::{
    DidChangeTextDocumentParams, DidChangeWorkspaceFoldersParams, DidCloseTextDocumentParams,
    DidOpenTextDocumentParams,
};

use crate::{
    ctags::CtagsEntry,
    document::{DocumentsCache, TextDocument},
    goto_handler::GotoHandler,
    initialize_handler::InitializeHandler,
    logger::Logger,
    workspace::WorkspaceManager,
};

pub struct LspServer {
    pub connection: Connection,
    pub documents: Mutex<DocumentsCache>,
    pub workspace_manager: Mutex<WorkspaceManager>,
    shutdown_requested: Arc<AtomicBool>,
}

pub struct GotoDefinitionHandler;
impl GotoHandler for GotoDefinitionHandler {
    fn filter(&self, entry: &CtagsEntry) -> bool {
        !matches!(entry.kind.as_str(), "p" | "prototype")
    }
}

pub struct GotoDeclarationHandler;
impl GotoHandler for GotoDeclarationHandler {
    fn filter(&self, entry: &CtagsEntry) -> bool {
        matches!(entry.kind.as_str(), "p" | "prototype")
    }
}

pub struct GotoImplementationHandler;
impl GotoHandler for GotoImplementationHandler {
    fn filter(&self, entry: &CtagsEntry) -> bool {
        matches!(entry.kind.as_str(), "f" | "function")
    }
}

impl LspServer {
    pub fn new(connection: Connection) -> Self {
        Self {
            connection,
            documents: Mutex::new(DocumentsCache::new()),
            workspace_manager: Mutex::new(WorkspaceManager::new(vec!["tags".to_string()])),
            shutdown_requested: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn run(&self) -> io::Result<()> {
        for msg in &self.connection.receiver {
            if self.shutdown_requested.load(Ordering::SeqCst) {
                Logger::info("Shutdown requested, exiting...");
                break;
            }
            match msg {
                Message::Request(req) => {
                    if let Err(e) = self.handle_request(req) {
                        Logger::error(&format!("Failed to handle request: {:?}", e));
                    }
                }
                Message::Response(_) => {}
                Message::Notification(notif) => {
                    if let Err(e) = self.handle_notification(notif) {
                        Logger::error(&format!("Failed to handle notification: {:?}", e));
                    }
                }
            }
        }
        Ok(())
    }

    fn handle_request(&self, req: Request) -> io::Result<()> {
        match req.method.as_str() {
            "initialize" => InitializeHandler.handle(req, self),
            "textDocument/definition" => GotoDefinitionHandler.handle(req, self),
            "textDocument/declaration" => GotoDeclarationHandler.handle(req, self),
            "textDocument/implementation" => GotoImplementationHandler.handle(req, self),
            "shutdown" => {
                self.shutdown_requested.store(true, Ordering::SeqCst);
                let resp = Response::new_ok(req.id.clone(), ());
                self.connection
                    .sender
                    .send(Message::Response(resp))
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
                Ok(())
            }
            _ => {
                Logger::info(&format!("Received unhandled request: {:?}", req.method));
                Ok(())
            }
        }
    }

    fn handle_notification(&self, notif: lsp_server::Notification) -> io::Result<()> {
        match notif.method.as_str() {
            "textDocument/didOpen" => {
                let params: DidOpenTextDocumentParams = serde_json::from_value(notif.params)
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
                let mut documents = self.documents.lock().unwrap();
                documents.insert(
                    params.text_document.uri,
                    TextDocument::new(params.text_document.text),
                );
            }
            "textDocument/didChange" => {
                let params: DidChangeTextDocumentParams = serde_json::from_value(notif.params)
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
                let mut documents = self.documents.lock().unwrap();
                if let Some(doc) = documents.get_mut(&params.text_document.uri) {
                    doc.apply_changes(params.content_changes);
                }
            }
            "textDocument/didClose" => {
                let params: DidCloseTextDocumentParams = serde_json::from_value(notif.params)
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
                let mut documents = self.documents.lock().unwrap();
                documents.remove(&params.text_document.uri);
            }
            "workspace/didChangeWorkspaceFolders" => {
                let params: DidChangeWorkspaceFoldersParams = serde_json::from_value(notif.params)?;
                let mut manager = self.workspace_manager.lock().unwrap();

                for folder in &params.event.removed {
                    manager.remove_workspace(folder);
                }

                for folder in &params.event.added {
                    manager.add_workspace(folder);
                }
            }
            _ => {
                Logger::info(&format!(
                    "Received unhandled notification: {:?}",
                    notif.method
                ));
            }
        }
        Ok(())
    }
}
