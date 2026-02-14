use lazy_static::lazy_static;
use tree_sitter::Query;

use crate::syntax::{
    cpp::patterns,
    query::{QuerySpec, SymbolType},
};

fn build_cpp_queries() -> Vec<QuerySpec> {
    let language: tree_sitter::Language = tree_sitter_cpp::LANGUAGE.into();

    let patterns: Vec<(SymbolType, &str)> = vec![
        (SymbolType::Function, patterns::FUNCTION_DEFINITION),
        (SymbolType::Class, patterns::CLASS_SPECIFIER),
        (SymbolType::Class, patterns::STRUCT_SPECIFIER),
        (SymbolType::Scope, patterns::QUALIFIED_SCOPE),
        (SymbolType::MethodCall, patterns::METHOD_CALL),
        (SymbolType::Variable, patterns::DECLARATION_INIT_IDENT),
        (SymbolType::FunctionCall, patterns::FUNCTION_CALL),
        (SymbolType::Type, patterns::TYPE_ANY),
        (SymbolType::Parameter, patterns::PARAM_DECL_ANY),
        (SymbolType::Parameter, patterns::PARAM_DECL_REF_CAPTURE),
        (SymbolType::Field, patterns::FIELD_DECLARATION),
        (SymbolType::FieldAccess, patterns::FIELD_ACCESS),
    ];

    patterns
        .into_iter()
        .map(|(symbol_type, pattern)| {
            let query =
                Query::new(&language, pattern).expect("Failed to compile tree-sitter query");
            QuerySpec::with_symbol_type(query, symbol_type)
        })
        .collect()
}

fn build_local_queries() -> Vec<QuerySpec> {
    let language: tree_sitter::Language = tree_sitter_cpp::LANGUAGE.into();

    let patterns: Vec<(&str, u8)> = vec![
        (patterns::PARAM_DECL_IDENTIFIER, 4),
        (patterns::PARAM_DECL_POINTER, 4),
        (patterns::PARAM_DECL_REFERENCE, 4),
        (patterns::LOCAL_DECLARATION_INIT_IDENT, 3),
        (patterns::LOCAL_DECLARATION_IDENT, 3),
        (patterns::LOCAL_FUNCTION_DECL, 2),
        (patterns::LOCAL_FIELD_DECL, 3),
    ];

    patterns
        .into_iter()
        .map(|(pattern, weight)| {
            let query =
                Query::new(&language, pattern).expect("Failed to compile local symbol query");
            QuerySpec::with_local_weight(query, weight)
        })
        .collect()
}

lazy_static! {
    pub static ref CPP_QUERIES: Vec<QuerySpec> = build_cpp_queries();
    pub static ref LOCAL_DECL_QUERIES: Vec<QuerySpec> = build_local_queries();
}
