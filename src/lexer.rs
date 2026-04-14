use crate::token::{Estado, Token, TokenInfo};
use crate::utils::{eh_digito, eh_espaco, eh_letra, push_token};

fn ler_caractere(chars: &[char], index: usize) -> char {
    if index < chars.len() {
        chars[index]
    } else {
        '\0'
    }
}

fn voltar_caractere(index: usize) -> usize {
    if index > 0 {
        index - 1
    } else {
        0
    }
    
}

fn identificadores_reservados(lexema: &str) -> Token {
    match lexema {
        "if" => Token::If,
        "else" => Token::Else,
        "while" => Token::While,
        "for" => Token::For,
        "return" => Token::Return,
        "int" => Token::Int,
        "float" => Token::FloatType,
        "char" => Token::CharType,
        "void" => Token::Void,
        "class" => Token::Class,
        "struct" => Token::Struct,
        "enum" => Token::Enum,
        "const" => Token::Const,
        "static" => Token::Static,
        "public" => Token::Public,
        "private" => Token::Private,
        "protected" => Token::Protected,
        "virtual" => Token::Virtual,
        "override" => Token::Override,
        "abstract" => Token::Abstract,
        "template" => Token::Template,
        "typedef" => Token::Typedef,
        "namespace" => Token::Namespace,
        "using" => Token::Using,
        "include" => Token::Include,
        "long" => Token::Long,
        "short" => Token::Short,
        "signed" => Token::Signed,
        "unsigned" => Token::Unsigned,
        "do" => Token::Do,
        "switch" => Token::Switch,
        "case" => Token::Case,
        "break" => Token::Break,
        "continue" => Token::Continue,
        "goto" => Token::Goto,
        "default" => Token::Default,
        "bool" => Token::Bool,
        "double" => Token::Double,
        "new" => Token::New,
        "delete" => Token::Delete,
        "sizeof" => Token::Sizeof,
        "and" => Token::And,
        "or" => Token::Or,
        "not" => Token::Not,
        "and_eq" => Token::AndEq,
        "or_eq" => Token::OrEq,
        "xor_eq" => Token::XorEq,
        "xor" => Token::Xor,
        "bitand" => Token::Bitand,
        "bitor" => Token::Bitor,
        "compl" => Token::Compl,
        "not_eq" => Token::NotEq,
        "this" => Token::This,
        "inline" => Token::Inline,
        "explicit" => Token::Explicit,
        "friend" => Token::Friend,
        "operator" => Token::Operator,
        "typename" => Token::Typename,
        "try" => Token::Try,
        "catch" => Token::Catch,
        "throw" => Token::Throw,
        "static_cast" => Token::StaticCast,
        "dynamic_cast" => Token::DynamicCast,
        "const_cast" => Token::ConstCast,
        "reinterpret_cast" => Token::ReinterpretCast,
        "auto" => Token::Auto,
        "register" => Token::Register,
        "extern" => Token::Extern,
        "volatile" => Token::Volatile,
        "mutable" => Token::Mutable,
        "export" => Token::Export,
        "true" => Token::TrueLiteral,
        "false" => Token::FalseLiteral,
        "typeid" => Token::Typeid,
        _ => Token::Identificador,
    }
}

fn analex(atual: char, tokens: &mut Vec<TokenInfo>, estado: &mut Estado, lexema: &mut String, linha: usize) {
    match *estado {
        Estado::Inicio => {
            if atual == '\0' {
                return;
            }

            if eh_letra(atual) || atual == '_' {
                lexema.push(atual);
                *estado = Estado::Identificador;
            } else if eh_digito(atual) {
                lexema.push(atual);
                *estado = Estado::Inteiro;
            } else if atual == '"' {
                lexema.push(atual);
                *estado = Estado::StringAberta;
            } else if atual == '\'' {
                lexema.push(atual);
                *estado = Estado::CharAberto;
            } else if eh_espaco(atual) {
                *estado = Estado::Inicio;
            } else {
                match atual {
                    '+' => {
                        lexema.push(atual);
                        *estado = Estado::OpMais;
                    }
                    '-' => {
                        lexema.push(atual);
                        *estado = Estado::OpMenos;
                    }
                    '*' => {
                        lexema.push(atual);
                        *estado = Estado::OpVezes;
                    }
                    '/' => {
                        lexema.push(atual);
                        *estado = Estado::OpDiv;
                    }
                    '%' => {
                        lexema.push(atual);
                        *estado = Estado::OpMod;
                    }
                    '=' => {
                        lexema.push(atual);
                        *estado = Estado::OpIgual;
                    }
                    '<' => {
                        lexema.push(atual);
                        *estado = Estado::OpMenor;
                    }
                    '>' => {
                        lexema.push(atual);
                        *estado = Estado::OpMaior;
                    }
                    '!' => {
                        lexema.push(atual);
                        *estado = Estado::OpExclamacao;
                    }
                    '&' => {
                        lexema.push(atual);
                        *estado = Estado::OpE;
                    }
                    '|' => {
                        lexema.push(atual);
                        *estado = Estado::OpOu;
                    }
                    ':' => {
                        lexema.push(atual);
                        *estado = Estado::OpDoisPontos;
                    }
                    '^' => {
                        lexema.push(atual);
                        push_token(tokens, Token::BitXor, lexema, linha);
                    }
                    '~' => {
                        lexema.push(atual);
                        push_token(tokens, Token::BitNot, lexema, linha);
                    }
                    '?' => {
                        lexema.push(atual);
                        push_token(tokens, Token::Interrogacao, lexema, linha);
                    }
                    ';' => {
                        lexema.push(atual);
                        push_token(tokens, Token::PtVirgula, lexema, linha);
                    }
                    ',' => {
                        lexema.push(atual);
                        push_token(tokens, Token::Virgula, lexema, linha);
                    }
                    '(' => {
                        lexema.push(atual);
                        push_token(tokens, Token::AbreParen, lexema, linha);
                    }
                    ')' => {
                        lexema.push(atual);
                        push_token(tokens, Token::FechaParen, lexema, linha);
                    }
                    '{' => {
                        lexema.push(atual);
                        push_token(tokens, Token::AbreChave, lexema, linha);
                    }
                    '}' => {
                        lexema.push(atual);
                        push_token(tokens, Token::FechaChave, lexema, linha);
                    }
                    '[' => {
                        lexema.push(atual);
                        push_token(tokens, Token::AbreColch, lexema, linha);
                    }
                    ']' => {
                        lexema.push(atual);
                        push_token(tokens, Token::FechaColch, lexema, linha);
                    }
                    '.' => {
                        lexema.push(atual);
                        push_token(tokens, Token::PontoMembro, lexema, linha);
                    }
                    _ => {}
                }
            }
        }
        Estado::Identificador => {
            if eh_letra(atual) || eh_digito(atual) || atual == '_' {
                lexema.push(atual);
            } else {
                let token = identificadores_reservados(lexema);
                push_token(tokens, token, lexema, linha);
                *estado = Estado::Outro;
            }
        }
        Estado::Inteiro => {
            if eh_digito(atual) {
                lexema.push(atual);
            } else if atual == '.' {
                lexema.push(atual);
                *estado = Estado::FloatPonto;
            } else {
                push_token(tokens, Token::Inteiro, lexema, linha);
                *estado = Estado::Outro;
            }
        }
        Estado::FloatPonto => {
            if eh_digito(atual) {
                lexema.push(atual);
                *estado = Estado::FloatDigitos;
            } else {
                push_token(tokens, Token::Float, lexema, linha);
                *estado = Estado::Outro;
            }
        }
        Estado::FloatDigitos => {
            if eh_digito(atual) {
                lexema.push(atual);
            } else {
                push_token(tokens, Token::Float, lexema, linha);
                *estado = Estado::Outro;
            }
        }
        Estado::StringAberta => {
            if atual == '\0' {
                push_token(tokens, Token::String, lexema, linha);
                *estado = Estado::Inicio;
            } else if atual == '\\' {
                lexema.push(atual);
                *estado = Estado::StringEscape;
            } else {
                lexema.push(atual);
                if atual == '"' {
                    push_token(tokens, Token::String, lexema, linha);
                    *estado = Estado::Inicio;
                }
            }
        }
        Estado::StringEscape => {
            if atual != '\0' {
                lexema.push(atual);
            }
            *estado = Estado::StringAberta;
        }
        Estado::CharAberto => {
            if atual == '\\' {
                lexema.push(atual);
                *estado = Estado::CharEscape;
            } else if atual != '\0' {
                lexema.push(atual);
                *estado = Estado::CharConteudo;
            } else {
                push_token(tokens, Token::Char, lexema, linha);
                *estado = Estado::Inicio;
            }
        }
        Estado::CharEscape => {
            if atual != '\0' {
                lexema.push(atual);
                *estado = Estado::CharConteudo;
            } else {
                push_token(tokens, Token::Char, lexema, linha);
                *estado = Estado::Inicio;
            }
        }
        Estado::CharConteudo => {
            if atual == '\'' {
                lexema.push(atual);
                push_token(tokens, Token::Char, lexema, linha);
            } else {
                push_token(tokens, Token::Char, lexema, linha);
                *estado = Estado::Outro;
                return;
            }
            *estado = Estado::Inicio;
        }
        Estado::OpMais => {
            if atual == '+' {
                lexema.push(atual);
                push_token(tokens, Token::Incremento, lexema, linha);
                *estado = Estado::Inicio;
            } else if atual == '=' {
                lexema.push(atual);
                push_token(tokens, Token::MaisIgual, lexema, linha);
                *estado = Estado::Inicio;
            } else {
                push_token(tokens, Token::Adicao, lexema, linha);
                *estado = Estado::Outro;
            }
        }
        Estado::OpMenos => {
            if atual == '-' {
                lexema.push(atual);
                push_token(tokens, Token::Decremento, lexema, linha);
                *estado = Estado::Inicio;
            } else if atual == '=' {
                lexema.push(atual);
                push_token(tokens, Token::MenosIgual, lexema, linha);
                *estado = Estado::Inicio;
            } else if atual == '>' {
                lexema.push(atual);
                push_token(tokens, Token::Seta, lexema, linha);
                *estado = Estado::Inicio;
            } else {
                push_token(tokens, Token::Subtracao, lexema, linha);
                *estado = Estado::Outro;
            }
        }
        Estado::OpVezes => {
            if atual == '=' {
                lexema.push(atual);
                push_token(tokens, Token::VezesIgual, lexema, linha);
                *estado = Estado::Inicio;
            } else {
                push_token(tokens, Token::Multiplicacao, lexema, linha);
                *estado = Estado::Outro;
            }
        }
        Estado::OpDiv => {
            if atual == '/' {
                lexema.push(atual);
                *estado = Estado::ComentarioLinha;
            } else if atual == '=' {
                lexema.push(atual);
                push_token(tokens, Token::DivIgual, lexema, linha);
                *estado = Estado::Inicio;
            } else if atual == '*' {
                lexema.push(atual);
                *estado = Estado::ComentarioBloco;
            } else {
                push_token(tokens, Token::Divisao, lexema, linha);
                *estado = Estado::Outro;
            }
        }
        Estado::OpMod => {
            if atual == '=' {
                lexema.push(atual);
                push_token(tokens, Token::ModIgual, lexema, linha);
                *estado = Estado::Inicio;
            } else {
                push_token(tokens, Token::Modulo, lexema, linha);
                *estado = Estado::Outro;
            }
        }
        Estado::OpIgual => {
            if atual == '=' {
                lexema.push(atual);
                push_token(tokens, Token::Igual, lexema, linha);
                *estado = Estado::Inicio;
            } else {
                push_token(tokens, Token::Atribuicao, lexema, linha);
                *estado = Estado::Outro;
            }
        }
        Estado::OpMenor => {
            if atual == '=' {
                lexema.push(atual);
                push_token(tokens, Token::MenorIgual, lexema, linha);
                *estado = Estado::Inicio;
            } else if atual == '<' {
                lexema.push(atual);
                push_token(tokens, Token::ShiftEsq, lexema, linha);
                *estado = Estado::Inicio;
            } else {
                push_token(tokens, Token::Menor, lexema, linha);
                *estado = Estado::Outro;
            }
        }
        Estado::OpMaior => {
            if atual == '=' {
                lexema.push(atual);
                push_token(tokens, Token::MaiorIgual, lexema, linha);
                *estado = Estado::Inicio;
            } else if atual == '>' {
                lexema.push(atual);
                push_token(tokens, Token::ShiftDir, lexema, linha);
                *estado = Estado::Inicio;
            } else {
                push_token(tokens, Token::Maior, lexema, linha);
                *estado = Estado::Outro;
            }
        }
        Estado::OpExclamacao => {
            if atual == '=' {
                lexema.push(atual);
                push_token(tokens, Token::Diferente, lexema, linha);
                *estado = Estado::Inicio;
            } else {
                push_token(tokens, Token::Not, lexema, linha);
                *estado = Estado::Outro;
            }
        }
        Estado::OpE => {
            if atual == '&' {
                lexema.push(atual);
                push_token(tokens, Token::And, lexema, linha);
                *estado = Estado::Inicio;
            } else {
                push_token(tokens, Token::BitAnd, lexema, linha);
                *estado = Estado::Outro;
            }
        }
        Estado::OpOu => {
            if atual == '|' {
                lexema.push(atual);
                push_token(tokens, Token::Or, lexema, linha);
                *estado = Estado::Inicio;
            } else {
                push_token(tokens, Token::BitOr, lexema, linha);
                *estado = Estado::Outro;
            }
        }
        Estado::OpDoisPontos => {
            if atual == ':' {
                lexema.push(atual);
                push_token(tokens, Token::Escopo, lexema, linha);
                *estado = Estado::Inicio;
            } else {
                push_token(tokens, Token::DoisPontos, lexema, linha);
                *estado = Estado::Outro;
            }
        }
        Estado::ComentarioLinha => {
            if atual == '\n' || atual == '\0' {
                push_token(tokens, Token::ComentarioLinha, lexema, linha);
                *estado = Estado::Outro;
            } else {
                lexema.push(atual);
            }
        }
        Estado::ComentarioBloco => {
            if atual == '\0' {
                push_token(tokens, Token::ComentarioBloco, lexema, linha);
                *estado = Estado::Inicio;
            } else if atual == '*' {
                lexema.push(atual);
                *estado = Estado::ComentarioBlocoAst;
            } else {
                lexema.push(atual);
            }
        }
        Estado::ComentarioBlocoAst => {
            if atual == '/' {
                lexema.push(atual);
                push_token(tokens, Token::ComentarioBloco, lexema, linha);
                *estado = Estado::Inicio;
            } else if atual == '*' {
                lexema.push(atual);
            } else if atual == '\0' {
                push_token(tokens, Token::ComentarioBloco, lexema, linha);
                *estado = Estado::Inicio;
            } else {
                lexema.push(atual);
                *estado = Estado::ComentarioBloco;
            }
        }
        Estado::Outro => {}
    }
}

pub fn tokenizar(conteudo: &str) -> Vec<TokenInfo> {
    let mut tokens: Vec<TokenInfo> = Vec::new();
    let chars: Vec<char> = conteudo.chars().collect();

    let mut estado = Estado::Inicio;
    let mut lexema = String::new();
    let mut i = 0usize;
    let mut linha = 1usize;

    while i <= chars.len() {
        let atual = ler_caractere(&chars, i);

        analex(atual, &mut tokens, &mut estado, &mut lexema, linha);
        if atual == '\n' {
            linha += 1;
        }
        if matches!(estado, Estado::Outro) {
            i = voltar_caractere(i);
            estado = Estado::Inicio;
        }

        i += 1;
    }

    tokens
}
