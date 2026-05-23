use crate::token::{Token, TokenInfo};

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
    
    while *pos < tokens.len() && !follow.contains(&tokens[*pos].kind) {
        *pos += 1;
    }
}

// <programa> ::= <include-seq> <declaracao-seq>
fn st_programa(tokens: &[TokenInfo], pos: &mut usize) {
    st_include_seq(tokens, pos);
    st_declaracao_seq(tokens, pos);
}

// <include-seq> ::= <include> <include-seq> | ε
fn st_include_seq(tokens: &[TokenInfo], pos: &mut usize) {
    let follow = &[Token::Class, Token::Struct, Token::Int, Token::FloatType, Token::CharType, 
                   Token::Double, Token::Bool, Token::Void, Token::Public, Token::Private, 
                   Token::Protected, Token::Static, Token::Const, Token::Identificador];
    
    if *pos < tokens.len() && matches!(tokens[*pos].kind, Token::Include) {
        st_include(tokens, pos);
        st_include_seq(tokens, pos);
    } 
    else if *pos >= tokens.len() || follow.contains(&tokens[*pos].kind) {
    } 
    else {
        panic_mode_recovery(tokens, pos, "#include ou declaração", follow);
    }
}

// <include> ::= "#include" <include-alvo>
fn st_include(tokens: &[TokenInfo], pos: &mut usize) {
    // Follow de <include>: outro Include ou qualquer declaração
    let follow = &[
        Token::Include, Token::Class, Token::Struct, Token::Int, Token::FloatType,
        Token::CharType, Token::Double, Token::Bool, Token::Void, Token::Public,
        Token::Private, Token::Protected, Token::Static, Token::Const, Token::Identificador,
    ];
    
    if !expect(tokens, pos, Token::Include, "#include", follow) {
        return;
    }
    st_include_alvo(tokens, pos);
}

// <include-alvo> ::= "<" <Cadeia> ">" | '"' <Cadeia> '"'
fn st_include_alvo(tokens: &[TokenInfo], pos: &mut usize) {
    // Follow de <include-alvo>: tokens que podem vir depois
    let follow = &[
        Token::Include, Token::Class, Token::Struct, Token::Int, Token::FloatType,
        Token::CharType, Token::Double, Token::Bool, Token::Void,
    ];
    
    if match_token(tokens, pos, Token::Menor) {
        if !expect(tokens, pos, Token::String, "nome da biblioteca", follow) {
            return;
        }
        if !expect(tokens, pos, Token::Maior, ">", follow) {
            return;
        }
    } else if match_token(tokens, pos, Token::String) {
        // Arquivo tipo "lib.h" é capturado como String no lexer
    } else {
        eprintln!("Erro: Include mal formatado");
    }
}

// <declaracao-seq> ::= <declaracao> <declaracao-seq> | ε
fn st_declaracao_seq(tokens: &[TokenInfo], pos: &mut usize) {
    let follow = &[]; // EOF
    if *pos < tokens.len() && pode_iniciar_declaracao(tokens, *pos) {
        st_declaracao(tokens, pos);
        st_declaracao_seq(tokens, pos);
    }
    else if *pos >= tokens.len() {
    }
    else {
        panic_mode_recovery(tokens, pos, "declaração ou EOF", follow);
    }
}

fn pode_iniciar_declaracao(tokens: &[TokenInfo], pos: usize) -> bool {
    if pos >= tokens.len() {
        return false;
    }
    matches!(
        tokens[pos].kind,
        Token::Class | Token::Struct | Token::Int | Token::FloatType
            | Token::CharType | Token::Double | Token::Bool | Token::Void
            | Token::Public | Token::Private | Token::Protected
            | Token::Static | Token::Const | Token::Identificador
    )
}

// <declaracao> ::= <declaracao-classe> | <modificador> <tipo> <identificador> <sufixo-decl>
fn st_declaracao(tokens: &[TokenInfo], pos: &mut usize) {
    let follow = &[Token::Class, Token::Struct, Token::Int, Token::FloatType, Token::CharType, 
                   Token::Double, Token::Bool, Token::Void, Token::Public, Token::Private, 
                   Token::Protected, Token::Static, Token::Const, Token::Identificador];
    
    if *pos < tokens.len() && matches!(tokens[*pos].kind, Token::Class | Token::Struct) {
        st_declaracao_classe(tokens, pos);
    } else {
        st_modificador(tokens, pos);
        st_tipo(tokens, pos);
        if !expect(tokens, pos, Token::Identificador, "identificador", follow) {
            return;
        }
        st_sufixo_decl(tokens, pos);
    }
}

// <modificador> ::= "public" | "private" | "protected" | "static" | "const" | ε
fn st_modificador(tokens: &[TokenInfo], pos: &mut usize) {
    if *pos < tokens.len() {
        match tokens[*pos].kind {
            Token::Public | Token::Private | Token::Protected | Token::Static | Token::Const
            | Token::Virtual | Token::Override | Token::Signed | Token::Unsigned
            | Token::Inline | Token::Explicit => {
                *pos += 1;
                st_modificador(tokens, pos);
            }
            _ => {}
        }
    }
}

// <tipo> ::= "int" | "float" | "double" | "char" | "bool" | "void" | "string" | <identificador>
fn st_tipo(tokens: &[TokenInfo], pos: &mut usize) {
    if *pos >= tokens.len() {
        eprintln!("Erro: Tipo esperado mas encontrado EOF");
        return;
    }
    match tokens[*pos].kind {
        Token::Int | Token::FloatType | Token::Double | Token::CharType | Token::Bool
        | Token::Void | Token::String => {
            *pos += 1;
        }
        Token::Identificador => {
            *pos += 1;
        }
        _ => {
            eprintln!("Erro [linha {}]: Tipo esperado", tokens[*pos].linha);
        }
    }
}

// <declaracao-classe> ::= "class" <identificador> <heranca> "{" <membros-classe> "}" ";"
fn st_declaracao_classe(tokens: &[TokenInfo], pos: &mut usize) {
    let follow = &[Token::Class, Token::Struct, Token::Int, Token::FloatType];
    
    if !match_token(tokens, pos, Token::Class) {
        match_token(tokens, pos, Token::Struct);
    }
    
    if !expect(tokens, pos, Token::Identificador, "nome da classe", follow) {
        return;
    }
    st_heranca(tokens, pos);
    if !expect(tokens, pos, Token::AbreChave, "{", follow) {
        return;
    }
    st_membros_classe(tokens, pos);
    if !expect(tokens, pos, Token::FechaChave, "}", follow) {
        return;
    }
    if !expect(tokens, pos, Token::PtVirgula, ";", follow) {
        return;
    }
}

// <heranca> ::= ":" <acesso> <identificador> | ε
fn st_heranca(tokens: &[TokenInfo], pos: &mut usize) {
    let follow = &[Token::AbreChave, Token::Public, Token::Private, Token::Protected];
    
    if *pos < tokens.len() && matches!(tokens[*pos].kind, Token::DoisPontos) {
        match_token(tokens, pos, Token::DoisPontos);
        st_acesso(tokens, pos);
        if !expect(tokens, pos, Token::Identificador, "nome da classe base", follow) {
            return;
        }
    }
    else if *pos >= tokens.len() || follow.contains(&tokens[*pos].kind) {
    }
    else {
        panic_mode_recovery(tokens, pos, "':' ou '{'", follow);
    }
}

// <acesso> ::= "public" | "private" | "protected"
fn st_acesso(tokens: &[TokenInfo], pos: &mut usize) {
    if *pos < tokens.len() {
        match tokens[*pos].kind {
            Token::Public | Token::Private | Token::Protected => {
                *pos += 1;
            }
            _ => {}
        }
    }
}

// <membros-classe> ::= <seccao-acesso> <membros-classe> | ε
fn st_membros_classe(tokens: &[TokenInfo], pos: &mut usize) {
    let follow = &[Token::FechaChave];
    
    if *pos < tokens.len() && pode_ser_secao_acesso(tokens, *pos) {
        st_seccao_acesso(tokens, pos);
        st_membros_classe(tokens, pos);
    }
    else if *pos >= tokens.len() || follow.contains(&tokens[*pos].kind) {
    }
    else {
        panic_mode_recovery(tokens, pos, "public/private/protected ou '}'", follow);
    }
}

fn pode_ser_secao_acesso(tokens: &[TokenInfo], pos: usize) -> bool {
    matches!(tokens[pos].kind, Token::Public | Token::Private | Token::Protected)
}

// <seccao-acesso> ::= <acesso> ":" <lista-membros>
fn st_seccao_acesso(tokens: &[TokenInfo], pos: &mut usize) {
    let follow = &[Token::Public, Token::Private, Token::Protected, Token::FechaChave];
    
    st_acesso(tokens, pos);
    if !expect(tokens, pos, Token::DoisPontos, ":", follow) {
        return;
    }
    st_lista_membros(tokens, pos);
}

// <lista-membros> ::= <membro> <lista-membros> | ε
fn st_lista_membros(tokens: &[TokenInfo], pos: &mut usize) {
    let follow = &[Token::FechaChave, Token::Public, Token::Private, Token::Protected];
    
    if *pos < tokens.len() && pode_iniciar_membro(tokens, *pos) {
        st_membro(tokens, pos);
        st_lista_membros(tokens, pos);
    }
    else if *pos >= tokens.len() || follow.contains(&tokens[*pos].kind) {
    }
    else {
        panic_mode_recovery(tokens, pos, "membro de classe", follow);
    }
}

fn pode_iniciar_membro(tokens: &[TokenInfo], pos: usize) -> bool {
    if pos >= tokens.len() {
        return false;
    }
    !matches!(tokens[pos].kind, Token::FechaChave | Token::Public | Token::Private | Token::Protected)
}

// <membro> ::= <declaracao-construtor> | <modificador> <tipo> <identificador> <sufixo-decl>
fn st_membro(tokens: &[TokenInfo], pos: &mut usize) {
    let follow = &[Token::Public, Token::Private, Token::Protected, Token::FechaChave];
    
    let checkpoint = *pos;
    
    // Tenta verificar se é um construtor (identificador sem tipo antes)
    if *pos < tokens.len() && std::mem::discriminant(&tokens[*pos].kind) == std::mem::discriminant(&Token::Identificador) {
        let next_pos = *pos + 1;
        if next_pos < tokens.len() && std::mem::discriminant(&tokens[next_pos].kind) == std::mem::discriminant(&Token::AbreParen) {
            st_declaracao_construtor(tokens, pos);
            return;
        }
    }
    
    *pos = checkpoint;
    st_modificador(tokens, pos);
    st_tipo(tokens, pos);
    if !expect(tokens, pos, Token::Identificador, "identificador", follow) {
        return;
    }
    st_sufixo_decl(tokens, pos);
}

// <declaracao-construtor> ::= <identificador> "(" <parametros> ")" <bloco>
fn st_declaracao_construtor(tokens: &[TokenInfo], pos: &mut usize) {
    let follow = &[Token::Public, Token::Private, Token::Protected, Token::FechaChave];
    
    if !expect(tokens, pos, Token::Identificador, "nome do construtor", follow) {
        return;
    }
    if !expect(tokens, pos, Token::AbreParen, "(", follow) {
        return;
    }
    st_parametros(tokens, pos);
    if !expect(tokens, pos, Token::FechaParen, ")", follow) {
        return;
    }
    st_bloco(tokens, pos);
}

// <sufixo-decl> ::= "(" <parametros> ")" <corpo-funcao> | <var-resto> ";"
fn st_sufixo_decl(tokens: &[TokenInfo], pos: &mut usize) {
    let follow = &[Token::Public, Token::Private, Token::Protected, Token::FechaChave, Token::PtVirgula];
    
    if match_token(tokens, pos, Token::AbreParen) {
        st_parametros(tokens, pos);
        if !expect(tokens, pos, Token::FechaParen, ")", follow) {
            return;
        }
        st_corpo_funcao(tokens, pos);
    } else {
        st_var_resto(tokens, pos);
        if !expect(tokens, pos, Token::PtVirgula, ";", follow) {
            return;
        }
    }
}

// <corpo-funcao> ::= <bloco> | ";"
fn st_corpo_funcao(tokens: &[TokenInfo], pos: &mut usize) {
    if *pos < tokens.len() && matches!(tokens[*pos].kind, Token::AbreChave) {
        st_bloco(tokens, pos);
    } else {
        match_token(tokens, pos, Token::PtVirgula);
    }
}

// <parametros> ::= <lista-parametros> | ε
fn st_parametros(tokens: &[TokenInfo], pos: &mut usize) {
    let follow = &[Token::FechaParen];
    
    if *pos < tokens.len() && pode_iniciar_parametro(tokens, *pos) {
        st_lista_parametros(tokens, pos);
    }
    else if *pos >= tokens.len() || follow.contains(&tokens[*pos].kind) {
    }
    else {
        panic_mode_recovery(tokens, pos, "tipo de parâmetro ou ')'", follow);
    }
}

fn pode_iniciar_parametro(tokens: &[TokenInfo], pos: usize) -> bool {
    matches!(
        tokens[pos].kind,
        Token::Int | Token::FloatType | Token::Double | Token::CharType | Token::Bool
            | Token::Void | Token::Identificador | Token::Const
    )
}

// <lista-parametros> ::= <parametro> <lista-parametros'>
fn st_lista_parametros(tokens: &[TokenInfo], pos: &mut usize) {
    st_parametro(tokens, pos);
    st_lista_parametros_resto(tokens, pos);
}

// <lista-parametros'> ::= "," <parametro> <lista-parametros'> | ε
fn st_lista_parametros_resto(tokens: &[TokenInfo], pos: &mut usize) {
    let follow = &[Token::FechaParen];
    
    if *pos < tokens.len() && matches!(tokens[*pos].kind, Token::Virgula) {
        match_token(tokens, pos, Token::Virgula);
        st_parametro(tokens, pos);
        st_lista_parametros_resto(tokens, pos);
    }
    else if *pos >= tokens.len() || follow.contains(&tokens[*pos].kind) {
    }
    else {
        panic_mode_recovery(tokens, pos, "',' ou ')'", follow);
    }
}

// <parametro> ::= <tipo> <identificador> <param-sufixo>
fn st_parametro(tokens: &[TokenInfo], pos: &mut usize) {
    let follow = &[Token::Virgula, Token::FechaParen];
    
    st_tipo(tokens, pos);
    if !expect(tokens, pos, Token::Identificador, "identificador", follow) {
        return;
    }
    st_param_sufixo(tokens, pos);
}

// <param-sufixo> ::= "[" "]" | ε
fn st_param_sufixo(tokens: &[TokenInfo], pos: &mut usize) {
    let follow = &[Token::Virgula, Token::FechaParen, Token::AbreParen];
    
    if *pos < tokens.len() && matches!(tokens[*pos].kind, Token::AbreColch) {
        match_token(tokens, pos, Token::AbreColch);
        if !expect(tokens, pos, Token::FechaColch, "]", follow) {
            return;
        }
    }
    else if *pos >= tokens.len() || follow.contains(&tokens[*pos].kind) {
    }
}

// <var-resto> ::= "=" <expressao> <lista-variaveis'> | "[" <dim-conteudo> "]" <mais-dims> <init-array> <lista-variaveis'> | <lista-variaveis'>
fn st_var_resto(tokens: &[TokenInfo], pos: &mut usize) {
    let follow = &[Token::Virgula, Token::PtVirgula, Token::AbreParen];
    
    if *pos < tokens.len() && matches!(tokens[*pos].kind, Token::Atribuicao) {
        match_token(tokens, pos, Token::Atribuicao);
        st_expressao(tokens, pos);
        st_lista_variaveis_resto(tokens, pos);
    } else if *pos < tokens.len() && matches!(tokens[*pos].kind, Token::AbreColch) {
        match_token(tokens, pos, Token::AbreColch);
        st_dim_conteudo(tokens, pos);
        if !expect(tokens, pos, Token::FechaColch, "]", follow) {
            return;
        }
        st_mais_dims(tokens, pos);
        st_init_array(tokens, pos);
        st_lista_variaveis_resto(tokens, pos);
    } else if *pos >= tokens.len() || follow.contains(&tokens[*pos].kind) {
        st_lista_variaveis_resto(tokens, pos);
    } else {
        eprintln!("Erro [linha {}]: Token inesperado em var-resto: {:?} (`{}`)", 
            tokens[*pos].linha, tokens[*pos].kind, tokens[*pos].lexema);
    }
}

// <lista-variaveis'> ::= "," <identificador> <var-resto> | ε
fn st_lista_variaveis_resto(tokens: &[TokenInfo], pos: &mut usize) {
    let follow = &[Token::PtVirgula, Token::AbreParen, Token::Virgula];
    
    if *pos < tokens.len() && matches!(tokens[*pos].kind, Token::Virgula) {
        match_token(tokens, pos, Token::Virgula);
        if !expect(tokens, pos, Token::Identificador, "identificador", follow) {
            return;
        }
        st_var_resto(tokens, pos);
        st_lista_variaveis_resto(tokens, pos);
    }
    else if *pos >= tokens.len() || follow.contains(&tokens[*pos].kind) {
    }
}

// <dim-conteudo> ::= <expressao> | ε
fn st_dim_conteudo(tokens: &[TokenInfo], pos: &mut usize) {
    let follow = &[Token::FechaColch];
    
    if *pos < tokens.len() && pode_iniciar_expressao(tokens, *pos) {
        st_expressao(tokens, pos);
    }
    else if *pos >= tokens.len() || follow.contains(&tokens[*pos].kind) {
    }
}

// <mais-dims> ::= "[" <dim-conteudo> "]" <mais-dims> | ε
fn st_mais_dims(tokens: &[TokenInfo], pos: &mut usize) {
    let follow = &[Token::Atribuicao, Token::Virgula, Token::PtVirgula];
    
    if *pos < tokens.len() && matches!(tokens[*pos].kind, Token::AbreColch) {
        match_token(tokens, pos, Token::AbreColch);
        st_dim_conteudo(tokens, pos);
        if !expect(tokens, pos, Token::FechaColch, "]", follow) {
            return;
        }
        st_mais_dims(tokens, pos);
    }
    else if *pos >= tokens.len() || follow.contains(&tokens[*pos].kind) {
    }
}

// <init-array> ::= "=" "{" <lista-init> "}" | ε
fn st_init_array(tokens: &[TokenInfo], pos: &mut usize) {
    let follow = &[Token::Virgula, Token::PtVirgula];
    
    if *pos < tokens.len() && matches!(tokens[*pos].kind, Token::Atribuicao) {
        match_token(tokens, pos, Token::Atribuicao);
        if !expect(tokens, pos, Token::AbreChave, "{", follow) {
            return;
        }
        st_lista_init(tokens, pos);
        if !expect(tokens, pos, Token::FechaChave, "}", follow) {
            return;
        }
    }
    else if *pos >= tokens.len() || follow.contains(&tokens[*pos].kind) {
    }
}

// <lista-init> ::= <expressao> <lista-init'> | ε
fn st_lista_init(tokens: &[TokenInfo], pos: &mut usize) {
    let follow = &[Token::FechaChave];
    
    if *pos < tokens.len() && pode_iniciar_expressao(tokens, *pos) {
        st_expressao(tokens, pos);
        st_lista_init_resto(tokens, pos);
    }
    else if *pos >= tokens.len() || follow.contains(&tokens[*pos].kind) {
    }
}

// <lista-init'> ::= "," <expressao> <lista-init'> | ε
fn st_lista_init_resto(tokens: &[TokenInfo], pos: &mut usize) {
    let follow = &[Token::FechaChave];
    
    if *pos < tokens.len() && matches!(tokens[*pos].kind, Token::Virgula) {
        match_token(tokens, pos, Token::Virgula);
        if pode_iniciar_expressao(tokens, *pos) {
            st_expressao(tokens, pos);
        }
        st_lista_init_resto(tokens, pos);
    }
    else if *pos >= tokens.len() || follow.contains(&tokens[*pos].kind) {
    }
}

// <bloco> ::= "{" <comando-seq> "}"
fn st_bloco(tokens: &[TokenInfo], pos: &mut usize) {
    let follow = &[Token::Public, Token::Private, Token::Protected, Token::FechaChave, Token::Else, Token::PtVirgula];
    
    if !expect(tokens, pos, Token::AbreChave, "{", follow) {
        return;
    }
    st_comando_seq(tokens, pos);
    if !expect(tokens, pos, Token::FechaChave, "}", follow) {
        return;
    }
}

// <comando-seq> ::= <comando> <comando-seq> | ε
fn st_comando_seq(tokens: &[TokenInfo], pos: &mut usize) {
    let follow = &[Token::FechaChave];
    
    if *pos < tokens.len() && pode_iniciar_comando(tokens, *pos) {
        st_comando(tokens, pos);
        st_comando_seq(tokens, pos);
    }
    else if *pos >= tokens.len() || follow.contains(&tokens[*pos].kind) {
    }
    else {
        panic_mode_recovery(tokens, pos, "comando ou '}'", follow);
    }
}

fn pode_iniciar_comando(tokens: &[TokenInfo], pos: usize) -> bool {
    if pos >= tokens.len() {
        return false;
    }
    !matches!(tokens[pos].kind, Token::FechaChave)
}

// <comando> ::= <modificador> <tipo> <identificador> <sufixo-decl>
//             | <comando-seleccao> | <comando-repeticao> | <comando-io>
//             | "return" <expressao-opt> ";" | "break" ";" | "continue" ";" | <bloco> | <expressao> ";"
fn st_comando(tokens: &[TokenInfo], pos: &mut usize) {
    let follow = &[Token::FechaChave, Token::PtVirgula, Token::Else, Token::Case, Token::Default];
    
    if *pos >= tokens.len() {
        return;
    }
    
    match tokens[*pos].kind {
        Token::If => st_comando_seleccao(tokens, pos),
        Token::Switch => st_comando_seleccao(tokens, pos),
        Token::While => st_comando_repeticao(tokens, pos),
        Token::Do => st_comando_repeticao(tokens, pos),
        Token::For => st_comando_repeticao(tokens, pos),
        Token::Cin => st_comando_io(tokens, pos),
        Token::Cout => st_comando_io(tokens, pos),
        Token::Return => {
            *pos += 1;
            st_expressao_opt(tokens, pos);
            if !expect(tokens, pos, Token::PtVirgula, ";", follow) {
                return;
            }
        }
        Token::Break => {
            *pos += 1;
            if !expect(tokens, pos, Token::PtVirgula, ";", follow) {
                return;
            }
        }
        Token::Continue => {
            *pos += 1;
            if !expect(tokens, pos, Token::PtVirgula, ";", follow) {
                return;
            }
        }
        Token::AbreChave => st_bloco(tokens, pos),
        Token::Int | Token::FloatType | Token::Double | Token::CharType | Token::Bool | Token::Void => {
            st_modificador(tokens, pos);
            st_tipo(tokens, pos);
            if !expect(tokens, pos, Token::Identificador, "identificador", follow) {
                return;
            }
            st_sufixo_decl(tokens, pos);
        }
        _ => {
            st_expressao(tokens, pos);
            if !expect(tokens, pos, Token::PtVirgula, ";", follow) {
                return;
            }
        }
    }
}

// <comando-seleccao> ::= "if" "(" <expressao> ")" <bloco> <else-parte>
//                      | "switch" "(" <expressao> ")" "{" <casos> "}"
fn st_comando_seleccao(tokens: &[TokenInfo], pos: &mut usize) {
    let follow = &[Token::FechaChave, Token::Else, Token::PtVirgula];
    
    if match_token(tokens, pos, Token::If) {
        if !expect(tokens, pos, Token::AbreParen, "(", follow) { return; }
        st_expressao(tokens, pos);
        if !expect(tokens, pos, Token::FechaParen, ")", follow) { return; }
        st_bloco(tokens, pos);
        st_else_parte(tokens, pos);
    } else if match_token(tokens, pos, Token::Switch) {
        if !expect(tokens, pos, Token::AbreParen, "(", follow) { return; }
        st_expressao(tokens, pos);
        if !expect(tokens, pos, Token::FechaParen, ")", follow) { return; }
        if !expect(tokens, pos, Token::AbreChave, "{", follow) { return; }
        st_casos(tokens, pos);
        if !expect(tokens, pos, Token::FechaChave, "}", follow) { return; }
    }
}

// <else-parte> ::= "else" <else-corpo> | ε
fn st_else_parte(tokens: &[TokenInfo], pos: &mut usize) {
    let follow = &[Token::FechaChave, Token::PtVirgula, Token::Else, Token::Case, Token::Default];
    
    if *pos < tokens.len() && matches!(tokens[*pos].kind, Token::Else) {
        match_token(tokens, pos, Token::Else);
        st_else_corpo(tokens, pos);
    }
    else if *pos >= tokens.len() || follow.contains(&tokens[*pos].kind) {
    }
}

// <else-corpo> ::= "if" "(" <expressao> ")" <bloco> <else-parte> | <bloco>
fn st_else_corpo(tokens: &[TokenInfo], pos: &mut usize) {
    let follow = &[Token::FechaChave, Token::Else, Token::PtVirgula];
    
    if *pos < tokens.len() && matches!(tokens[*pos].kind, Token::If) {
        *pos += 1;
        if !expect(tokens, pos, Token::AbreParen, "(", follow) { return; }
        st_expressao(tokens, pos);
        if !expect(tokens, pos, Token::FechaParen, ")", follow) { return; }
        st_bloco(tokens, pos);
        st_else_parte(tokens, pos);
    } else {
        st_bloco(tokens, pos);
    }
}

// <casos> ::= <caso> <casos> | ε
fn st_casos(tokens: &[TokenInfo], pos: &mut usize) {
    let follow = &[Token::FechaChave, Token::Case, Token::Default];
    
    if *pos < tokens.len() && matches!(tokens[*pos].kind, Token::Case | Token::Default) {
        st_caso(tokens, pos);
        st_casos(tokens, pos);
    }
    else if *pos >= tokens.len() || follow.contains(&tokens[*pos].kind) {
    }
    else {
        panic_mode_recovery(tokens, pos, "case ou default", follow);
    }
}

// <caso> ::= "case" <literal> ":" <comando-seq>
//           | "default" ":" <comando-seq>
fn st_caso(tokens: &[TokenInfo], pos: &mut usize) {
    let follow = &[Token::Case, Token::Default, Token::FechaChave];
    
    if match_token(tokens, pos, Token::Case) {
        st_literal(tokens, pos);
        if !expect(tokens, pos, Token::DoisPontos, ":", follow) { return; }
        st_comando_seq(tokens, pos);
    } else if match_token(tokens, pos, Token::Default) {
        if !expect(tokens, pos, Token::DoisPontos, ":", follow) { return; }
        st_comando_seq(tokens, pos);
    }
}

// <comando-repeticao> ::= "while" "(" <expressao> ")" <bloco>
//                       | "do" <bloco> "while" "(" <expressao> ")" ";"
//                       | "for" "(" <for-init> <expressao-opt> ";" <expressao-opt> ")" <bloco>
fn st_comando_repeticao(tokens: &[TokenInfo], pos: &mut usize) {
    let follow = &[Token::FechaChave, Token::Else, Token::PtVirgula];
    
    if match_token(tokens, pos, Token::While) {
        if !expect(tokens, pos, Token::AbreParen, "(", follow) { return; }
        st_expressao(tokens, pos);
        if !expect(tokens, pos, Token::FechaParen, ")", follow) { return; }
        st_bloco(tokens, pos);
    } else if match_token(tokens, pos, Token::Do) {
        st_bloco(tokens, pos);
        if !expect(tokens, pos, Token::While, "while", follow) { return; }
        if !expect(tokens, pos, Token::AbreParen, "(", follow) { return; }
        st_expressao(tokens, pos);
        if !expect(tokens, pos, Token::FechaParen, ")", follow) { return; }
        if !expect(tokens, pos, Token::PtVirgula, ";", follow) { return; }
    } else if match_token(tokens, pos, Token::For) {
        if !expect(tokens, pos, Token::AbreParen, "(", follow) { return; }
        st_for_init(tokens, pos);
        st_expressao_opt(tokens, pos);
        if !expect(tokens, pos, Token::PtVirgula, ";", follow) { return; }
        st_expressao_opt(tokens, pos);
        if !expect(tokens, pos, Token::FechaParen, ")", follow) { return; }
        st_bloco(tokens, pos);
    }
}

// <for-init> ::= <tipo> <identificador> <var-resto> ";"
//              | <expressao> ";"
//              | ";"
fn st_for_init(tokens: &[TokenInfo], pos: &mut usize) {
    let follow = &[Token::PtVirgula];
    
    if *pos < tokens.len() && pode_iniciar_tipo(tokens, *pos) {
        st_tipo(tokens, pos);
        if !expect(tokens, pos, Token::Identificador, "identificador", follow) { return; }
        st_var_resto(tokens, pos);
        if !expect(tokens, pos, Token::PtVirgula, ";", follow) { return; }
    } else if *pos < tokens.len() && pode_iniciar_expressao(tokens, *pos) {
        st_expressao(tokens, pos);
        if !expect(tokens, pos, Token::PtVirgula, ";", follow) { return; }
    } else {
        match_token(tokens, pos, Token::PtVirgula);
    }
}

fn pode_iniciar_tipo(tokens: &[TokenInfo], pos: usize) -> bool {
    matches!(
        tokens[pos].kind,
        Token::Int | Token::FloatType | Token::Double | Token::CharType | Token::Bool
            | Token::Void | Token::String | Token::Identificador
    )
}

// <expressao-opt> ::= <expressao> | ε
fn st_expressao_opt(tokens: &[TokenInfo], pos: &mut usize) {
    let follow = &[Token::PtVirgula, Token::FechaParen];
    
    if *pos < tokens.len() && pode_iniciar_expressao(tokens, *pos) {
        st_expressao(tokens, pos);
    }
    else if *pos >= tokens.len() || follow.contains(&tokens[*pos].kind) {
    }
    else {
        panic_mode_recovery(tokens, pos, "expressão", follow);
    }
}

// <comando-io> ::= "cin" <cin-cadeia> ";"
//               | "cout" <cout-cadeia> ";"
fn st_comando_io(tokens: &[TokenInfo], pos: &mut usize) {
    let follow = &[Token::FechaChave, Token::Else, Token::PtVirgula, Token::Case, Token::Default];
    
    if match_token(tokens, pos, Token::Cin) {
        st_cin_cadeia(tokens, pos);
        if !expect(tokens, pos, Token::PtVirgula, ";", follow) { return; }
    } else if match_token(tokens, pos, Token::Cout) {
        st_cout_cadeia(tokens, pos);
        if !expect(tokens, pos, Token::PtVirgula, ";", follow) { return; }
    }
}

// <cin-cadeia> ::= ">>" <expressao> <cin-cadeia'>
fn st_cin_cadeia(tokens: &[TokenInfo], pos: &mut usize) {
    match_token(tokens, pos, Token::ShiftDir);
    st_expressao(tokens, pos);
    st_cin_cadeia_resto(tokens, pos);
}

// <cin-cadeia'> ::= ">>" <expressao> <cin-cadeia'> | ε
fn st_cin_cadeia_resto(tokens: &[TokenInfo], pos: &mut usize) {
    if match_token(tokens, pos, Token::ShiftDir) {
        st_expressao(tokens, pos);
        st_cin_cadeia_resto(tokens, pos);
    }
}

// <cout-cadeia> ::= "<<" <cout-item> <cout-cadeia'>
fn st_cout_cadeia(tokens: &[TokenInfo], pos: &mut usize) {
    match_token(tokens, pos, Token::ShiftEsq);
    st_cout_item(tokens, pos);
    st_cout_cadeia_resto(tokens, pos);
}

// <cout-cadeia'> ::= "<<" <cout-item> <cout-cadeia'> | ε
fn st_cout_cadeia_resto(tokens: &[TokenInfo], pos: &mut usize) {
    if match_token(tokens, pos, Token::ShiftEsq) {
        st_cout_item(tokens, pos);
        st_cout_cadeia_resto(tokens, pos);
    }
}

// <cout-item> ::= "endl" | <expressao>
fn st_cout_item(tokens: &[TokenInfo], pos: &mut usize) {
    // "endl" é um identificador especial
    if *pos < tokens.len() && matches!(tokens[*pos].kind, Token::Identificador) && tokens[*pos].lexema == "endl" {
        *pos += 1;
    } else {
        st_expressao(tokens, pos);
    }
}

// ── EXPRESSÕES ──
// <expressao> ::= <atribuicao>
fn st_expressao(tokens: &[TokenInfo], pos: &mut usize) {
    st_atribuicao(tokens, pos);
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

// <atribuicao> ::= <expr-logica> <atribuicao'>
fn st_atribuicao(tokens: &[TokenInfo], pos: &mut usize) {
    st_expr_logica(tokens, pos);
    st_atribuicao_resto(tokens, pos);
}

// <atribuicao'> ::= <op-atrib> <atribuicao> | ε
fn st_atribuicao_resto(tokens: &[TokenInfo], pos: &mut usize) {
    let follow = &[Token::PtVirgula, Token::FechaParen, Token::Virgula, Token::FechaColch];
    
    if *pos < tokens.len() && eh_op_atrib(tokens[*pos].kind) {
        match_token(tokens, pos, tokens[*pos].kind);
        st_atribuicao(tokens, pos);
    }
    else if *pos >= tokens.len() || follow.contains(&tokens[*pos].kind) {
    }
    else {
        panic_mode_recovery(tokens, pos, "operador de atribuição", follow);
    }
}

fn eh_op_atrib(kind: Token) -> bool {
    matches!(
        kind,
        Token::Atribuicao | Token::MaisIgual | Token::MenosIgual | Token::VezesIgual
            | Token::DivIgual | Token::ModIgual
    )
}

// <expr-logica> ::= <expr-relacional> <expr-logica'>
fn st_expr_logica(tokens: &[TokenInfo], pos: &mut usize) {
    st_expr_relacional(tokens, pos);
    st_expr_logica_resto(tokens, pos);
}

// <expr-logica'> ::= "&&" <expr-relacional> <expr-logica'>
//                  | "||" <expr-relacional> <expr-logica'> | ε
fn st_expr_logica_resto(tokens: &[TokenInfo], pos: &mut usize) {
    let follow = &[Token::PtVirgula, Token::FechaParen, Token::Virgula, Token::FechaColch, Token::Atribuicao];
    
    if *pos < tokens.len() && matches!(tokens[*pos].kind, Token::And | Token::Or) {
        if !match_token(tokens, pos, Token::And) {
            match_token(tokens, pos, Token::Or);
        }
        st_expr_relacional(tokens, pos);
        st_expr_logica_resto(tokens, pos);
    }
    else if *pos >= tokens.len() || follow.contains(&tokens[*pos].kind) {
    }
    else {
        panic_mode_recovery(tokens, pos, "'&&' ou '||'", follow);
    }
}

// <expr-relacional> ::= <expr-aritmetica> <expr-relacional'>
fn st_expr_relacional(tokens: &[TokenInfo], pos: &mut usize) {
    st_expr_aritmetica(tokens, pos);
    st_expr_relacional_resto(tokens, pos);
}

// <expr-relacional'> ::= <op-relacional> <expr-aritmetica> <expr-relacional'> | ε
fn st_expr_relacional_resto(tokens: &[TokenInfo], pos: &mut usize) {
    let follow = &[Token::PtVirgula, Token::FechaParen, Token::Virgula, Token::FechaColch, Token::And, Token::Or, Token::Atribuicao];
    
    if *pos < tokens.len() && eh_op_relacional(tokens[*pos].kind) {
        match_token(tokens, pos, tokens[*pos].kind);
        st_expr_aritmetica(tokens, pos);
        st_expr_relacional_resto(tokens, pos);
    }
    else if *pos >= tokens.len() || follow.contains(&tokens[*pos].kind) {
    }
    else {
        panic_mode_recovery(tokens, pos, "operador relacional", follow);
    }
}

fn eh_op_relacional(kind: Token) -> bool {
    matches!(
        kind,
        Token::Igual | Token::Diferente | Token::Menor | Token::Maior | Token::MenorIgual
            | Token::MaiorIgual
    )
}

// <expr-aritmetica> ::= <termo> <expr-aritmetica'>
fn st_expr_aritmetica(tokens: &[TokenInfo], pos: &mut usize) {
    st_termo(tokens, pos);
    st_expr_aritmetica_resto(tokens, pos);
}

// <expr-aritmetica'> ::= "+" <termo> <expr-aritmetica'>
//                      | "-" <termo> <expr-aritmetica'> | ε
fn st_expr_aritmetica_resto(tokens: &[TokenInfo], pos: &mut usize) {
    let follow = &[Token::PtVirgula, Token::FechaParen, Token::Virgula, Token::FechaColch, Token::And, Token::Or, Token::Igual, Token::Diferente, Token::Menor, Token::Maior, Token::Atribuicao];
    
    if *pos < tokens.len() && matches!(tokens[*pos].kind, Token::Adicao | Token::Subtracao) {
        if !match_token(tokens, pos, Token::Adicao) {
            match_token(tokens, pos, Token::Subtracao);
        }
        st_termo(tokens, pos);
        st_expr_aritmetica_resto(tokens, pos);
    }
    else if *pos >= tokens.len() || follow.contains(&tokens[*pos].kind) {
    }
    else {
        panic_mode_recovery(tokens, pos, "'+' ou '-'", follow);
    }
}

// <termo> ::= <factor> <termo'>
fn st_termo(tokens: &[TokenInfo], pos: &mut usize) {
    st_factor(tokens, pos);
    st_termo_resto(tokens, pos);
}

// <termo'> ::= "*" <factor> <termo'>
//            | "/" <factor> <termo'> | "%" <factor> <termo'> | ε
fn st_termo_resto(tokens: &[TokenInfo], pos: &mut usize) {
    let follow = &[Token::PtVirgula, Token::FechaParen, Token::Virgula, Token::FechaColch, Token::Adicao, Token::Subtracao, Token::And, Token::Or, Token::Igual, Token::Diferente, Token::Menor, Token::Maior, Token::Atribuicao];
    
    if *pos < tokens.len() && matches!(tokens[*pos].kind, Token::Multiplicacao | Token::Divisao | Token::Modulo) {
        match_token(tokens, pos, tokens[*pos].kind);
        st_factor(tokens, pos);
        st_termo_resto(tokens, pos);
    }
    else if *pos >= tokens.len() || follow.contains(&tokens[*pos].kind) {
    }
    else {
        panic_mode_recovery(tokens, pos, "'*', '/', ou '%'", follow);
    }
}

// <factor> ::= <unario>
fn st_factor(tokens: &[TokenInfo], pos: &mut usize) {
    st_unario(tokens, pos);
}

// <unario> ::= <postfixo>
//            | "!" <unario> | "-" <unario> | "++" <unario> | "--" <unario>
//            | "&" <unario> | "*" <unario>
fn st_unario(tokens: &[TokenInfo], pos: &mut usize) {
    if *pos < tokens.len() {
        match tokens[*pos].kind {
            Token::Not | Token::Subtracao | Token::Incremento | Token::Decremento
            | Token::BitAnd | Token::Multiplicacao | Token::BitNot => {
                match_token(tokens, pos, tokens[*pos].kind);
                st_unario(tokens, pos);
            }
            _ => st_postfixo(tokens, pos),
        }
    }
}

// <postfixo> ::= <primario> <postfixo'>
fn st_postfixo(tokens: &[TokenInfo], pos: &mut usize) {
    st_primario(tokens, pos);
    st_postfixo_resto(tokens, pos);
}

// <postfixo'> ::= "++" <postfixo'>
//              | "--" <postfixo'> | "[" <expressao> "]" <postfixo'> | "(" <argumentos> ")" <postfixo'>
//              | "." <identificador> <postfixo'> | "->" <identificador> <postfixo'> | ε
fn st_postfixo_resto(tokens: &[TokenInfo], pos: &mut usize) {
    let follow = &[Token::PtVirgula, Token::FechaParen, Token::Virgula, Token::FechaColch, Token::And, Token::Or, Token::Igual, Token::Diferente];
    
    if *pos >= tokens.len() {
        return;
    }
    match tokens[*pos].kind {
        Token::Incremento | Token::Decremento => {
            match_token(tokens, pos, tokens[*pos].kind);
            st_postfixo_resto(tokens, pos);
        }
        Token::AbreColch => {
            match_token(tokens, pos, Token::AbreColch);
            st_expressao(tokens, pos);
            if !expect(tokens, pos, Token::FechaColch, "]", follow) {
                return;
            }
            st_postfixo_resto(tokens, pos);
        }
        Token::AbreParen => {
            match_token(tokens, pos, Token::AbreParen);
            st_argumentos(tokens, pos);
            if !expect(tokens, pos, Token::FechaParen, ")", follow) {
                return;
            }
            st_postfixo_resto(tokens, pos);
        }
        Token::PontoMembro => {
            match_token(tokens, pos, Token::PontoMembro);
            if !expect(tokens, pos, Token::Identificador, "identificador", follow) {
                return;
            }
            st_postfixo_resto(tokens, pos);
        }
        Token::Seta => {
            match_token(tokens, pos, Token::Seta);
            if !expect(tokens, pos, Token::Identificador, "identificador", follow) {
                return;
            }
            st_postfixo_resto(tokens, pos);
        }
        _ => {}
    }
}

// <primario> ::= <literal>
//             | <identificador> | "this" | "(" <expressao> ")"
//             | "new" <tipo> <new-sufixo>
fn st_primario(tokens: &[TokenInfo], pos: &mut usize) {
    if *pos >= tokens.len() {
        return;
    }
    
    match tokens[*pos].kind {
        Token::Inteiro | Token::Float | Token::Char | Token::String | Token::TrueLiteral | Token::FalseLiteral => {
            *pos += 1;
        }
        Token::Identificador => {
            *pos += 1;
        }
        Token::This => {
            *pos += 1;
        }
        Token::AbreParen => {
            *pos += 1;
            st_expressao(tokens, pos);
            let follow = &[Token::AbreColch, Token::AbreParen, Token::PontoMembro, Token::Seta, Token::Incremento, Token::Decremento, Token::PtVirgula, Token::Virgula, Token::FechaParen];
            if !expect(tokens, pos, Token::FechaParen, ")", follow) { return; }
        }
        Token::New => {
            *pos += 1;
            st_tipo(tokens, pos);
            st_new_sufixo(tokens, pos);
        }
        _ => {
            let follow = &[Token::AbreColch, Token::AbreParen, Token::PontoMembro, Token::Seta, Token::Incremento, Token::Decremento, Token::PtVirgula, Token::Virgula, Token::FechaParen, Token::And, Token::Or, Token::Igual, Token::Diferente, Token::Menor, Token::Maior, Token::Adicao, Token::Subtracao, Token::Multiplicacao, Token::Divisao, Token::Modulo];
            panic_mode_recovery(tokens, pos, "Primário", follow);
        }
    }
}

// <new-sufixo> ::= "(" <argumentos> ")" | "[" <expressao> "]"
fn st_new_sufixo(tokens: &[TokenInfo], pos: &mut usize) {
    let follow = &[Token::AbreColch, Token::AbreParen, Token::PontoMembro, Token::Seta, Token::Incremento, Token::Decremento, Token::PtVirgula, Token::Virgula];
    
    if *pos < tokens.len() && matches!(tokens[*pos].kind, Token::AbreParen) {
        match_token(tokens, pos, Token::AbreParen);
        st_argumentos(tokens, pos);
        if !expect(tokens, pos, Token::FechaParen, ")", follow) { return; }
    } 
    else if *pos < tokens.len() && matches!(tokens[*pos].kind, Token::AbreColch) {
        match_token(tokens, pos, Token::AbreColch);
        st_expressao(tokens, pos);
        if !expect(tokens, pos, Token::FechaColch, "]", follow) { return; }
    }
    else if *pos >= tokens.len() || follow.contains(&tokens[*pos].kind) {
    }
    else {
        panic_mode_recovery(tokens, pos, "'(' ou '['", follow);
    }
}

// <argumentos> ::= <lista-args> | ε
fn st_argumentos(tokens: &[TokenInfo], pos: &mut usize) {
    let follow = &[Token::FechaParen];
    
    if *pos < tokens.len() && pode_iniciar_expressao(tokens, *pos) {
        st_lista_args(tokens, pos);
    }
    else if *pos >= tokens.len() || follow.contains(&tokens[*pos].kind) {
    }
    else {
        panic_mode_recovery(tokens, pos, "expressão ou ')'", follow);
    }
}

// <lista-args> ::= <expressao> <lista-args'>
fn st_lista_args(tokens: &[TokenInfo], pos: &mut usize) {
    st_expressao(tokens, pos);
    st_lista_args_resto(tokens, pos);
}

// <lista-args'> ::= "," <expressao> <lista-args'> | ε
fn st_lista_args_resto(tokens: &[TokenInfo], pos: &mut usize) {
    let follow = &[Token::FechaParen];
    
    if *pos < tokens.len() && matches!(tokens[*pos].kind, Token::Virgula) {
        match_token(tokens, pos, Token::Virgula);
        st_expressao(tokens, pos);
        st_lista_args_resto(tokens, pos);
    }
    else if *pos >= tokens.len() || follow.contains(&tokens[*pos].kind) {
    }
    else {
        panic_mode_recovery(tokens, pos, "',' ou ')'", follow);
    }
}

// <literal> ::= <inteiro> | <real> | <caracter> | <cadeia> | "true" | "false" | "nullptr"
fn st_literal(tokens: &[TokenInfo], pos: &mut usize) {
    if *pos >= tokens.len() {
        return;
    }
    
    match tokens[*pos].kind {
        Token::Inteiro | Token::Float | Token::Char | Token::String | Token::TrueLiteral
        | Token::FalseLiteral => {
            *pos += 1;
        }
        _ => {
            let follow = &[Token::AbreColch, Token::AbreParen, Token::PontoMembro, Token::Seta, Token::Incremento, Token::Decremento, Token::PtVirgula, Token::Virgula, Token::FechaParen, Token::And, Token::Or, Token::Igual, Token::Diferente, Token::Menor, Token::Maior, Token::Adicao, Token::Subtracao, Token::Multiplicacao, Token::Divisao, Token::Modulo];
            panic_mode_recovery(tokens, pos, "Literal esperado", follow);
        }
    }
}

pub fn analisar(tokens: Vec<TokenInfo>) {
    println!("Iniciando analise sintatica...");
    let mut pos = 0;
    
    st_programa(&tokens, &mut pos);
    
    if pos < tokens.len() {
        println!("Aviso: O analisador parou prematuramente na posicao {}. Tokens restantes ignorados.", pos);
    } else {
        println!("Analise sintatica bem-sucedida!");
    }
}