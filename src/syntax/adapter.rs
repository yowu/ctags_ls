use lsp_types::Url;
use tree_sitter::Language;

use crate::syntax::{
    cpp::queries::{CPP_QUERIES, LOCAL_DECL_QUERIES},
    query::QuerySpec,
};

pub trait LanguageAdapter {
    fn language() -> Language;
    fn symbol_queries() -> &'static [QuerySpec];
    fn local_decl_queries() -> &'static [QuerySpec];
}

pub struct CppAdapter;

impl LanguageAdapter for CppAdapter {
    fn language() -> Language {
        tree_sitter_cpp::LANGUAGE.into()
    }

    fn symbol_queries() -> &'static [QuerySpec] {
        &CPP_QUERIES
    }

    fn local_decl_queries() -> &'static [QuerySpec] {
        &LOCAL_DECL_QUERIES
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyntaxLanguage {
    Cpp,
    Unsupported,
}

pub fn detect_language(uri: &Url) -> SyntaxLanguage {
    let Some(path) = uri.path_segments().and_then(|segments| segments.last()) else {
        return SyntaxLanguage::Unsupported;
    };

    let ext = path.rsplit('.').next().unwrap_or_default().to_ascii_lowercase();
    match ext.as_str() {
        "c" | "cc" | "cpp" | "cxx" | "h" | "hh" | "hpp" | "hxx" => SyntaxLanguage::Cpp,
        _ => SyntaxLanguage::Unsupported,
    }
}

pub fn parser_language(language: SyntaxLanguage) -> Option<Language> {
    match language {
        SyntaxLanguage::Cpp => Some(CppAdapter::language()),
        SyntaxLanguage::Unsupported => None,
    }
}

pub fn symbol_queries(language: SyntaxLanguage) -> &'static [QuerySpec] {
    match language {
        SyntaxLanguage::Cpp => CppAdapter::symbol_queries(),
        SyntaxLanguage::Unsupported => &[],
    }
}

pub fn local_decl_queries(language: SyntaxLanguage) -> &'static [QuerySpec] {
    match language {
        SyntaxLanguage::Cpp => CppAdapter::local_decl_queries(),
        SyntaxLanguage::Unsupported => &[],
    }
}
