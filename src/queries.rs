use lazy_static::lazy_static;
use tree_sitter::Query;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum SymbolType {
    Class,
    Field,
    FieldAccess,
    Function,
    FunctionCall,
    MethodCall,
    Parameter,
    Type,
    Variable,
    Scope,
}

pub struct PreparedQuery {
    pub query: Query,
    pub symbol_type: SymbolType,
}

fn build_cpp_queries() -> Vec<PreparedQuery> {
    let language: tree_sitter::Language = tree_sitter_cpp::LANGUAGE.into();

    let patterns: Vec<(SymbolType, &str)> = vec![
        (
            SymbolType::Function,
            r#"(function_definition
              declarator: (function_declarator
                  declarator: (identifier) @function_name))"#,
        ),
        (
            SymbolType::Class,
            r#"(class_specifier
              name: (type_identifier) @class_name)"#,
        ),
        (
            SymbolType::Class,
            r#"(struct_specifier
              name: (type_identifier) @struct_name)"#,
        ),
        (
            SymbolType::Scope,
            r#"(qualified_identifier
                scope: (namespace_identifier) @scope_name)"#,
        ),
        (
            SymbolType::MethodCall,
            r#"(call_expression
              function: (field_expression
                  field: (field_identifier) @function_call))"#,
        ),
        (
            SymbolType::Variable,
            r#"(declaration
              declarator: (init_declarator
                  declarator: (identifier) @variable_name))"#,
        ),
        (
            SymbolType::FunctionCall,
            r#"(call_expression
              function: (identifier) @function_call)"#,
        ),
        (
            SymbolType::MethodCall,
            r#"(call_expression
              function: (field_expression
                  field: (field_identifier) @function_call))"#,
        ),
        (
            SymbolType::Type,
            r#"[
              (type_identifier) @type
              (primitive_type) @type
          ]"#,
        ),
        (
            SymbolType::Parameter,
            r#"[
              (parameter_declaration
                  declarator: (identifier) @param)
              (parameter_declaration
                  declarator: (pointer_declarator
                      declarator: (identifier) @param))
          ]"#,
        ),
        (
            SymbolType::Parameter,
            r#"(parameter_declaration
              declarator: (reference_declarator (identifier) @param))"#,
        ),
        (
            SymbolType::Field,
            r#"(field_declaration
              declarator: (field_identifier) @field_name)"#,
        ),
        (
            SymbolType::FieldAccess,
            r#"[
              (field_expression
                  field: (field_identifier) @field)
              (field_identifier) @field
          ]"#,
        ),
    ];

    patterns
        .into_iter()
        .map(|(symbol_type, pattern)| {
            let query =
                Query::new(&language, pattern).expect("Failed to compile tree-sitter query");
            PreparedQuery { query, symbol_type }
        })
        .collect()
}

lazy_static! {
    pub static ref CPP_QUERIES: Vec<PreparedQuery> = build_cpp_queries();
}
