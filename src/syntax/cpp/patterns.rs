pub const FUNCTION_DEFINITION: &str = r#"(function_definition
              declarator: (function_declarator
                  declarator: (identifier) @function_name))"#;

pub const CLASS_SPECIFIER: &str = r#"(class_specifier
              name: (type_identifier) @class_name)"#;

pub const STRUCT_SPECIFIER: &str = r#"(struct_specifier
              name: (type_identifier) @struct_name)"#;

pub const QUALIFIED_SCOPE: &str = r#"(qualified_identifier
                scope: (namespace_identifier) @scope_name)"#;

pub const METHOD_CALL: &str = r#"(call_expression
              function: (field_expression
                  field: (field_identifier) @function_call))"#;

pub const DECLARATION_INIT_IDENT: &str = r#"(declaration
              declarator: (init_declarator
                  declarator: (identifier) @variable_name))"#;

pub const FUNCTION_CALL: &str = r#"(call_expression
              function: (identifier) @function_call)"#;

pub const TYPE_ANY: &str = r#"[
              (type_identifier) @type
              (primitive_type) @type
          ]"#;

pub const PARAM_DECL_ANY: &str = r#"[
              (parameter_declaration
                  declarator: (identifier) @param)
              (parameter_declaration
                  declarator: (pointer_declarator
                      declarator: (identifier) @param))
          ]"#;

pub const PARAM_DECL_REF_CAPTURE: &str = r#"(parameter_declaration
              declarator: (reference_declarator (identifier) @param))"#;

pub const FIELD_DECLARATION: &str = r#"(field_declaration
              declarator: (field_identifier) @field_name)"#;

pub const FIELD_ACCESS: &str = r#"[
              (field_expression
                  field: (field_identifier) @field)
              (field_identifier) @field
          ]"#;

pub const PARAM_DECL_IDENTIFIER: &str = r#"(parameter_declaration
                declarator: (identifier) @decl)"#;

pub const PARAM_DECL_POINTER: &str = r#"(parameter_declaration
                declarator: (pointer_declarator
                    declarator: (identifier) @decl))"#;

pub const PARAM_DECL_REFERENCE: &str = r#"(parameter_declaration
                declarator: (reference_declarator (identifier) @decl))"#;

pub const LOCAL_DECLARATION_INIT_IDENT: &str = r#"(declaration
                declarator: (init_declarator
                    declarator: (identifier) @decl))"#;

pub const LOCAL_DECLARATION_IDENT: &str = r#"(declaration
                declarator: (identifier) @decl)"#;

pub const LOCAL_FUNCTION_DECL: &str = r#"(function_definition
                declarator: (function_declarator
                    declarator: (identifier) @decl))"#;

pub const LOCAL_FIELD_DECL: &str = r#"(field_declaration
                declarator: (field_identifier) @decl)"#;
