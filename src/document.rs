use std::collections::HashMap;

use lsp_types::{TextDocumentContentChangeEvent, Url};

use crate::syntax::{
    adapter::{detect_language, parser_language, SyntaxLanguage},
    parse_state::ParseState,
};

pub struct TextDocument {
    text: String,
    language: SyntaxLanguage,
    parse_state: Option<ParseState>,
}

impl TextDocument {
    pub fn new(uri: &Url, text: String) -> Self {
        let language = detect_language(uri);
        let parse_state = parser_language(language).map(|lang| ParseState::new(lang, &text));
        Self {
            text,
            language,
            parse_state,
        }
    }

    pub fn apply_changes(&mut self, changes: Vec<TextDocumentContentChangeEvent>) {
        let text_before = self.text.clone();

        for change in &changes {
            if let Some(range) = change.range {
                let start_byte = ParseState::position_to_offset(&self.text, range.start);
                let end_byte = ParseState::position_to_offset(&self.text, range.end);

                self.text.replace_range(start_byte..end_byte, &change.text);
            } else {
                self.text = change.text.clone();
            }
        }

        if let Some(parse_state) = &mut self.parse_state {
            parse_state.apply_changes(&text_before, &self.text, &changes);
        }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn language(&self) -> SyntaxLanguage {
        self.language
    }

    pub(crate) fn parse_state(&self) -> Option<&ParseState> {
        self.parse_state.as_ref()
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
