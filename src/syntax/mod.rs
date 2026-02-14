use std::io;

use lsp_types::{Location, Position, Url};
use streaming_iterator::StreamingIterator;

use crate::{document::TextDocument, logger::Logger};

use self::adapter::{local_decl_queries, symbol_queries};
use self::local_resolver::LocalSymbolResolver;
use self::parse_state::ParseState;

pub mod adapter;
pub mod cpp;
pub mod query;
pub mod parse_state;
mod local_resolver;

pub use query::SymbolType;

pub struct SyntaxContext {
    pub symbol: String,
    pub symbol_type: Option<SymbolType>,
    pub local_definition: Option<Location>,
}

pub struct SyntaxEngine;

impl SyntaxEngine {
    pub fn analyze_at(
        document: &TextDocument,
        uri: &Url,
        position: Position,
    ) -> io::Result<SyntaxContext> {
        if document.parse_state().is_none() {
            Logger::info("Skipping tree-sitter syntax analysis for unsupported language");
        }

        let symbol = Self::symbol_at_position(document, position)?;
        let symbol_type = Self::query_symbol_type(document, position);
        let local_definition = LocalSymbolResolver::resolve(
            document,
            uri,
            &symbol,
            position,
            symbol_type.as_ref(),
            local_decl_queries(document.language()),
        );

        Ok(SyntaxContext {
            symbol,
            symbol_type,
            local_definition,
        })
    }

    fn symbol_at_position(document: &TextDocument, position: Position) -> io::Result<String> {
        let line = document
            .text()
            .lines()
            .nth(position.line as usize)
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Line number out of range"))?;

        let symbol_start = line[..position.character as usize]
            .rfind(|c: char| !c.is_alphanumeric() && c != '_')
            .map_or(0, |pos| pos + 1);
        let symbol_end = line[position.character as usize..]
            .find(|c: char| !c.is_alphanumeric() && c != '_')
            .map_or(line.len(), |pos| position.character as usize + pos);

        Ok(line[symbol_start..symbol_end].to_string())
    }

    fn query_symbol_type(document: &TextDocument, position: Position) -> Option<SymbolType> {
        let source = document.text();
        let tree = document.parse_state()?.tree()?;
        let byte_offset = ParseState::position_to_offset(source, position);

        for prepared_query in symbol_queries(document.language()).iter() {
            let mut cursor = tree_sitter::QueryCursor::new();
            let mut matches = cursor.matches(&prepared_query.query, tree.root_node(), source.as_bytes());

            while let Some(m) = matches.next() {
                for capture in m.captures.iter() {
                    let node = capture.node;
                    if node.start_byte() <= byte_offset && byte_offset < node.end_byte() {
                        return prepared_query.symbol_type();
                    }
                }
            }
        }

        None
    }
}
