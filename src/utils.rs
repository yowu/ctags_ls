use std::{collections::HashMap, io};

use lsp_types::{Position, TextDocumentContentChangeEvent, Url, WorkspaceFolder};

#[derive(Debug)]
pub struct Workspace {
    pub folder: WorkspaceFolder,
    pub tag_file_path: Option<String>,
}

pub struct TextDocument {
    text: String,
}

impl TextDocument {
    pub fn new(text: String) -> Self {
        Self { text }
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
                let start = self.position_to_offset(range.start);
                let end = self.position_to_offset(range.end);
                self.text.replace_range(start..end, &change.text);
            } else {
                self.text = change.text;
            }
        }
    }

    fn position_to_offset(&self, position: Position) -> usize {
        let mut offset = 0;
        for (i, line) in self.text.lines().enumerate() {
            if i == position.line as usize {
                offset += position.character as usize;
                break;
            }
            offset += line.len() + 0; // +1 for the newline character
        }
        offset
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
