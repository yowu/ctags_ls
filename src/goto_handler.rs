use std::{
    collections::HashMap,
    fs::File,
    io::{self, BufRead},
};

use lsp_server::{Message, Request, Response};
use lsp_types::{GotoDefinitionParams, GotoDefinitionResponse, Location, Position, Range, Url};

use crate::LspServer;
use crate::{
    ctags::{CtagsEntry, CtagsHandler},
    logger::Logger,
    utils::{DocumentsCache, Workspace},
};

fn find_tags_location(entries: &Vec<CtagsEntry>, locations: &mut Vec<Location>) -> io::Result<()> {
    // Group entries by file to minimize file reads
    let mut file_to_entries: HashMap<String, Vec<&CtagsEntry>> = HashMap::new();
    for entry in entries {
        file_to_entries
            .entry(entry.file.clone())
            .or_default()
            .push(entry);
    }

    for (file_path, entries) in file_to_entries {
        let file = File::open(&file_path)?;
        let reader = io::BufReader::new(file);

        let mut found = vec![false; entries.len()];
        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            for (idx, entry) in entries.iter().enumerate() {
                if found[idx] {
                    continue;
                }
                if line.contains(&entry.pattern) {
                    if let Some(character) = line.find(&entry.name) {
                        locations.push(Location {
                            uri: Url::parse(&format!("file://{}", entry.file))
                                .expect("Failed to parse URL"),
                            range: Range {
                                start: Position {
                                    line: line_num as u32,
                                    character: character as u32,
                                },
                                end: Position {
                                    line: line_num as u32,
                                    character: (character + entry.name.len()) as u32,
                                },
                            },
                        });
                        found[idx] = true;
                        break;
                    }
                }
            }

            if found.iter().all(|&f| f) {
                break;
            }
        }
    }

    Ok(())
}

pub trait GotoHandler {
    fn filter(&self, entry: &CtagsEntry) -> bool;

    fn handle_goto(
        &self,
        workspaces: &Vec<Workspace>,
        params: GotoDefinitionParams,
        documents: &DocumentsCache,
    ) -> io::Result<GotoDefinitionResponse> {
        let position = params.text_document_position_params.position;
        let uri = params.text_document_position_params.text_document.uri;

        let symbol = documents
            .get(&uri)
            .ok_or_else(|| {
                Logger::error(&format!("Document not found: {:?}", uri));
                io::Error::new(io::ErrorKind::NotFound, "Document not found")
            })?
            .get_symbol_at_position(position)?;
        let entries = CtagsHandler::query_ctags(&workspaces, &symbol)?;
        let mut locations: Vec<Location> = Vec::new();
        find_tags_location(
            &entries
                .into_iter()
                .filter(|entry| self.filter(entry))
                .collect(),
            &mut locations,
        )?;
        Logger::info(&format!(
            "Found {} locations for symbol: {}",
            locations.len(),
            symbol
        ));
        Ok(GotoDefinitionResponse::Array(locations))
    }

    fn handle(&self, req: Request, server: &LspServer) -> io::Result<()> {
        Logger::info(&format!("Received request: {:?}", req.method));
        let params: GotoDefinitionParams = serde_json::from_value(req.params)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;

        let workspaces = server.workspaces.lock().unwrap();
        let documents = server.documents.lock().unwrap();
        let response = self.handle_goto(&workspaces, params, &documents)?;

        let resp = Response::new_ok(req.id.clone(), response);
        server
            .connection
            .sender
            .send(Message::Response(resp))
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
        Ok(())
    }
}
