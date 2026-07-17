use crate::ast::{AstKind, AstNode};
use crate::token::{Token, TokenInfo};
use std::sync::atomic::{AtomicUsize, Ordering};

static ERROR_COUNT: AtomicUsize = AtomicUsize::new(0);

fn incrementar_erro() {
    ERROR_COUNT.fetch_add(1, Ordering::SeqCst);
}

fn match_token(tokens: &[TokenInfo], pos: &mut usize, expected_kind: Token) -> bool {
    if *pos >= tokens.len() {
        return false;
    }
    if std::mem::discriminant(&tokens[*pos].kind) == std::mem::discriminant(&expected_kind) {
        *pos += 1;
        true
    } else {
        false
    }
}

fn expect(tokens: &[TokenInfo], pos: &mut usize, expected_kind: Token, expect_name: &str, follow: &[Token]) -> bool {
    if match_token(tokens, pos, expected_kind) {
        return true;
    }

    let current = if *pos < tokens.len() {
        format!("{:?} (`{}`)", tokens[*pos].kind, tokens[*pos].lexema)
    } else {
        "EOF".to_string()
    };
    eprintln!(
        "Erro [linha {}]: Esperado {}, mas encontrado {}",
        if *pos < tokens.len() { tokens[*pos].linha } else { 0 },
        expect_name,
        current
    );
    incrementar_erro();

    while *pos < tokens.len() && !follow.contains(&tokens[*pos].kind) {
        *pos += 1;
    }

    false
}

fn panic_mode_recovery(tokens: &[TokenInfo], pos: &mut usize, what_expected: &str, follow: &[Token]) {
    let current = if *pos < tokens.len() {
        format!("{:?} (`{}`)", tokens[*pos].kind, tokens[*pos].lexema)
    } else {
        "EOF".to_string()
    };
    eprintln!(
        "Erro [linha {}]: Esperado {}, mas encontrado {}",
        if *pos < tokens.len() { tokens[*pos].linha } else { 0 },
        what_expected,
        current
    );
    incrementar_erro();

    while *pos < tokens.len() && !follow.contains(&tokens[*pos].kind) {
        *pos += 1;
    }
}

fn lexema_at(tokens: &[TokenInfo], pos: usize) -> String {
    if pos < tokens.len() {
        tokens[pos].lexema.clone()
    } else {
        String::new()
    }
}

fn linha_at(tokens: &[TokenInfo], pos: usize) -> usize {
    if pos < tokens.len() { tokens[pos].linha } else { 0 }
}

fn pode_iniciar_declaracao(tokens: &[TokenInfo], pos: usize) -> bool {
    if pos >= tokens.len() {
        return false;
    }
    matches!(
        tokens[pos].kind,
        Token::Class | Token::Struct | Token::Int | Token::FloatType
            | Token::CharType | Token::Double | Token::Bool | Token::Void
            | Token::StringType
            | Token::Public | Token::Private | Token::Protected
            | Token::Static | Token::Const | Token::Identificador
    )
}

fn pode_ser_secao_acesso(tokens: &[TokenInfo], pos: usize) -> bool {
    matches!(tokens[pos].kind, Token::Public | Token::Private | Token::Protected)
}

fn pode_iniciar_membro(tokens: &[TokenInfo], pos: usize) -> bool {
    if pos >= tokens.len() {
        return false;
    }
    !matches!(tokens[pos].kind, Token::FechaChave | Token::Public | Token::Private | Token::Protected)
}

fn pode_iniciar_parametro(tokens: &[TokenInfo], pos: usize) -> bool {
    matches!(
        tokens[pos].kind,
        Token::Int | Token::FloatType | Token::Double | Token::CharType | Token::Bool
            | Token::Void | Token::StringType | Token::Identificador | Token::Const
    )
}

fn pode_iniciar_tipo(tokens: &[TokenInfo], pos: usize) -> bool {
    matches!(
        tokens[pos].kind,
        Token::Int | Token::FloatType | Token::Double | Token::CharType | Token::Bool
            | Token::Void | Token::String | Token::StringType | Token::Identificador
    )
}

fn pode_iniciar_expressao(tokens: &[TokenInfo], pos: usize) -> bool {
    if pos >= tokens.len() {
        return false;
    }
    matches!(
        tokens[pos].kind,
        Token::Identificador | Token::Inteiro | Token::Float | Token::Char | Token::String
            | Token::AbreParen | Token::Not | Token::Subtracao | Token::Incremento | Token::Decremento
            | Token::BitAnd | Token::Multiplicacao | Token::New | Token::This | Token::TrueLiteral
            | Token::FalseLiteral | Token::Adicao | Token::BitNot
    )
}

fn eh_op_atrib(kind: Token) -> bool {
    matches!(
        kind,
        Token::Atribuicao | Token::MaisIgual | Token::MenosIgual | Token::VezesIgual
            | Token::DivIgual | Token::ModIgual
    )
}

fn eh_op_relacional(kind: Token) -> bool {
    matches!(
        kind,
        Token::Igual | Token::Diferente | Token::Menor | Token::Maior | Token::MenorIgual
            | Token::MaiorIgual
    )
}

fn pode_iniciar_comando(tokens: &[TokenInfo], pos: usize) -> bool {
    if pos >= tokens.len() {
        return false;
    }
    !matches!(tokens[pos].kind, Token::FechaChave)
}

// ═══════════════════════════════════════════════════════════════════
// Programa
// ═══════════════════════════════════════════════════════════════════

// <programa> ::= <include-seq> <declaracao-seq>
fn st_programa(tokens: &[TokenInfo], pos: &mut usize) -> AstNode {
    let mut children = vec![];
    children.extend(st_include_seq(tokens, pos));
    children.extend(st_declaracao_seq(tokens, pos));
    AstNode::with_children(AstKind::Program, children, linha_at(tokens, *pos))
}

// <include-seq> ::= <include> <include-seq> | ε
fn st_include_seq(tokens: &[TokenInfo], pos: &mut usize) -> Vec<AstNode> {
    let follow = &[Token::Class, Token::Struct, Token::Int, Token::FloatType,
                   Token::CharType, Token::Double, Token::Bool, Token::Void, Token::Public,
                   Token::Private, Token::Protected, Token::Static, Token::Const, Token::Identificador];

    if *pos < tokens.len() && matches!(tokens[*pos].kind, Token::Include) {
        let mut nodes = vec![st_include(tokens, pos)];
        nodes.extend(st_include_seq(tokens, pos));
        nodes
    } else if *pos >= tokens.len() || follow.contains(&tokens[*pos].kind) {
        vec![]
    } else {
        panic_mode_recovery(tokens, pos, "#include ou declaração", follow);
        vec![]
    }
}

// <include> ::= "#include" <include-alvo>
fn st_include(tokens: &[TokenInfo], pos: &mut usize) -> AstNode {
    let follow = &[
        Token::Include, Token::Class, Token::Struct, Token::Int, Token::FloatType,
        Token::CharType, Token::Double, Token::Bool, Token::Void, Token::Public,
        Token::Private, Token::Protected, Token::Static, Token::Const, Token::Identificador,
    ];

    if !expect(tokens, pos, Token::Include, "#include", follow) {
        return AstNode::leaf(AstKind::Include, "", linha_at(tokens, *pos));
    }
    let literal = st_include_alvo(tokens, pos);
    AstNode::with_children(AstKind::Include, vec![literal], linha_at(tokens, *pos))
}

// <include-alvo> ::= "<" <Cadeia> ">" | '"' <Cadeia> '"'
fn st_include_alvo(tokens: &[TokenInfo], pos: &mut usize) -> AstNode {
    let follow = &[
        Token::Include, Token::Class, Token::Struct, Token::Int, Token::FloatType,
        Token::CharType, Token::Double, Token::Bool, Token::Void,
    ];

    if *pos < tokens.len() && tokens[*pos].kind == Token::Menor {
        *pos += 1;
        let lex = lexema_at(tokens, *pos);
        expect(tokens, pos, Token::String, "nome da biblioteca", follow);
        expect(tokens, pos, Token::Maior, ">", follow);
        AstNode::leaf(AstKind::Literal, &lex, linha_at(tokens, *pos))
    } else if *pos < tokens.len() && tokens[*pos].kind == Token::String {
        let lex = lexema_at(tokens, *pos);
        *pos += 1;
        AstNode::leaf(AstKind::Literal, &lex, linha_at(tokens, *pos))
    } else {
        eprintln!("Erro: Include mal formatado");
        incrementar_erro();
        AstNode::leaf(AstKind::Literal, "", linha_at(tokens, *pos))
    }
}

// <declaracao-seq> ::= <declaracao> <declaracao-seq> | ε
fn st_declaracao_seq(tokens: &[TokenInfo], pos: &mut usize) -> Vec<AstNode> {
    let follow = &[];
    if *pos < tokens.len() && pode_iniciar_declaracao(tokens, *pos) {
        let mut nodes = st_declaracao(tokens, pos);
        nodes.extend(st_declaracao_seq(tokens, pos));
        nodes
    } else if *pos >= tokens.len() {
        vec![]
    } else {
        panic_mode_recovery(tokens, pos, "declaração ou EOF", follow);
        vec![]
    }
}

// <declaracao> ::= <declaracao-classe> | <modificador> <tipo> <identificador> <sufixo-decl>
fn st_declaracao(tokens: &[TokenInfo], pos: &mut usize) -> Vec<AstNode> {
    let follow = &[Token::Class, Token::Struct, Token::Int, Token::FloatType, Token::CharType,
                   Token::Double, Token::Bool, Token::Void, Token::Public, Token::Private,
                   Token::Protected, Token::Static, Token::Const, Token::Identificador];

    if *pos < tokens.len() && matches!(tokens[*pos].kind, Token::Class | Token::Struct) {
        vec![st_declaracao_classe(tokens, pos)]
    } else {
        let mod_nodes = st_modificador(tokens, pos);
        let type_node = st_tipo(tokens, pos);
        let id_lex = lexema_at(tokens, *pos);
        if !expect(tokens, pos, Token::Identificador, "identificador", follow) {
            return vec![AstNode::new(AstKind::Error, 0)];
        }
        st_sufixo_decl(tokens, pos, mod_nodes, type_node, &id_lex)
    }
}

// ═══════════════════════════════════════════════════════════════════
// Modificadores e Tipos
// ═══════════════════════════════════════════════════════════════════

// <modificador> ::= "public" | "private" | "protected" | "static" | "const" | ε
fn st_modificador(tokens: &[TokenInfo], pos: &mut usize) -> Vec<AstNode> {
    let mut mods = vec![];
    while *pos < tokens.len() {
        match tokens[*pos].kind {
            Token::Public | Token::Private | Token::Protected | Token::Static | Token::Const
            | Token::Virtual | Token::Override | Token::Signed | Token::Unsigned
            | Token::Inline | Token::Explicit => {
                let lex = lexema_at(tokens, *pos);
                *pos += 1;
                mods.push(AstNode::leaf(AstKind::Modifier, &lex, linha_at(tokens, *pos)));
            }
            _ => break,
        }
    }
    mods
}

// <tipo> ::= "int" | "float" | "double" | "char" | "bool" | "void" | "string" | <identificador>
fn st_tipo(tokens: &[TokenInfo], pos: &mut usize) -> AstNode {
    if *pos >= tokens.len() {
        eprintln!("Erro: Tipo esperado mas encontrado EOF");
        incrementar_erro();
        return AstNode::leaf(AstKind::Type, "", linha_at(tokens, *pos));
    }
    match tokens[*pos].kind {
        Token::Int | Token::FloatType | Token::Double | Token::CharType | Token::Bool
        | Token::Void | Token::String | Token::StringType => {
            let lex = lexema_at(tokens, *pos);
            *pos += 1;
            AstNode::leaf(AstKind::Type, &lex, linha_at(tokens, *pos))
        }
        Token::Identificador => {
            let lex = lexema_at(tokens, *pos);
            *pos += 1;
            AstNode::leaf(AstKind::Type, &lex, linha_at(tokens, *pos))
        }
        _ => {
            eprintln!("Erro [linha {}]: Tipo esperado", tokens[*pos].linha);
            incrementar_erro();
            AstNode::leaf(AstKind::Type, "", linha_at(tokens, *pos))
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
// Declaração de Classe
// ═══════════════════════════════════════════════════════════════════

// <declaracao-classe> ::= "class" <identificador> <heranca> "{" <membros-classe> "}" ";"
fn st_declaracao_classe(tokens: &[TokenInfo], pos: &mut usize) -> AstNode {
    let follow = &[Token::Class, Token::Struct, Token::Int, Token::FloatType];

    if !match_token(tokens, pos, Token::Class) {
        match_token(tokens, pos, Token::Struct);
    }

    let id_lex = lexema_at(tokens, *pos);
    if !expect(tokens, pos, Token::Identificador, "nome da classe", follow) {
        return AstNode::new(AstKind::Error, 0);
    }

    let inherit = st_heranca(tokens, pos);
    if !expect(tokens, pos, Token::AbreChave, "{", follow) {
        return AstNode::new(AstKind::Error, 0);
    }
    let members = st_membros_classe(tokens, pos);
    if !expect(tokens, pos, Token::FechaChave, "}", follow) {
        return AstNode::new(AstKind::Error, 0);
    }
    if !expect(tokens, pos, Token::PtVirgula, ";", follow) {
        return AstNode::new(AstKind::Error, 0);
    }

    let mut children = vec![AstNode::leaf(AstKind::Identifier, &id_lex, linha_at(tokens, *pos))];
    if let Some(i) = inherit {
        children.push(i);
    }
    children.extend(members);
    AstNode::with_children(AstKind::ClassDecl, children, linha_at(tokens, *pos))
}

// <heranca> ::= ":" <acesso> <identificador> | ε
fn st_heranca(tokens: &[TokenInfo], pos: &mut usize) -> Option<AstNode> {
    let follow = &[Token::AbreChave, Token::Public, Token::Private, Token::Protected];

    if *pos < tokens.len() && tokens[*pos].kind == Token::DoisPontos {
        *pos += 1;
        let access = st_acesso(tokens, pos);
        let id_lex = lexema_at(tokens, *pos);
        if !expect(tokens, pos, Token::Identificador, "nome da classe base", follow) {
            return None;
        }

        let mut children = vec![];
        if let Some(a) = access {
            children.push(a);
        }
        children.push(AstNode::leaf(AstKind::Identifier, &id_lex, linha_at(tokens, *pos)));
        Some(AstNode::with_children(AstKind::Inherit, children, linha_at(tokens, *pos)))
    } else {
        None
    }
}

// <acesso> ::= "public" | "private" | "protected"
fn st_acesso(tokens: &[TokenInfo], pos: &mut usize) -> Option<AstNode> {
    if *pos < tokens.len() {
        match tokens[*pos].kind {
            Token::Public | Token::Private | Token::Protected => {
                let lex = lexema_at(tokens, *pos);
                *pos += 1;
                Some(AstNode::leaf(AstKind::Modifier, &lex, linha_at(tokens, *pos)))
            }
            _ => None,
        }
    } else {
        None
    }
}

// <membros-classe> ::= <seccao-acesso> <membros-classe> | ε
fn st_membros_classe(tokens: &[TokenInfo], pos: &mut usize) -> Vec<AstNode> {
    let follow = &[Token::FechaChave];

    if *pos < tokens.len() && pode_ser_secao_acesso(tokens, *pos) {
        let mut nodes = vec![st_seccao_acesso(tokens, pos)];
        nodes.extend(st_membros_classe(tokens, pos));
        nodes
    } else if *pos >= tokens.len() || follow.contains(&tokens[*pos].kind) {
        vec![]
    } else {
        panic_mode_recovery(tokens, pos, "public/private/protected ou '}'", follow);
        vec![]
    }
}

// <seccao-acesso> ::= <acesso> ":" <lista-membros>
fn st_seccao_acesso(tokens: &[TokenInfo], pos: &mut usize) -> AstNode {
    let follow = &[Token::Public, Token::Private, Token::Protected, Token::FechaChave];

    let access = st_acesso(tokens, pos);
    if !expect(tokens, pos, Token::DoisPontos, ":", follow) {
        return AstNode::new(AstKind::Error, 0);
    }
    let members = st_lista_membros(tokens, pos);

    let mut children = vec![];
    if let Some(a) = access {
        children.push(a);
    }
    children.extend(members);
    AstNode::with_children(AstKind::AccessSection, children, linha_at(tokens, *pos))
}

// <lista-membros> ::= <membro> <lista-membros> | ε
fn st_lista_membros(tokens: &[TokenInfo], pos: &mut usize) -> Vec<AstNode> {
    let follow = &[Token::FechaChave, Token::Public, Token::Private, Token::Protected];

    if *pos < tokens.len() && pode_iniciar_membro(tokens, *pos) {
        let mut nodes = st_membro(tokens, pos);
        nodes.extend(st_lista_membros(tokens, pos));
        nodes
    } else if *pos >= tokens.len() || follow.contains(&tokens[*pos].kind) {
        vec![]
    } else {
        panic_mode_recovery(tokens, pos, "membro de classe", follow);
        vec![]
    }
}

// <membro> ::= <declaracao-construtor> | <modificador> <tipo> <identificador> <sufixo-decl>
fn st_membro(tokens: &[TokenInfo], pos: &mut usize) -> Vec<AstNode> {
    let follow = &[Token::Public, Token::Private, Token::Protected, Token::FechaChave];

    let checkpoint = *pos;

    if *pos < tokens.len() && tokens[*pos].kind == Token::Identificador {
        let next_pos = *pos + 1;
        if next_pos < tokens.len() && tokens[next_pos].kind == Token::AbreParen {
            return vec![st_declaracao_construtor(tokens, pos)];
        }
    }

    *pos = checkpoint;
    let mod_nodes = st_modificador(tokens, pos);
    let type_node = st_tipo(tokens, pos);
    let id_lex = lexema_at(tokens, *pos);
    if !expect(tokens, pos, Token::Identificador, "identificador", follow) {
        return vec![AstNode::new(AstKind::Error, 0)];
    }
    st_sufixo_decl(tokens, pos, mod_nodes, type_node, &id_lex)
}

// <declaracao-construtor> ::= <identificador> "(" <parametros> ")" <bloco>
fn st_declaracao_construtor(tokens: &[TokenInfo], pos: &mut usize) -> AstNode {
    let follow = &[Token::Public, Token::Private, Token::Protected, Token::FechaChave];

    let id_lex = lexema_at(tokens, *pos);
    if !expect(tokens, pos, Token::Identificador, "nome do construtor", follow) {
        return AstNode::new(AstKind::Error, 0);
    }
    if !expect(tokens, pos, Token::AbreParen, "(", follow) {
        return AstNode::new(AstKind::Error, 0);
    }
    let params = st_parametros(tokens, pos);
    if !expect(tokens, pos, Token::FechaParen, ")", follow) {
        return AstNode::new(AstKind::Error, 0);
    }
    let body = st_bloco(tokens, pos);

    AstNode::with_children(AstKind::ConstructorDecl, vec![
        AstNode::leaf(AstKind::Identifier, &id_lex, linha_at(tokens, *pos)),
        params,
        body,
    ], linha_at(tokens, *pos))
}

// ═══════════════════════════════════════════════════════════════════
// Sufixo de Declaração (função vs variável/array)
// ═══════════════════════════════════════════════════════════════════

// <sufixo-decl> ::= "(" <parametros> ")" <corpo-funcao> | <var-resto> ";"
fn st_sufixo_decl(tokens: &[TokenInfo], pos: &mut usize, mod_nodes: Vec<AstNode>, type_node: AstNode, id_lex: &str) -> Vec<AstNode> {
    let follow = &[Token::Public, Token::Private, Token::Protected, Token::FechaChave, Token::PtVirgula];

    if *pos < tokens.len() && tokens[*pos].kind == Token::AbreParen {
        *pos += 1;
        let params = st_parametros(tokens, pos);
        if !expect(tokens, pos, Token::FechaParen, ")", follow) {
            return vec![AstNode::new(AstKind::Error, 0)];
        }
        let body = st_corpo_funcao(tokens, pos);

        let mut children = vec![];
        children.extend(mod_nodes);
        children.push(type_node);
        children.push(AstNode::leaf(AstKind::Identifier, id_lex, linha_at(tokens, *pos)));
        children.push(params);
        if let Some(b) = body {
            children.push(b);
        }
        vec![AstNode::with_children(AstKind::FunctionDecl, children, linha_at(tokens, *pos))]
    } else {
        let decls = st_var_resto(tokens, pos, mod_nodes, type_node, id_lex);
        if !expect(tokens, pos, Token::PtVirgula, ";", follow) {
            return decls;
        }
        decls
    }
}

// <corpo-funcao> ::= <bloco> | ";"
fn st_corpo_funcao(tokens: &[TokenInfo], pos: &mut usize) -> Option<AstNode> {
    if *pos < tokens.len() && tokens[*pos].kind == Token::AbreChave {
        Some(st_bloco(tokens, pos))
    } else {
        match_token(tokens, pos, Token::PtVirgula);
        None
    }
}

// ═══════════════════════════════════════════════════════════════════
// Parâmetros
// ═══════════════════════════════════════════════════════════════════

// <parametros> ::= <lista-parametros> | ε
fn st_parametros(tokens: &[TokenInfo], pos: &mut usize) -> AstNode {
    let follow = &[Token::FechaParen];

    if *pos < tokens.len() && pode_iniciar_parametro(tokens, *pos) {
        let params = st_lista_parametros(tokens, pos);
        AstNode::with_children(AstKind::ParamList, params, linha_at(tokens, *pos))
    } else if *pos >= tokens.len() || follow.contains(&tokens[*pos].kind) {
        AstNode::with_children(AstKind::ParamList, vec![], linha_at(tokens, *pos))
    } else {
        panic_mode_recovery(tokens, pos, "tipo de parâmetro ou ')'", follow);
        AstNode::with_children(AstKind::ParamList, vec![], linha_at(tokens, *pos))
    }
}

// <lista-parametros> ::= <parametro> <lista-parametros'>
fn st_lista_parametros(tokens: &[TokenInfo], pos: &mut usize) -> Vec<AstNode> {
    let mut nodes = vec![st_parametro(tokens, pos)];
    nodes.extend(st_lista_parametros_resto(tokens, pos));
    nodes
}

// <lista-parametros'> ::= "," <parametro> <lista-parametros'> | ε
fn st_lista_parametros_resto(tokens: &[TokenInfo], pos: &mut usize) -> Vec<AstNode> {
    let follow = &[Token::FechaParen];

    if *pos < tokens.len() && tokens[*pos].kind == Token::Virgula {
        *pos += 1;
        let mut nodes = vec![st_parametro(tokens, pos)];
        nodes.extend(st_lista_parametros_resto(tokens, pos));
        nodes
    } else if *pos >= tokens.len() || follow.contains(&tokens[*pos].kind) {
        vec![]
    } else {
        panic_mode_recovery(tokens, pos, "',' ou ')'", follow);
        vec![]
    }
}

// <parametro> ::= <tipo> <identificador> <param-sufixo>
fn st_parametro(tokens: &[TokenInfo], pos: &mut usize) -> AstNode {
    let follow = &[Token::Virgula, Token::FechaParen];

    let type_node = st_tipo(tokens, pos);
    let id_lex = lexema_at(tokens, *pos);
    if !expect(tokens, pos, Token::Identificador, "identificador", follow) {
        return AstNode::with_children(AstKind::Error, vec![type_node], 0);
    }
    let is_array = st_param_sufixo(tokens, pos);

    let id_node = if is_array {
        AstNode::leaf(AstKind::Identifier, &format!("{}[]", id_lex), linha_at(tokens, *pos))
    } else {
        AstNode::leaf(AstKind::Identifier, &id_lex, linha_at(tokens, *pos))
    };

    AstNode::with_children(AstKind::Param, vec![type_node, id_node], linha_at(tokens, *pos))
}

// <param-sufixo> ::= "[" "]" | ε
fn st_param_sufixo(tokens: &[TokenInfo], pos: &mut usize) -> bool {
    if *pos < tokens.len() && tokens[*pos].kind == Token::AbreColch {
        *pos += 1;
        expect(tokens, pos, Token::FechaColch, "]", &[Token::Virgula, Token::FechaParen, Token::AbreParen]);
        true
    } else {
        false
    }
}

// ═══════════════════════════════════════════════════════════════════
// Variável / Array — resto da declaração
// ═══════════════════════════════════════════════════════════════════

// <var-resto> ::= "=" <expressao> <lista-variaveis'>
//               | "[" <dim-conteudo> "]" <mais-dims> <init-array> <lista-variaveis'>
//               | <lista-variaveis'>
fn st_var_resto(tokens: &[TokenInfo], pos: &mut usize, mod_nodes: Vec<AstNode>, type_node: AstNode, id_lex: &str) -> Vec<AstNode> {
    let follow = &[Token::Virgula, Token::PtVirgula, Token::AbreParen];

    if *pos < tokens.len() && tokens[*pos].kind == Token::Atribuicao {
        *pos += 1;
        let init_expr = st_expressao(tokens, pos);
        let extra_decls = st_lista_variaveis_resto(tokens, pos, mod_nodes.clone(), type_node.clone());

        let mut children = vec![];
        children.extend(mod_nodes);
        children.push(type_node);
        children.push(AstNode::leaf(AstKind::Identifier, id_lex, linha_at(tokens, *pos)));
        children.push(init_expr);

        let mut result = vec![AstNode::with_children(AstKind::VarDecl, children, linha_at(tokens, *pos))];
        result.extend(extra_decls);
        result

    } else if *pos < tokens.len() && tokens[*pos].kind == Token::AbreColch {
        *pos += 1;
        let dim = st_dim_conteudo(tokens, pos);
        if !expect(tokens, pos, Token::FechaColch, "]", follow) {
            return vec![];
        }
        let mais_dims = st_mais_dims(tokens, pos);
        let inits = st_init_array(tokens, pos);
        let extra_decls = st_lista_variaveis_resto(tokens, pos, mod_nodes.clone(), type_node.clone());

        let mut children = vec![];
        children.extend(mod_nodes);
        children.push(type_node);
        children.push(AstNode::leaf(AstKind::Identifier, id_lex, linha_at(tokens, *pos)));
        if let Some(d) = dim {
            children.push(d);
        }
        children.extend(mais_dims);
        children.extend(inits);

        let mut result = vec![AstNode::with_children(AstKind::ArrayDecl, children, linha_at(tokens, *pos))];
        result.extend(extra_decls);
        result

    } else {
        let extra_decls = st_lista_variaveis_resto(tokens, pos, mod_nodes.clone(), type_node.clone());

        let mut children = vec![];
        children.extend(mod_nodes);
        children.push(type_node);
        children.push(AstNode::leaf(AstKind::Identifier, id_lex, linha_at(tokens, *pos)));

        let mut result = vec![AstNode::with_children(AstKind::VarDecl, children, linha_at(tokens, *pos))];
        result.extend(extra_decls);
        result
    }
}

// <lista-variaveis'> ::= "," <identificador> <var-resto> | ε
fn st_lista_variaveis_resto(tokens: &[TokenInfo], pos: &mut usize, mod_nodes: Vec<AstNode>, type_node: AstNode) -> Vec<AstNode> {
    let follow = &[Token::PtVirgula, Token::AbreParen, Token::Virgula];

    if *pos < tokens.len() && tokens[*pos].kind == Token::Virgula {
        *pos += 1;
        let id_lex = lexema_at(tokens, *pos);
        if !expect(tokens, pos, Token::Identificador, "identificador", follow) {
            return vec![];
        }
        let more = st_var_resto(tokens, pos, mod_nodes, type_node, &id_lex);
        more
    } else if *pos >= tokens.len() || follow.contains(&tokens[*pos].kind) {
        vec![]
    } else {
        panic_mode_recovery(tokens, pos, "',' ou identificador", follow);
        vec![]
    }
}

// <dim-conteudo> ::= <expressao> | ε
fn st_dim_conteudo(tokens: &[TokenInfo], pos: &mut usize) -> Option<AstNode> {
    let follow = &[Token::FechaColch];

    if *pos < tokens.len() && pode_iniciar_expressao(tokens, *pos) {
        let expr = st_expressao(tokens, pos);
        Some(AstNode::with_children(AstKind::ArrayDimension, vec![expr], linha_at(tokens, *pos)))
    } else if *pos >= tokens.len() || follow.contains(&tokens[*pos].kind) {
        None
    } else {
        panic_mode_recovery(tokens, pos, "expressão", follow);
        None
    }
}

// <mais-dims> ::= "[" <dim-conteudo> "]" <mais-dims> | ε
fn st_mais_dims(tokens: &[TokenInfo], pos: &mut usize) -> Vec<AstNode> {
    let follow = &[Token::Atribuicao, Token::Virgula, Token::PtVirgula];

    if *pos < tokens.len() && tokens[*pos].kind == Token::AbreColch {
        *pos += 1;
        let dim = st_dim_conteudo(tokens, pos);
        if !expect(tokens, pos, Token::FechaColch, "]", follow) {
            return vec![];
        }
        let mut dims = st_mais_dims(tokens, pos);
        if let Some(d) = dim {
            dims.insert(0, d);
        }
        dims
    } else if *pos >= tokens.len() || follow.contains(&tokens[*pos].kind) {
        vec![]
    } else {
        panic_mode_recovery(tokens, pos, "'['", follow);
        vec![]
    }
}

// <init-array> ::= "=" "{" <lista-init> "}" | ε
fn st_init_array(tokens: &[TokenInfo], pos: &mut usize) -> Vec<AstNode> {
    let follow = &[Token::Virgula, Token::PtVirgula];

    if *pos < tokens.len() && tokens[*pos].kind == Token::Atribuicao {
        *pos += 1;
        if !expect(tokens, pos, Token::AbreChave, "{", follow) {
            return vec![];
        }
        let inits = st_lista_init(tokens, pos);
        if !expect(tokens, pos, Token::FechaChave, "}", follow) {
            return vec![];
        }
        inits
    } else {
        vec![]
    }
}

// <lista-init> ::= <expressao> <lista-init'> | ε
fn st_lista_init(tokens: &[TokenInfo], pos: &mut usize) -> Vec<AstNode> {
    let follow = &[Token::FechaChave];

    if *pos < tokens.len() && pode_iniciar_expressao(tokens, *pos) {
        let mut nodes = vec![st_expressao(tokens, pos)];
        nodes.extend(st_lista_init_resto(tokens, pos));
        nodes
    } else if *pos >= tokens.len() || follow.contains(&tokens[*pos].kind) {
        vec![]
    } else {
        panic_mode_recovery(tokens, pos, "expressão", follow);
        vec![]
    }
}

// <lista-init'> ::= "," <expressao> <lista-init'> | ε
fn st_lista_init_resto(tokens: &[TokenInfo], pos: &mut usize) -> Vec<AstNode> {
    let follow = &[Token::FechaChave];

    if *pos < tokens.len() && tokens[*pos].kind == Token::Virgula {
        *pos += 1;
        let mut nodes = vec![];
        if pode_iniciar_expressao(tokens, *pos) {
            nodes.push(st_expressao(tokens, pos));
        }
        nodes.extend(st_lista_init_resto(tokens, pos));
        nodes
    } else if *pos >= tokens.len() || follow.contains(&tokens[*pos].kind) {
        vec![]
    } else {
        panic_mode_recovery(tokens, pos, "',' ou '}'", follow);
        vec![]
    }
}

// ═══════════════════════════════════════════════════════════════════
// Bloco e Comandos
// ═══════════════════════════════════════════════════════════════════

// <bloco> ::= "{" <comando-seq> "}"
fn st_bloco(tokens: &[TokenInfo], pos: &mut usize) -> AstNode {
    let follow = &[Token::Public, Token::Private, Token::Protected, Token::FechaChave, Token::Else, Token::PtVirgula];

    if !expect(tokens, pos, Token::AbreChave, "{", follow) {
        return AstNode::new(AstKind::Error, 0);
    }
    let stmts = st_comando_seq(tokens, pos);
    if !expect(tokens, pos, Token::FechaChave, "}", follow) {
        return AstNode::with_children(AstKind::Block, stmts, linha_at(tokens, *pos));
    }
    AstNode::with_children(AstKind::Block, stmts, linha_at(tokens, *pos))
}

// <comando-seq> ::= <comando> <comando-seq> | ε
fn st_comando_seq(tokens: &[TokenInfo], pos: &mut usize) -> Vec<AstNode> {
    let follow = &[Token::FechaChave];

    if *pos < tokens.len() && pode_iniciar_comando(tokens, *pos) {
        let mut nodes = st_comando(tokens, pos);
        nodes.extend(st_comando_seq(tokens, pos));
        nodes
    } else if *pos >= tokens.len() || follow.contains(&tokens[*pos].kind) {
        vec![]
    } else {
        panic_mode_recovery(tokens, pos, "comando ou '}'", follow);
        vec![]
    }
}

// <comando> ::= <modificador> <tipo> <identificador> <sufixo-decl>
//             | <comando-seleccao> | <comando-repeticao> | <comando-io>
//             | "return" <expressao-opt> ";" | "break" ";" | "continue" ";" | <bloco> | <expressao> ";"
fn st_comando(tokens: &[TokenInfo], pos: &mut usize) -> Vec<AstNode> {
    let follow = &[Token::FechaChave, Token::PtVirgula, Token::Else, Token::Case, Token::Default];

    if *pos >= tokens.len() {
        return vec![];
    }

    match tokens[*pos].kind {
        Token::If | Token::Switch => {
            vec![st_comando_seleccao(tokens, pos)]
        }
        Token::While | Token::Do | Token::For => {
            vec![st_comando_repeticao(tokens, pos)]
        }
        Token::Cin | Token::Cout => {
            vec![st_comando_io(tokens, pos)]
        }
        Token::Return => {
            *pos += 1;
            let expr = st_expressao_opt(tokens, pos);
            let mut node = AstNode::new(AstKind::Return, linha_at(tokens, *pos));
            if let Some(e) = expr {
                node.children.push(e);
            }
            if !expect(tokens, pos, Token::PtVirgula, ";", follow) {
                return vec![node];
            }
            vec![node]
        }
        Token::Break => {
            *pos += 1;
            if !expect(tokens, pos, Token::PtVirgula, ";", follow) {
                return vec![AstNode::new(AstKind::Break, linha_at(tokens, *pos))];
            }
            vec![AstNode::new(AstKind::Break, linha_at(tokens, *pos))]
        }
        Token::Continue => {
            *pos += 1;
            if !expect(tokens, pos, Token::PtVirgula, ";", follow) {
                return vec![AstNode::new(AstKind::Continue, linha_at(tokens, *pos))];
            }
            vec![AstNode::new(AstKind::Continue, linha_at(tokens, *pos))]
        }
        Token::AbreChave => {
            vec![st_bloco(tokens, pos)]
        }
        Token::Int | Token::FloatType | Token::Double | Token::CharType | Token::Bool | Token::Void => {
            let mods = st_modificador(tokens, pos);
            let type_node = st_tipo(tokens, pos);
            let id_lex = lexema_at(tokens, *pos);
            if !expect(tokens, pos, Token::Identificador, "identificador", follow) {
                return vec![AstNode::new(AstKind::Error, 0)];
            }
            st_sufixo_decl(tokens, pos, mods, type_node, &id_lex)
        }
        _ => {
            let expr = st_expressao(tokens, pos);
            if !expect(tokens, pos, Token::PtVirgula, ";", follow) {
                return vec![expr];
            }
            vec![expr]
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
// Comandos de Seleção
// ═══════════════════════════════════════════════════════════════════

// <comando-seleccao> ::= "if" "(" <expressao> ")" <bloco> <else-parte>
//                      | "switch" "(" <expressao> ")" "{" <casos> "}"
fn st_comando_seleccao(tokens: &[TokenInfo], pos: &mut usize) -> AstNode {
    let follow = &[Token::FechaChave, Token::Else, Token::PtVirgula];

    if match_token(tokens, pos, Token::If) {
        if !expect(tokens, pos, Token::AbreParen, "(", follow) { return AstNode::new(AstKind::Error, 0); }
        let cond = st_expressao(tokens, pos);
        if !expect(tokens, pos, Token::FechaParen, ")", follow) { return AstNode::new(AstKind::Error, 0); }
        let then_block = st_bloco(tokens, pos);
        let else_part = st_else_parte(tokens, pos);

        let mut children = vec![cond, then_block];
        if let Some(e) = else_part {
            children.push(e);
        }
        AstNode::with_children(AstKind::If, children, linha_at(tokens, *pos))

    } else if match_token(tokens, pos, Token::Switch) {
        if !expect(tokens, pos, Token::AbreParen, "(", follow) { return AstNode::new(AstKind::Error, 0); }
        let expr = st_expressao(tokens, pos);
        if !expect(tokens, pos, Token::FechaParen, ")", follow) { return AstNode::new(AstKind::Error, 0); }
        if !expect(tokens, pos, Token::AbreChave, "{", follow) { return AstNode::new(AstKind::Error, 0); }
        let cases = st_casos(tokens, pos);
        if !expect(tokens, pos, Token::FechaChave, "}", follow) { return AstNode::new(AstKind::Error, 0); }

        let mut children = vec![expr];
        children.extend(cases);
        AstNode::with_children(AstKind::Switch, children, linha_at(tokens, *pos))

    } else {
        AstNode::new(AstKind::Error, 0)
    }
}

// <else-parte> ::= "else" <else-corpo> | ε
fn st_else_parte(tokens: &[TokenInfo], pos: &mut usize) -> Option<AstNode> {
    let _follow = &[Token::FechaChave, Token::PtVirgula, Token::Else, Token::Case, Token::Default];

    if *pos < tokens.len() && tokens[*pos].kind == Token::Else {
        *pos += 1;
        Some(st_else_corpo(tokens, pos))
    } else {
        None
    }
}

// <else-corpo> ::= "if" "(" <expressao> ")" <bloco> <else-parte> | <bloco>
fn st_else_corpo(tokens: &[TokenInfo], pos: &mut usize) -> AstNode {
    let follow = &[Token::FechaChave, Token::Else, Token::PtVirgula];

    if *pos < tokens.len() && tokens[*pos].kind == Token::If {
        *pos += 1;
        if !expect(tokens, pos, Token::AbreParen, "(", follow) { return AstNode::new(AstKind::Error, 0); }
        let cond = st_expressao(tokens, pos);
        if !expect(tokens, pos, Token::FechaParen, ")", follow) { return AstNode::new(AstKind::Error, 0); }
        let then_block = st_bloco(tokens, pos);
        let else_part = st_else_parte(tokens, pos);

        let mut children = vec![cond, then_block];
        if let Some(e) = else_part {
            children.push(e);
        }
        AstNode::with_children(AstKind::If, children, linha_at(tokens, *pos))
    } else {
        st_bloco(tokens, pos)
    }
}

// <casos> ::= <caso> <casos> | ε
fn st_casos(tokens: &[TokenInfo], pos: &mut usize) -> Vec<AstNode> {
    let follow = &[Token::FechaChave, Token::Case, Token::Default];

    if *pos < tokens.len() && matches!(tokens[*pos].kind, Token::Case | Token::Default) {
        let mut nodes = vec![st_caso(tokens, pos)];
        nodes.extend(st_casos(tokens, pos));
        nodes
    } else if *pos >= tokens.len() || follow.contains(&tokens[*pos].kind) {
        vec![]
    } else {
        panic_mode_recovery(tokens, pos, "case ou default", follow);
        vec![]
    }
}

// <caso> ::= "case" <literal> ":" <comando-seq>
//           | "default" ":" <comando-seq>
fn st_caso(tokens: &[TokenInfo], pos: &mut usize) -> AstNode {
    let follow = &[Token::Case, Token::Default, Token::FechaChave];

    if match_token(tokens, pos, Token::Case) {
        let literal = st_literal(tokens, pos);
        if !expect(tokens, pos, Token::DoisPontos, ":", follow) { return AstNode::new(AstKind::Error, 0); }
        let stmts = st_comando_seq(tokens, pos);
        let block = AstNode::with_children(AstKind::Block, stmts, linha_at(tokens, *pos));
        AstNode::with_children(AstKind::Case, vec![literal, block], linha_at(tokens, *pos))

    } else if match_token(tokens, pos, Token::Default) {
        if !expect(tokens, pos, Token::DoisPontos, ":", follow) { return AstNode::new(AstKind::Error, 0); }
        let stmts = st_comando_seq(tokens, pos);
        let block = AstNode::with_children(AstKind::Block, stmts, linha_at(tokens, *pos));
        AstNode::with_children(AstKind::Default, vec![block], linha_at(tokens, *pos))

    } else {
        AstNode::new(AstKind::Error, 0)
    }
}

// ═══════════════════════════════════════════════════════════════════
// Comandos de Repetição
// ═══════════════════════════════════════════════════════════════════

// <comando-repeticao> ::= "while" "(" <expressao> ")" <bloco>
//                       | "do" <bloco> "while" "(" <expressao> ")" ";"
//                       | "for" "(" <for-init> <expressao-opt> ";" <expressao-opt> ")" <bloco>
fn st_comando_repeticao(tokens: &[TokenInfo], pos: &mut usize) -> AstNode {
    let follow = &[Token::FechaChave, Token::Else, Token::PtVirgula];

    if match_token(tokens, pos, Token::While) {
        if !expect(tokens, pos, Token::AbreParen, "(", follow) { return AstNode::new(AstKind::Error, 0); }
        let cond = st_expressao(tokens, pos);
        if !expect(tokens, pos, Token::FechaParen, ")", follow) { return AstNode::new(AstKind::Error, 0); }
        let body = st_bloco(tokens, pos);
        AstNode::with_children(AstKind::While, vec![cond, body], linha_at(tokens, *pos))

    } else if match_token(tokens, pos, Token::Do) {
        let body = st_bloco(tokens, pos);
        if !expect(tokens, pos, Token::While, "while", follow) { return AstNode::new(AstKind::Error, 0); }
        if !expect(tokens, pos, Token::AbreParen, "(", follow) { return AstNode::new(AstKind::Error, 0); }
        let cond = st_expressao(tokens, pos);
        if !expect(tokens, pos, Token::FechaParen, ")", follow) { return AstNode::new(AstKind::Error, 0); }
        if !expect(tokens, pos, Token::PtVirgula, ";", follow) { return AstNode::new(AstKind::Error, 0); }
        AstNode::with_children(AstKind::DoWhile, vec![body, cond], linha_at(tokens, *pos))

    } else if match_token(tokens, pos, Token::For) {
        if !expect(tokens, pos, Token::AbreParen, "(", follow) { return AstNode::new(AstKind::Error, 0); }
        let init = st_for_init(tokens, pos);
        let cond = st_expressao_opt(tokens, pos);
        if !expect(tokens, pos, Token::PtVirgula, ";", follow) { return AstNode::new(AstKind::Error, 0); }
        let incr = st_expressao_opt(tokens, pos);
        if !expect(tokens, pos, Token::FechaParen, ")", follow) { return AstNode::new(AstKind::Error, 0); }
        let body = st_bloco(tokens, pos);

        let mut children = vec![];
        if let Some(i) = init { children.push(i); }
        if let Some(c) = cond { children.push(c); }
        if let Some(i) = incr { children.push(i); }
        children.push(body);
        AstNode::with_children(AstKind::For, children, linha_at(tokens, *pos))

    } else {
        AstNode::new(AstKind::Error, 0)
    }
}

// <for-init> ::= <tipo> <identificador> <var-resto> ";"
//              | <expressao> ";"
//              | ";"
fn st_for_init(tokens: &[TokenInfo], pos: &mut usize) -> Option<AstNode> {
    let follow = &[Token::PtVirgula];
    let saved_pos = *pos;

    if *pos < tokens.len() && pode_iniciar_tipo(tokens, *pos) {
        let type_node = st_tipo(tokens, pos);
        if *pos < tokens.len() && tokens[*pos].kind == Token::Identificador {
            let id_lex = lexema_at(tokens, *pos);
            *pos += 1;
            let decls = st_var_resto(tokens, pos, vec![], type_node, &id_lex);
            expect(tokens, pos, Token::PtVirgula, ";", follow);
            return decls.into_iter().next();
        }
        *pos = saved_pos;
    }

    if *pos < tokens.len() && pode_iniciar_expressao(tokens, *pos) {
        let expr = st_expressao(tokens, pos);
        expect(tokens, pos, Token::PtVirgula, ";", follow);
        return Some(expr);
    }

    match_token(tokens, pos, Token::PtVirgula);
    None
}

// <expressao-opt> ::= <expressao> | ε
fn st_expressao_opt(tokens: &[TokenInfo], pos: &mut usize) -> Option<AstNode> {
    let follow = &[Token::PtVirgula, Token::FechaParen];

    if *pos < tokens.len() && pode_iniciar_expressao(tokens, *pos) {
        Some(st_expressao(tokens, pos))
    } else if *pos >= tokens.len() || follow.contains(&tokens[*pos].kind) {
        None
    } else {
        panic_mode_recovery(tokens, pos, "expressão", follow);
        None
    }
}

// ═══════════════════════════════════════════════════════════════════
// Comandos de I/O
// ═══════════════════════════════════════════════════════════════════

// <comando-io> ::= "cin" <cin-cadeia> ";"
//               | "cout" <cout-cadeia> ";"
fn st_comando_io(tokens: &[TokenInfo], pos: &mut usize) -> AstNode {
    let follow = &[Token::FechaChave, Token::Else, Token::PtVirgula, Token::Case, Token::Default];

    if match_token(tokens, pos, Token::Cin) {
        let targets = st_cin_cadeia(tokens, pos);
        if !expect(tokens, pos, Token::PtVirgula, ";", follow) { return AstNode::with_children(AstKind::Error, targets, 0); }
        AstNode::with_children(AstKind::IoIn, targets, linha_at(tokens, *pos))
    } else if match_token(tokens, pos, Token::Cout) {
        let items = st_cout_cadeia(tokens, pos);
        if !expect(tokens, pos, Token::PtVirgula, ";", follow) { return AstNode::with_children(AstKind::Error, items, 0); }
        AstNode::with_children(AstKind::IoOut, items, linha_at(tokens, *pos))
    } else {
        AstNode::new(AstKind::Error, 0)
    }
}

// <cin-cadeia> ::= ">>" <expressao> <cin-cadeia'>
fn st_cin_cadeia(tokens: &[TokenInfo], pos: &mut usize) -> Vec<AstNode> {
    match_token(tokens, pos, Token::ShiftDir);
    let mut nodes = vec![st_expressao(tokens, pos)];
    nodes.extend(st_cin_cadeia_resto(tokens, pos));
    nodes
}

// <cin-cadeia'> ::= ">>" <expressao> <cin-cadeia'> | ε
fn st_cin_cadeia_resto(tokens: &[TokenInfo], pos: &mut usize) -> Vec<AstNode> {
    if match_token(tokens, pos, Token::ShiftDir) {
        let mut nodes = vec![st_expressao(tokens, pos)];
        nodes.extend(st_cin_cadeia_resto(tokens, pos));
        nodes
    } else {
        vec![]
    }
}

// <cout-cadeia> ::= "<<" <cout-item> <cout-cadeia'>
fn st_cout_cadeia(tokens: &[TokenInfo], pos: &mut usize) -> Vec<AstNode> {
    match_token(tokens, pos, Token::ShiftEsq);
    let mut nodes = vec![st_cout_item(tokens, pos)];
    nodes.extend(st_cout_cadeia_resto(tokens, pos));
    nodes
}

// <cout-cadeia'> ::= "<<" <cout-item> <cout-cadeia'> | ε
fn st_cout_cadeia_resto(tokens: &[TokenInfo], pos: &mut usize) -> Vec<AstNode> {
    if match_token(tokens, pos, Token::ShiftEsq) {
        let mut nodes = vec![st_cout_item(tokens, pos)];
        nodes.extend(st_cout_cadeia_resto(tokens, pos));
        nodes
    } else {
        vec![]
    }
}

// <cout-item> ::= "endl" | <expressao>
fn st_cout_item(tokens: &[TokenInfo], pos: &mut usize) -> AstNode {
    if *pos < tokens.len() && tokens[*pos].kind == Token::Identificador && tokens[*pos].lexema == "endl" {
        let lex = lexema_at(tokens, *pos);
        *pos += 1;
        AstNode::leaf(AstKind::Identifier, &lex, linha_at(tokens, *pos))
    } else {
        st_expressao(tokens, pos)
    }
}

// ═══════════════════════════════════════════════════════════════════
// Expressões
// ═══════════════════════════════════════════════════════════════════

// <expressao> ::= <atribuicao>
fn st_expressao(tokens: &[TokenInfo], pos: &mut usize) -> AstNode {
    st_atribuicao(tokens, pos)
}

// <atribuicao> ::= <expr-logica> <atribuicao'>
fn st_atribuicao(tokens: &[TokenInfo], pos: &mut usize) -> AstNode {
    let left = st_expr_logica(tokens, pos);
    st_atribuicao_resto(tokens, pos, left)
}

// <atribuicao'> ::= <op-atrib> <atribuicao> | ε
fn st_atribuicao_resto(tokens: &[TokenInfo], pos: &mut usize, left: AstNode) -> AstNode {
    let _follow = &[Token::PtVirgula, Token::FechaParen, Token::Virgula, Token::FechaColch];

    if *pos < tokens.len() && eh_op_atrib(tokens[*pos].kind) {
        let op = lexema_at(tokens, *pos);
        match_token(tokens, pos, tokens[*pos].kind);
        let right = st_atribuicao(tokens, pos);
        AstNode::with_token(AstKind::Assign, vec![left, right], &op, linha_at(tokens, *pos))
    } else {
        left
    }
}

// <expr-logica> ::= <expr-relacional> <expr-logica'>
fn st_expr_logica(tokens: &[TokenInfo], pos: &mut usize) -> AstNode {
    let left = st_expr_relacional(tokens, pos);
    st_expr_logica_resto(tokens, pos, left)
}

// <expr-logica'> ::= "&&" <expr-relacional> <expr-logica'>
//                  | "||" <expr-relacional> <expr-logica'> | ε
fn st_expr_logica_resto(tokens: &[TokenInfo], pos: &mut usize, left: AstNode) -> AstNode {
    let _follow = &[Token::PtVirgula, Token::FechaParen, Token::Virgula, Token::FechaColch, Token::Atribuicao];

    if *pos < tokens.len() && matches!(tokens[*pos].kind, Token::And | Token::Or) {
        let op = lexema_at(tokens, *pos);
        if !match_token(tokens, pos, Token::And) {
            match_token(tokens, pos, Token::Or);
        }
        let right = st_expr_relacional(tokens, pos);
        let binop = AstNode::with_token(AstKind::BinaryOp, vec![left, right], &op, linha_at(tokens, *pos));
        st_expr_logica_resto(tokens, pos, binop)
    } else {
        left
    }
}

// <expr-relacional> ::= <expr-aritmetica> <expr-relacional'>
fn st_expr_relacional(tokens: &[TokenInfo], pos: &mut usize) -> AstNode {
    let left = st_expr_aritmetica(tokens, pos);
    st_expr_relacional_resto(tokens, pos, left)
}

// <expr-relacional'> ::= <op-relacional> <expr-aritmetica> <expr-relacional'> | ε
fn st_expr_relacional_resto(tokens: &[TokenInfo], pos: &mut usize, left: AstNode) -> AstNode {
    let _follow = &[Token::PtVirgula, Token::FechaParen, Token::Virgula, Token::FechaColch, Token::And, Token::Or, Token::Atribuicao];

    if *pos < tokens.len() && eh_op_relacional(tokens[*pos].kind) {
        let op = lexema_at(tokens, *pos);
        match_token(tokens, pos, tokens[*pos].kind);
        let right = st_expr_aritmetica(tokens, pos);
        let binop = AstNode::with_token(AstKind::BinaryOp, vec![left, right], &op, linha_at(tokens, *pos));
        st_expr_relacional_resto(tokens, pos, binop)
    } else {
        left
    }
}

// <expr-aritmetica> ::= <termo> <expr-aritmetica'>
fn st_expr_aritmetica(tokens: &[TokenInfo], pos: &mut usize) -> AstNode {
    let left = st_termo(tokens, pos);
    st_expr_aritmetica_resto(tokens, pos, left)
}

// <expr-aritmetica'> ::= "+" <termo> <expr-aritmetica'>
//                      | "-" <termo> <expr-aritmetica'> | ε
fn st_expr_aritmetica_resto(tokens: &[TokenInfo], pos: &mut usize, left: AstNode) -> AstNode {
    let _follow = &[Token::PtVirgula, Token::FechaParen, Token::Virgula, Token::FechaColch, Token::And, Token::Or, Token::Igual, Token::Diferente, Token::Menor, Token::Maior, Token::Atribuicao];

    if *pos < tokens.len() && matches!(tokens[*pos].kind, Token::Adicao | Token::Subtracao) {
        let op = lexema_at(tokens, *pos);
        if !match_token(tokens, pos, Token::Adicao) {
            match_token(tokens, pos, Token::Subtracao);
        }
        let right = st_termo(tokens, pos);
        let binop = AstNode::with_token(AstKind::BinaryOp, vec![left, right], &op, linha_at(tokens, *pos));
        st_expr_aritmetica_resto(tokens, pos, binop)
    } else {
        left
    }
}

// <termo> ::= <factor> <termo'>
fn st_termo(tokens: &[TokenInfo], pos: &mut usize) -> AstNode {
    let left = st_factor(tokens, pos);
    st_termo_resto(tokens, pos, left)
}

// <termo'> ::= "*" <factor> <termo'>
//            | "/" <factor> <termo'> | "%" <factor> <termo'> | ε
fn st_termo_resto(tokens: &[TokenInfo], pos: &mut usize, left: AstNode) -> AstNode {
    let _follow = &[Token::PtVirgula, Token::FechaParen, Token::Virgula, Token::FechaColch, Token::Adicao, Token::Subtracao, Token::And, Token::Or, Token::Igual, Token::Diferente, Token::Menor, Token::Maior, Token::Atribuicao];

    if *pos < tokens.len() && matches!(tokens[*pos].kind, Token::Multiplicacao | Token::Divisao | Token::Modulo) {
        let op = lexema_at(tokens, *pos);
        match_token(tokens, pos, tokens[*pos].kind);
        let right = st_factor(tokens, pos);
        let binop = AstNode::with_token(AstKind::BinaryOp, vec![left, right], &op, linha_at(tokens, *pos));
        st_termo_resto(tokens, pos, binop)
    } else {
        left
    }
}

// <factor> ::= <unario>
fn st_factor(tokens: &[TokenInfo], pos: &mut usize) -> AstNode {
    st_unario(tokens, pos)
}

// <unario> ::= <postfixo>
//            | "!" <unario> | "-" <unario> | "++" <unario> | "--" <unario>
//            | "&" <unario> | "*" <unario>
fn st_unario(tokens: &[TokenInfo], pos: &mut usize) -> AstNode {
    if *pos < tokens.len() {
        match tokens[*pos].kind {
            Token::Not | Token::Subtracao | Token::Incremento | Token::Decremento
            | Token::BitAnd | Token::Multiplicacao | Token::BitNot => {
                let op = lexema_at(tokens, *pos);
                match_token(tokens, pos, tokens[*pos].kind);
                let operand = st_unario(tokens, pos);
                AstNode::with_token(AstKind::UnaryOp, vec![operand], &op, linha_at(tokens, *pos))
            }
            _ => st_postfixo(tokens, pos),
        }
    } else {
        AstNode::new(AstKind::Error, 0)
    }
}

// <postfixo> ::= <primario> <postfixo'>
fn st_postfixo(tokens: &[TokenInfo], pos: &mut usize) -> AstNode {
    let left = st_primario(tokens, pos);
    st_postfixo_resto(tokens, pos, left)
}

// <postfixo'> ::= "++" <postfixo'>
//              | "--" <postfixo'> | "[" <expressao> "]" <postfixo'> | "(" <argumentos> ")" <postfixo'>
//              | "." <identificador> <postfixo'> | "->" <identificador> <postfixo'> | ε
fn st_postfixo_resto(tokens: &[TokenInfo], pos: &mut usize, left: AstNode) -> AstNode {
    let follow = &[Token::PtVirgula, Token::FechaParen, Token::Virgula, Token::FechaColch, Token::And, Token::Or, Token::Igual, Token::Diferente];

    if *pos >= tokens.len() {
        return left;
    }

    match tokens[*pos].kind {
        Token::Incremento | Token::Decremento => {
            let is_inc = tokens[*pos].kind == Token::Incremento;
            match_token(tokens, pos, tokens[*pos].kind);
            let prefix = if is_inc { "post++" } else { "post--" };
            let unary = AstNode::with_token(AstKind::UnaryOp, vec![left], prefix, linha_at(tokens, *pos));
            st_postfixo_resto(tokens, pos, unary)
        }
        Token::AbreColch => {
            match_token(tokens, pos, Token::AbreColch);
            let index_expr = st_expressao(tokens, pos);
            if !expect(tokens, pos, Token::FechaColch, "]", follow) {
                return AstNode::new(AstKind::Error, 0);
            }
            let index = AstNode::with_children(AstKind::Index, vec![left, index_expr], linha_at(tokens, *pos));
            st_postfixo_resto(tokens, pos, index)
        }
        Token::AbreParen => {
            match_token(tokens, pos, Token::AbreParen);
            let args = st_argumentos(tokens, pos);
            if !expect(tokens, pos, Token::FechaParen, ")", follow) {
                return AstNode::new(AstKind::Error, 0);
            }
            let mut call_children = vec![left];
            call_children.extend(args);
            let call = AstNode::with_children(AstKind::Call, call_children, linha_at(tokens, *pos));
            st_postfixo_resto(tokens, pos, call)
        }
        Token::PontoMembro => {
            match_token(tokens, pos, Token::PontoMembro);
            let member_lex = lexema_at(tokens, *pos);
            if !expect(tokens, pos, Token::Identificador, "identificador", follow) {
                return AstNode::new(AstKind::Error, 0);
            }
            let member = AstNode::leaf(AstKind::Identifier, &member_lex, linha_at(tokens, *pos));
            let access = AstNode::with_children(AstKind::MemberAccess, vec![left, member], linha_at(tokens, *pos));
            st_postfixo_resto(tokens, pos, access)
        }
        Token::Seta => {
            match_token(tokens, pos, Token::Seta);
            let member_lex = lexema_at(tokens, *pos);
            if !expect(tokens, pos, Token::Identificador, "identificador", follow) {
                return AstNode::new(AstKind::Error, 0);
            }
            let member = AstNode::leaf(AstKind::Identifier, &member_lex, linha_at(tokens, *pos));
            let access = AstNode::with_children(AstKind::PtrAccess, vec![left, member], linha_at(tokens, *pos));
            st_postfixo_resto(tokens, pos, access)
        }
        _ => left,
    }
}

// <primario> ::= <literal>
//             | <identificador> | "this" | "(" <expressao> ")"
//             | "new" <tipo> <new-sufixo>
fn st_primario(tokens: &[TokenInfo], pos: &mut usize) -> AstNode {
    if *pos >= tokens.len() {
        return AstNode::new(AstKind::Error, 0);
    }

    match tokens[*pos].kind {
        Token::Inteiro | Token::Float | Token::Char | Token::String | Token::TrueLiteral | Token::FalseLiteral => {
            let lex = lexema_at(tokens, *pos);
            let tok = tokens[*pos].kind;
            *pos += 1;
            AstNode::with_original_token(AstKind::Literal, &lex, tok, linha_at(tokens, *pos))
        }
        Token::Identificador => {
            let lex = lexema_at(tokens, *pos);
            *pos += 1;
            AstNode::leaf(AstKind::Identifier, &lex, linha_at(tokens, *pos))
        }
        Token::This => {
            *pos += 1;
            AstNode::new(AstKind::This, linha_at(tokens, *pos))
        }
        Token::AbreParen => {
            *pos += 1;
            let expr = st_expressao(tokens, pos);
            let follow = &[Token::AbreColch, Token::AbreParen, Token::PontoMembro, Token::Seta, Token::Incremento, Token::Decremento, Token::PtVirgula, Token::Virgula, Token::FechaParen];
            expect(tokens, pos, Token::FechaParen, ")", follow);
            expr
        }
        Token::New => {
            *pos += 1;
            let type_node = st_tipo(tokens, pos);
            let sufixo = st_new_sufixo(tokens, pos);
            AstNode::with_children(AstKind::New, vec![type_node, sufixo], linha_at(tokens, *pos))
        }
        _ => {
            let follow = &[Token::AbreColch, Token::AbreParen, Token::PontoMembro, Token::Seta, Token::Incremento, Token::Decremento, Token::PtVirgula, Token::Virgula, Token::FechaParen, Token::And, Token::Or, Token::Igual, Token::Diferente, Token::Menor, Token::Maior, Token::Adicao, Token::Subtracao, Token::Multiplicacao, Token::Divisao, Token::Modulo];
            panic_mode_recovery(tokens, pos, "primário", follow);
            AstNode::new(AstKind::Error, 0)
        }
    }
}

// <new-sufixo> ::= "(" <argumentos> ")" | "[" <expressao> "]"
fn st_new_sufixo(tokens: &[TokenInfo], pos: &mut usize) -> AstNode {
    let follow = &[Token::AbreColch, Token::AbreParen, Token::PontoMembro, Token::Seta, Token::Incremento, Token::Decremento, Token::PtVirgula, Token::Virgula];

    if *pos < tokens.len() && tokens[*pos].kind == Token::AbreParen {
        match_token(tokens, pos, Token::AbreParen);
        let args = st_argumentos(tokens, pos);
        if !expect(tokens, pos, Token::FechaParen, ")", follow) {
            return AstNode::new(AstKind::Error, 0);
        }
        AstNode::with_children(AstKind::Call, args, linha_at(tokens, *pos))

    } else if *pos < tokens.len() && tokens[*pos].kind == Token::AbreColch {
        match_token(tokens, pos, Token::AbreColch);
        let expr = st_expressao(tokens, pos);
        if !expect(tokens, pos, Token::FechaColch, "]", follow) {
            return AstNode::new(AstKind::Error, 0);
        }
        AstNode::with_children(AstKind::Index, vec![expr], linha_at(tokens, *pos))

    } else {
        AstNode::new(AstKind::Error, 0)
    }
}

// <argumentos> ::= <lista-args> | ε
fn st_argumentos(tokens: &[TokenInfo], pos: &mut usize) -> Vec<AstNode> {
    let follow = &[Token::FechaParen];

    if *pos < tokens.len() && pode_iniciar_expressao(tokens, *pos) {
        st_lista_args(tokens, pos)
    } else if *pos >= tokens.len() || follow.contains(&tokens[*pos].kind) {
        vec![]
    } else {
        panic_mode_recovery(tokens, pos, "expressão ou ')'", follow);
        vec![]
    }
}

// <lista-args> ::= <expressao> <lista-args'>
fn st_lista_args(tokens: &[TokenInfo], pos: &mut usize) -> Vec<AstNode> {
    let mut nodes = vec![st_expressao(tokens, pos)];
    nodes.extend(st_lista_args_resto(tokens, pos));
    nodes
}

// <lista-args'> ::= "," <expressao> <lista-args'> | ε
fn st_lista_args_resto(tokens: &[TokenInfo], pos: &mut usize) -> Vec<AstNode> {
    let follow = &[Token::FechaParen];

    if *pos < tokens.len() && tokens[*pos].kind == Token::Virgula {
        *pos += 1;
        let mut nodes = vec![st_expressao(tokens, pos)];
        nodes.extend(st_lista_args_resto(tokens, pos));
        nodes
    } else if *pos >= tokens.len() || follow.contains(&tokens[*pos].kind) {
        vec![]
    } else {
        panic_mode_recovery(tokens, pos, "',' ou ')'", follow);
        vec![]
    }
}

// <literal> ::= <inteiro> | <real> | <caracter> | <cadeia> | "true" | "false" | "nullptr"
fn st_literal(tokens: &[TokenInfo], pos: &mut usize) -> AstNode {
    if *pos >= tokens.len() {
        return AstNode::leaf(AstKind::Literal, "", linha_at(tokens, *pos));
    }

    match tokens[*pos].kind {
        Token::Inteiro | Token::Float | Token::Char | Token::String | Token::TrueLiteral
        | Token::FalseLiteral => {
            let lex = lexema_at(tokens, *pos);
            let tok = tokens[*pos].kind;
            *pos += 1;
            AstNode::with_original_token(AstKind::Literal, &lex, tok, linha_at(tokens, *pos))
        }
        _ => {
            let follow = &[Token::AbreColch, Token::AbreParen, Token::PontoMembro, Token::Seta, Token::Incremento, Token::Decremento, Token::PtVirgula, Token::Virgula, Token::FechaParen, Token::And, Token::Or, Token::Igual, Token::Diferente, Token::Menor, Token::Maior, Token::Adicao, Token::Subtracao, Token::Multiplicacao, Token::Divisao, Token::Modulo];
            panic_mode_recovery(tokens, pos, "literal esperado", follow);
            AstNode::leaf(AstKind::Literal, "", linha_at(tokens, *pos))
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
// Entrada pública
// ═══════════════════════════════════════════════════════════════════

pub fn analisar(tokens: Vec<TokenInfo>) -> AstNode {
    println!("Iniciando analise sintatica...");
    ERROR_COUNT.store(0, Ordering::SeqCst);
    let mut pos = 0;

    let ast = st_programa(&tokens, &mut pos);

    let total_erros = ERROR_COUNT.load(Ordering::SeqCst);

    if pos < tokens.len() {
        println!("Aviso: O analisador parou prematuramente na posicao {}. Tokens restantes ignorados.", pos);
    } else if total_erros > 0 {
        println!("Analise sintatica concluida com {} erro(s).", total_erros);
    } else {
        println!("Analise sintatica bem-sucedida!");
    }

    ast
}
