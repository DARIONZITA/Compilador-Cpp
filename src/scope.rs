use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum SymbolType {
    Int(&'static str),
    Float(&'static str),
    String(&'static str),
    Char(&'static str),
    Bool(&'static str),
    Double(&'static str),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum SymbolCategory {
    Variable,
    Function,
    Class,
    Array,
    Parameter
}

#[derive(Debug, Clone, PartialEq)]
pub enum BlockType {
    None,
    Class(String),
    Function(String),
    Constructor(String),
    For,
    While,
    DoWhile,
    If,
    Switch,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Symbol {
    pub name: String,
    pub symbol_type: SymbolType,
    pub category: SymbolCategory,
    pub scope_level: usize,
    pub line_declared: usize,
    pub memory_address: usize,
    pub size_in_bytes: usize,
    pub dimension: usize,
    pub parameter_types: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Scope {
    pub symbols: HashMap<String, Symbol>,
    pub parent_idx: Option<usize>,
    pub scope_level: usize,
    pub next_memory_offset: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SemanticAnalyzer {
    pub scopes: Vec<Scope>,
    pub current_scope_idx: usize,
    pub pending_parameters: Vec<Symbol>,
    pub block_type: BlockType,
    pub current_function_return_type: Option<String>,
    pub for_init_mode: bool,
}