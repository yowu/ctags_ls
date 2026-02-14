use lsp_types::{Location, Position, Range, Url};
use streaming_iterator::StreamingIterator;

use crate::{
    document::TextDocument,
    syntax::{parse_state::ParseState, query::{QuerySpec, SymbolType}},
};

#[derive(Clone, Copy)]
struct Candidate {
    byte: usize,
    position: Position,
    weight: u8,
}

pub struct LocalSymbolResolver;

impl LocalSymbolResolver {
    pub fn resolve(
        document: &TextDocument,
        uri: &Url,
        symbol: &str,
        cursor: Position,
        symbol_type: Option<&SymbolType>,
        local_queries: &[QuerySpec],
    ) -> Option<Location> {
        let tree = document.parse_state()?.tree()?;
        let source = document.text();
        let source_bytes = source.as_bytes();
        let cursor_byte = ParseState::position_to_offset(source, cursor);

        let function_scope = Self::function_scope_range(document, cursor_byte);
        let mut best: Option<Candidate> = None;

        for query in local_queries.iter() {
            let mut query_cursor = tree_sitter::QueryCursor::new();
            let mut matches = query_cursor.matches(&query.query, tree.root_node(), source_bytes);

            while let Some(m) = matches.next() {
                for capture in m.captures.iter() {
                    let node = capture.node;
                    let Ok(text) = node.utf8_text(source_bytes) else {
                        continue;
                    };
                    if text != symbol {
                        continue;
                    }

                    let start_byte = node.start_byte();
                    if !Self::scope_eligible(symbol_type, function_scope, start_byte) {
                        continue;
                    }

                    let start = node.start_position();
                    let candidate = Candidate {
                        byte: start_byte,
                        position: Position {
                            line: start.row as u32,
                            character: start.column as u32,
                        },
                        weight: query.weight().unwrap_or(0),
                    };

                    if Self::is_better(candidate, best, cursor_byte) {
                        best = Some(candidate);
                    }
                }
            }
        }

        best.map(|candidate| Location {
            uri: uri.clone(),
            range: Range {
                start: candidate.position,
                end: Position {
                    line: candidate.position.line,
                    character: candidate.position.character + symbol.len() as u32,
                },
            },
        })
    }

    fn function_scope_range(document: &TextDocument, cursor_byte: usize) -> Option<(usize, usize)> {
        let tree = document.parse_state()?.tree()?;
        let mut node = tree.root_node().descendant_for_byte_range(cursor_byte, cursor_byte)?;

        loop {
            if node.kind() == "function_definition" {
                return Some((node.start_byte(), node.end_byte()));
            }

            let Some(parent) = node.parent() else {
                break;
            };
            node = parent;
        }

        None
    }

    fn scope_eligible(
        symbol_type: Option<&SymbolType>,
        function_scope: Option<(usize, usize)>,
        declaration_byte: usize,
    ) -> bool {
        match symbol_type {
            Some(SymbolType::Variable) | Some(SymbolType::Parameter) => {
                if let Some((start, end)) = function_scope {
                    declaration_byte >= start && declaration_byte <= end
                } else {
                    true
                }
            }
            _ => true,
        }
    }

    fn is_better(candidate: Candidate, current: Option<Candidate>, cursor_byte: usize) -> bool {
        let Some(current) = current else {
            return true;
        };

        if candidate.weight != current.weight {
            return candidate.weight > current.weight;
        }

        let candidate_before = candidate.byte <= cursor_byte;
        let current_before = current.byte <= cursor_byte;

        if candidate_before != current_before {
            return candidate_before;
        }

        if candidate_before {
            candidate.byte > current.byte
        } else {
            candidate.byte < current.byte
        }
    }
}
