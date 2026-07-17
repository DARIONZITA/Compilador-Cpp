use std::fmt;
use crate::token::Token;

#[derive(Debug, Clone, PartialEq)]
pub enum AstKind {
    Program,
    Include,
    ClassDecl,
    Inherit,
    AccessSection,
    ConstructorDecl,
    FunctionDecl,
    VarDecl,
    ArrayDecl,
    ParamList,
    Param,
    Block,
    If,
    Switch,
    Case,
    Default,
    While,
    DoWhile,
    For,
    Return,
    Break,
    Continue,
    IoIn,
    IoOut,
    Assign,
    BinaryOp,
    UnaryOp,
    Call,
    ArrayDimension,
    Index,
    MemberAccess,
    PtrAccess,
    New,
    This,
    Type,
    Modifier,
    Identifier,
    Literal,
    Error,
}

#[derive(Debug, Clone)]
pub struct AstNode {
    pub kind: AstKind,
    pub children: Vec<AstNode>,
    pub token: Option<String>,
    pub original_token: Option<Token>,
    pub inferred_type: String,
    pub line: usize,
}

impl AstNode {
    pub fn new(kind: AstKind, line: usize) -> Self {
        AstNode { kind, children: vec![], token: None, original_token: None, inferred_type: String::new(), line }
    }

    pub fn leaf(kind: AstKind, token: &str, line: usize) -> Self {
        AstNode { kind, children: vec![], token: Some(token.to_string()), original_token: None, inferred_type: String::new(), line }
    }

    pub fn with_children(kind: AstKind, children: Vec<AstNode>, line: usize) -> Self {
        AstNode { kind, children, token: None, original_token: None, inferred_type: String::new(), line }
    }

    pub fn with_token(kind: AstKind, children: Vec<AstNode>, token: &str, line: usize) -> Self {
        AstNode { kind, children, token: Some(token.to_string()), original_token: None, inferred_type: String::new(), line }
    }

    pub fn with_original_token(kind: AstKind, token: &str, original_token: Token, line: usize) -> Self {
        AstNode { kind, children: vec![], token: Some(token.to_string()), original_token: Some(original_token), inferred_type: String::new(), line }
    }
}

fn print_ast_node(node: &AstNode, indent: usize, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let kind_str = format!("{:?}", node.kind);
    write!(f, "{:indent$}{}", "", kind_str, indent = indent)?;
    if let Some(ref tok) = node.token {
        write!(f, " ({})", tok)?;
    }
    writeln!(f)?;
    for child in &node.children {
        print_ast_node(child, indent + 2, f)?;
    }
    Ok(())
}

pub fn format_ast(node: &AstNode) -> String {
    struct AstDisplay<'a>(&'a AstNode);
    impl<'a> fmt::Display for AstDisplay<'a> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            print_ast_node(self.0, 0, f)
        }
    }
    format!("{}", AstDisplay(node))
}
