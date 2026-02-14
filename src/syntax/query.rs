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

#[derive(Debug, Clone, Copy)]
pub enum QueryMetadata {
    SymbolType(SymbolType),
    LocalWeight(u8),
}

pub struct QuerySpec {
    pub query: Query,
    metadata: QueryMetadata,
}

impl QuerySpec {
    pub(crate) fn with_symbol_type(query: Query, symbol_type: SymbolType) -> Self {
        Self {
            query,
            metadata: QueryMetadata::SymbolType(symbol_type),
        }
    }

    pub(crate) fn with_local_weight(query: Query, weight: u8) -> Self {
        Self {
            query,
            metadata: QueryMetadata::LocalWeight(weight),
        }
    }

    pub fn symbol_type(&self) -> Option<SymbolType> {
        match self.metadata {
            QueryMetadata::SymbolType(symbol_type) => Some(symbol_type),
            QueryMetadata::LocalWeight(_) => None,
        }
    }

    pub fn weight(&self) -> Option<u8> {
        match self.metadata {
            QueryMetadata::LocalWeight(weight) => Some(weight),
            QueryMetadata::SymbolType(_) => None,
        }
    }
}
