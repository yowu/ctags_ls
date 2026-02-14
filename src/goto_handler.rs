use std::{
    collections::HashMap,
    fs::File,
    io::{self, BufRead},
};

use lsp_server::{Message, Request, Response};
use lsp_types::{GotoDefinitionParams, GotoDefinitionResponse, Location, Position, Range, Url};

use crate::{
    ctags::{CtagsEntry, CtagsHandler},
    document::DocumentsCache,
    logger::Logger,
    workspace::Workspace,
    queries::SymbolType,
    LspServer,
};

/// Filter ctags entries based on the syntactic role of the symbol under cursor.
/// Maps each SymbolType to the set of ctags kinds that are relevant for that context.
/// Returns entries unchanged for ambiguous contexts (Variable, Field, Parameter, Function).
pub fn refine_by_symbol_type(entries: Vec<CtagsEntry>, symbol_type: &SymbolType) -> Vec<CtagsEntry> {
    let allowed_kinds: &[&str] = match symbol_type {
        SymbolType::Type => &[
            "c", "class", "s", "struct", "t", "typedef", "g", "enum", "u", "union", "n",
            "namespace",
        ],
        SymbolType::Class => &["c", "class", "s", "struct"],
        SymbolType::FunctionCall => &["f", "function", "p", "prototype"],
        SymbolType::MethodCall => &["f", "function", "p", "prototype"],
        SymbolType::FieldAccess => &["m", "member"],
        SymbolType::Scope => &["c", "class", "s", "struct", "t", "typedef", "g", "enum", "u", "union", "n", "namespace"],
        _ => return entries,
    };

    entries
        .into_iter()
        .filter(|entry| allowed_kinds.contains(&entry.kind.as_str()))
        .collect()
}

/// Resolve ctags entries to LSP Locations by scanning source files
/// for matching patterns.
fn find_tags_location(entries: &[CtagsEntry], locations: &mut Vec<Location>) -> io::Result<()> {
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

        let doc = documents.get(&uri).ok_or_else(|| {
            Logger::error(&format!("Document not found: {:?}", uri));
            io::Error::new(io::ErrorKind::NotFound, "Document not found")
        })?;

        let symbol = doc.get_symbol_at_position(position)?;
        let symbol_type = doc.query_symbol_type(position);
        Logger::info(&format!(
            "Symbol '{}' analyzed as {:?}",
            symbol, symbol_type
        ));

        let entries = CtagsHandler::query_ctags(workspaces, &symbol)?;

        // Apply handler-specific filter (definition/declaration/implementation)
        let filtered_entries: Vec<CtagsEntry> = entries
            .into_iter()
            .filter(|entry| self.filter(entry))
            .collect();

        // Apply symbol-type-based refinement if available, with fallback
        let final_entries = if let Some(ref st) = symbol_type {
            let refined = refine_by_symbol_type(filtered_entries.clone(), st);
            if refined.is_empty() {
                Logger::info(&format!(
                    "Symbol type refinement for {:?} produced no results, falling back",
                    st
                ));
                filtered_entries
            } else {
                refined
            }
        } else {
            filtered_entries
        };

        let mut locations: Vec<Location> = Vec::new();
        find_tags_location(&final_entries, &mut locations)?;
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

        let workspace_manager = server.workspace_manager.lock().unwrap();
        let documents = server.documents.lock().unwrap();
        let response = self.handle_goto(&workspace_manager.workspaces, params, &documents)?;

        let resp = Response::new_ok(req.id.clone(), response);
        server
            .connection
            .sender
            .send(Message::Response(resp))
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
        Ok(())
    }
}
