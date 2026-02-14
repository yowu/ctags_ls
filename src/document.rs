use std::{collections::HashMap, io};

use lsp_types::{Position, TextDocumentContentChangeEvent, Url};
use streaming_iterator::StreamingIterator;
use tree_sitter::{InputEdit, Parser, Point, QueryCursor, Tree};

use crate::queries::{CPP_QUERIES, SymbolType};

pub struct TextDocument {
    text: String,
    parser: Parser,
    tree: Option<Tree>,
}

impl TextDocument {
    pub fn new(text: String) -> Self {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_cpp::LANGUAGE.into())
            .expect("Failed to set C++ language for parser");
        let tree = parser.parse(&text, None);
        Self { text, parser, tree }
    }

    pub fn get_line(&self, line_number: usize) -> io::Result<String> {
        self.text
            .lines()
            .nth(line_number)
            .map(|line| line.to_string())
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Line number out of range"))
    }

    pub fn apply_changes(&mut self, changes: Vec<TextDocumentContentChangeEvent>) {
        for change in changes {
            if let Some(range) = change.range {
                let start_byte = self.position_to_offset(range.start);
                let end_byte = self.position_to_offset(range.end);

                let start_point = Point {
                    row: range.start.line as usize,
                    column: range.start.character as usize,
                };
                let old_end_point = Point {
                    row: range.end.line as usize,
                    column: range.end.character as usize,
                };

                let new_end_byte = start_byte + change.text.len();
                let new_end_point = Self::compute_end_point(start_point, &change.text);

                // Apply edit to tree-sitter tree for incremental re-parsing
                if let Some(tree) = &mut self.tree {
                    tree.edit(&InputEdit {
                        start_byte,
                        old_end_byte: end_byte,
                        new_end_byte,
                        start_position: start_point,
                        old_end_position: old_end_point,
                        new_end_position: new_end_point,
                    });
                }

                self.text.replace_range(start_byte..end_byte, &change.text);
            } else {
                // Full document replacement — discard old tree
                self.text = change.text;
                self.tree = None;
            }
        }
        // Re-parse: incremental if edited tree exists, full parse otherwise
        self.tree = self.parser.parse(&self.text, self.tree.as_ref());
    }

    fn position_to_offset(&self, position: Position) -> usize {
        let mut offset = 0;
        for (i, line) in self.text.lines().enumerate() {
            if i == position.line as usize {
                offset += position.character as usize;
                break;
            }
            offset += line.len() + 1; // +1 for the newline character
        }
        offset
    }

    /// Compute the end Point after inserting `new_text` starting at `start_point`.
    fn compute_end_point(start_point: Point, new_text: &str) -> Point {
        let mut row = start_point.row;
        let mut column = start_point.column;
        for byte in new_text.bytes() {
            if byte == b'\n' {
                row += 1;
                column = 0;
            } else {
                column += 1;
            }
        }
        Point { row, column }
    }

    pub fn get_symbol_at_position(&self, position: Position) -> io::Result<String> {
        let line = self.get_line(position.line as usize)?;
        let symbol_start = line[..position.character as usize]
            .rfind(|c: char| !c.is_alphanumeric() && c != '_')
            .map_or(0, |pos| pos + 1);
        let symbol_end = line[position.character as usize..]
            .find(|c: char| !c.is_alphanumeric() && c != '_')
            .map_or(line.len(), |pos| position.character as usize + pos);
        Ok(line[symbol_start..symbol_end].to_string())
    }

    /// Query the syntactic role of the symbol at the given position
    /// using the cached tree-sitter parse tree.
    pub fn query_symbol_type(&self, position: Position) -> Option<SymbolType> {
        let tree = self.tree.as_ref()?;
        let byte_offset = self.position_to_offset(position);

        for prepared_query in CPP_QUERIES.iter() {
            let mut cursor = QueryCursor::new();
            let mut matches = cursor.matches(
                &prepared_query.query,
                tree.root_node(),
                self.text.as_bytes(),
            );

            while let Some(m) = matches.next() {
                for capture in m.captures.iter() {
                    let node = capture.node;
                    if node.start_byte() <= byte_offset && byte_offset < node.end_byte() {
                        return Some(prepared_query.symbol_type);
                    }
                }
            }
        }
        None
    }
}

pub struct DocumentsCache {
    documents: HashMap<Url, TextDocument>,
}

impl DocumentsCache {
    pub fn new() -> Self {
        Self {
            documents: HashMap::new(),
        }
    }

    pub fn insert(&mut self, uri: Url, document: TextDocument) {
        self.documents.insert(uri, document);
    }

    pub fn remove(&mut self, uri: &Url) {
        self.documents.remove(uri);
    }

    pub fn get(&self, uri: &Url) -> Option<&TextDocument> {
        self.documents.get(uri)
    }

    pub fn get_mut(&mut self, uri: &Url) -> Option<&mut TextDocument> {
        self.documents.get_mut(uri)
    }
}
