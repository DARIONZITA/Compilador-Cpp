# Manual do Programador (MP) — Parser LL(1)

## 1. Visão geral técnica

Este documento descreve a arquitetura interna do **analisador sintático (parser)** implementado em Rust. O parser:

- **Tipo**: Recursive Descent LL(1) — lê de esquerda para direita, usa 1 token de lookahead
- **Saída**: Validação sintática (void-returning) — não constrói AST
- **Recuperação de erro**: Panic mode com FOLLOW sets
- **Ficheiro principal**: `src/analise_sintatica.rs`
- **Gramática**: BNF em `gramatica.bnf`

---

## 2. Arquitetura de módulos

```
main.rs
  ├─ calls ──→ lexer.rs (tokenização)
  ├─ calls ──→ analise_sintatica.rs (validação de sintaxe)
  │             └─ imports ──→ token.rs (Token, TokenInfo)
```

### 2.1 Fluxo de dados

1. **lexer.rs** → tokeniza código-fonte → `Vec<TokenInfo>`
2. **analise_sintatica.rs** → parse tokens → valida estrutura → emite erros ou sucesso
3. **main.rs** → orquestra o fluxo

---

## 3. Estrutura do parser (analise_sintatica.rs)

### 3.1 Funções auxiliares globais

```rust
fn match_token(tokens: &[TokenInfo], pos: &mut usize, expected_kind: Token) -> bool
```
- **Propósito**: Verificar se token atual é do tipo esperado
- **Efeito**: Avança `pos` se for correspondência
- **Retorno**: `true` se correspondeu, `false` caso contrário
- **Não emite erro**: simplesmente retorna

```rust
fn expect(tokens: &[TokenInfo], pos: &mut usize, expected_kind: Token, 
          expect_name: &str, follow: &[Token]) -> bool
```
- **Propósito**: Exigir token específico; se falhar, emitir erro e recuperar
- **Efeito**: 
  1. Se token atual corresponde: avança e retorna `true`
  2. Se não corresponde: emite erro + chama `panic_mode_recovery()`
- **Retorno**: `true` se encontrado, `false` após recuperação

```rust
fn panic_mode_recovery(tokens: &[TokenInfo], pos: &mut usize, 
                      what_expected: &str, follow: &[Token])
```
- **Propósito**: Entrar em panic mode (erro sem esperança)
- **Comportamento**:
  1. Emite mensagem de erro com linha + token atual
  2. Avança até encontrar token em `follow`
  3. Permite continuar parsing do próximo elemento válido

**Exemplo de uso**:
```rust
if !expect(tokens, pos, Token::PtVirgula, ";", follow) {
    // expect já emitiu erro e recuperou
}
```

### 3.2 Padrão LL(1) recursivo descendente

Cada função de parsing segue este padrão:

```rust
fn st_construcao(tokens: &[TokenInfo], pos: &mut usize) {
    let follow = &[Token::A, Token::B, Token::C];  // FOLLOW set
    
    // Tentar primeiro caminho
    if *pos < tokens.len() && matches!(tokens[*pos].kind, Token::X | Token::Y) {
        // Parse primeira alternativa
        // ... match_token / expect / recursão ...
        return;
    }
    
    // Tentar segundo caminho
    if *pos < tokens.len() && matches!(tokens[*pos].kind, Token::Z) {
        // Parse segunda alternativa
        return;
    }
    
    // Épsilon (epsilon é válido aqui?)
    if *pos >= tokens.len() || follow.contains(&tokens[*pos].kind) {
        return;  // épsilon aceito, sem erro
    }
    
    // Nenhum caminho possível → erro
    panic_mode_recovery(tokens, pos, "descrição do esperado", follow);
}
```

**Princípio LL(1)**:
- Decide qual produção usar **apenas pelo token atual**
- No máximo 1 token de lookahead
- Se nenhuma produção corresponde e épsilon é válido (token em FOLLOW), épsilon é silenciosamente aceito
- Se nenhuma produção corresponde e épsilon não é válido, emite erro

---

## 4. Catálogo de funções de parsing

### 4.1 Nível alto — Programa

```rust
fn st_programa(tokens: &[TokenInfo], pos: &mut usize)
```
**Gramática**: `<programa> ::= <include-seq> <declaracao-seq>`
**Comportamento**: Parse sequência de includes + sequência de declarações
**Efeito colateral**: Emite erros se sintaxe inválida

```rust
fn st_include_seq(tokens: &[TokenInfo], pos: &mut usize)
```
**Gramática**: `<include-seq> ::= <include> <include-seq> | ε`
**Comportamento**: Recursão à direita para processar múltiplos includes
**FOLLOW**: Começam tokens de declaração (class, int, float, etc.)

```rust
fn st_include(tokens: &[TokenInfo], pos: &mut usize)
```
**Gramática**: `<include> ::= "#include" <include-alvo>`
**Comportamento**: Valida presença de `#include` + alvo

### 4.2 Declarações

```rust
fn st_declaracao_seq(tokens: &[TokenInfo], pos: &mut usize)
```
**Gramática**: `<declaracao-seq> ::= <declaracao> <declaracao-seq> | ε`
**Comportamento**: Recursão à direita, processa até EOF

```rust
fn st_declaracao(tokens: &[TokenInfo], pos: &mut usize)
```
**Gramática**: `<declaracao> ::= <declaracao-classe> | <modificador> <tipo> <identificador> <sufixo-decl>`
**Decisor**: Se token é `class` ou `struct` → declaração de classe; senão → variável/função

```rust
fn st_declaracao_classe(tokens: &[TokenInfo], pos: &mut usize)
```
**Gramática**: `<declaracao-classe> ::= "class" <id> <heranca> "{" <membros-classe> "}"`
**Valida**: Nome classe, herança (opcional), membros, fechamento com `}`

### 4.3 Modificadores e Tipos

```rust
fn st_modificador(tokens: &[TokenInfo], pos: &mut usize)
```
**Gramática**: `<modificador> ::= public | private | protected | static | const | ... | ε`
**Nota**: Recursivo — permite múltiplos modificadores

```rust
fn st_tipo(tokens: &[TokenInfo], pos: &mut usize)
```
**Gramática**: `<tipo> ::= int | float | double | char | bool | void | string | <identificador>`
**Comportamento**: Consome um token de tipo, emite erro se não for tipo válido

### 4.4 Blocos e Comandos

```rust
fn st_bloco(tokens: &[TokenInfo], pos: &mut usize)
```
**Gramática**: `<bloco> ::= "{" <comando-seq> "}"`
**Comportamento**: Exige `{`, parse comandos, exige `}`

```rust
fn st_comando_seq(tokens: &[TokenInfo], pos: &mut usize)
```
**Gramática**: `<comando-seq> ::= <comando> <comando-seq> | ε`
**FOLLOW**: `FechaChave`

```rust
fn st_comando(tokens: &[TokenInfo], pos: &mut usize)
```
**Dispatcher**: Verifica tipo de comando via token atual
**Casos**: if, switch, while, for, do, cin, cout, return, break, continue, bloco, tipo (decl local), expressão

### 4.5 Comandos de Seleção

```rust
fn st_comando_seleccao(tokens: &[TokenInfo], pos: &mut usize)
```
**Gramática**: 
```
<comando-seleccao> ::= "if" "(" <expr> ")" <bloco> <else-parte>
                    | "switch" "(" <expr> ")" "{" <casos> "}"
```

```rust
fn st_else_parte(tokens: &[TokenInfo], pos: &mut usize)
```
**Gramática**: `<else-parte> ::= "else" <else-corpo> | ε`

### 4.6 Expressões

```rust
fn st_expressao(tokens: &[TokenInfo], pos: &mut usize)
```
**Gramática**: `<expressao> ::= <atribuicao>`

```rust
fn st_atribuicao(tokens: &[TokenInfo], pos: &mut usize)
```
**Gramática**: `<atribuicao> ::= <expr-logica> <atribuicao'>`
**Right-associative**: `a = b = c` é `a = (b = c)`

```rust
fn st_expr_logica(tokens: &[TokenInfo], pos: &mut usize)
```
**Operadores**: `&&`, `||`

```rust
fn st_expr_relacional(tokens: &[TokenInfo], pos: &mut usize)
```
**Operadores**: `==`, `!=`, `<`, `>`, `<=`, `>=`

```rust
fn st_expr_aritmetica(tokens: &[TokenInfo], pos: &mut usize)
```
**Operadores**: `+`, `-`
**Precedência**: Menor que multiplicação

```rust
fn st_termo(tokens: &[TokenInfo], pos: &mut usize)
```
**Operadores**: `*`, `/`, `%`
**Precedência**: Maior que adição

```rust
fn st_unario(tokens: &[TokenInfo], pos: &mut usize)
```
**Operadores prefixo**: `!`, `-`, `++`, `--`, `&`, `*`

```rust
fn st_postfixo(tokens: &[TokenInfo], pos: &mut usize)
```
**Operadores**: `[]`, `()`, `.`, `->`, `++`, `--`

```rust
fn st_primario(tokens: &[TokenInfo], pos: &mut usize)
```
**Base**: Literais, identificadores, `this`, `(expr)`, `new tipo`

---

## 5. Padrões comuns

### 5.1 Decisão ternária

```rust
let follow = &[Token::FechaChave, Token::Else];

if *pos < tokens.len() && matches!(tokens[*pos].kind, Token::If) {
    // Alternativa 1
} else if *pos < tokens.len() && matches!(tokens[*pos].kind, Token::For) {
    // Alternativa 2
} else if *pos >= tokens.len() || follow.contains(&tokens[*pos].kind) {
    // Épsilon
} else {
    panic_mode_recovery(tokens, pos, "if ou for", follow);
}
```

### 5.2 Recursão à direita

```rust
fn st_lista(tokens: &[TokenInfo], pos: &mut usize) {
    let follow = &[Token::FechaChave];
    
    if pode_iniciar_item(tokens, *pos) {
        st_item(tokens, pos);
        st_lista(tokens, pos);  // ← recursão
    }
    else if *pos >= tokens.len() || follow.contains(&tokens[*pos].kind) {
        // épsilon
    }
    else {
        panic_mode_recovery(tokens, pos, "item ou '}'", follow);
    }
}
```

### 5.3 Match vs Expect

```rust
// Match (sem erro):
if match_token(tokens, pos, Token::If) {
    // encontrou if
}

// Expect (com erro e recuperação):
if !expect(tokens, pos, Token::PtVirgula, ";", follow) {
    // não encontrou, mas já recuperou
}
```

---

## 6. FIRST e FOLLOW Sets

### 6.1 Conceito

- **FIRST(X)**: Tokens que podem iniciar X
- **FOLLOW(X)**: Tokens que podem seguir X

### 6.2 Exemplo

```
<else-parte> ::= "else" <bloco> | ε
FIRST(<else-parte>) = { "else" }
FOLLOW(<else-parte>) = { "}", ";", "else", ... }
```

Se token não está em FIRST e está em FOLLOW → épsilon é aceito.

---

## 7. Recuperação de erro

### 7.1 Algoritmo

```
expect(Token X) falha:
    1. Emitir: "Erro [linha N]: Esperado X, mas encontrado Y"
    2. Avançar até tokens[pos] ∈ FOLLOW
    3. Retornar false (análise continua)
```

### 7.2 Exemplo

```rust
if !expect(tokens, pos, Token::AbreChave, "{", follow) {
    // erro já emitido e recuperado
    return;
}
```

---

## 8. Funções auxiliares

### 8.1 Predicados

```rust
fn pode_iniciar_declaracao(tokens: &[TokenInfo], pos: usize) -> bool {
    matches!(tokens[pos].kind, Token::Class | Token::Int | ...)
}

fn eh_op_atrib(kind: Token) -> bool {
    matches!(kind, Token::Atribuicao | Token::MaisIgual | ...)
}
```

---

## 9. Debugging

### 9.1 Saída de erro

```
Erro [linha 5]: Esperado }, mas encontrado EOF
Erro [linha 10]: Esperado ;, mas encontrado Identificador (`x`)
```

**Formato**: `Erro [linha N]: Esperado X, mas encontrado Y`

### 9.2 Testes

Edite `src/files/main.cpp` e rode:
```bash
cargo run 2>&1
```

### 9.3 Estrutura simples

```cpp
#include <stdio.h>

int main() {
    int x = 5;
    return 0;
}
```

---

## 10. Extensões futuras

### 10.1 Novo token

1. Adicionar em `token.rs`
2. Adicionar DFA estado em `lexer.rs`
3. Adicionar produção em `gramatica.bnf`
4. Implementar `st_xxx()`
5. Adicionar FOLLOW set

### 10.2 Novo comando

```rust
// Em st_comando():
Token::Pragma => {
    *pos += 1;
    st_expressao(tokens, pos);
    if !expect(tokens, pos, Token::PtVirgula, ";", follow) { return; }
}
```

### 10.3 Precedência de operadores

- Aumentar: mover para nível mais profundo
- Diminuir: mover para nível mais alto

---

## 11. Notas de implementação

### 11.1 Comparação de tokens

```rust
// Usar discriminant:
std::mem::discriminant(&tokens[*pos].kind) == std::mem::discriminant(&expected_kind)

// Ou se tem PartialEq:
tokens[*pos].kind == expected_kind
```

### 11.2 Tratamento de EOF

```rust
if *pos >= tokens.len() {
    // estamos no EOF
}
```

### 11.3 Mensagens de erro

```rust
eprintln!(
    "Erro [linha {}]: {}",
    tokens[*pos].linha,
    "descrição do erro"
);
```

---

## 12. Fluxo completo

```
main.rs
  └─ lexer::tokenizar(código)
     └─ Vec<TokenInfo>

analise_sintatica.rs::analisar()
  └─ st_programa(&tokens, &mut pos)
     ├─ st_include_seq() → múltiplas includes
     └─ st_declaracao_seq() → múltiplas declarações

Resultado:
  ✅ Se pos == len(tokens): sucesso
  ⚠️ Se pos < len(tokens): aviso de tokens não consumidos
  ❌ Se emitiu "Erro [linha X]": falha
```

---

## 13. Referência rápida

| Função | Propósito | Retorna |
|--------|----------|---------|
| `match_token` | Verificar correspondência | bool |
| `expect` | Exigir + recuperar | bool |
| `panic_mode_recovery` | Pular até FOLLOW | void |
| `st_*` | Parse produção | void |
| `pode_*` | Predicado | bool |
| `eh_*` | Verificação tipo | bool |

---

## 14. Recursos

- **Gramática formal**: `gramatica.bnf`
- **Manual do utilizador**: `MU_PARSER.md`
- **Tokens**: `src/token.rs`
- **Lexer**: `src/lexer.rs`

---

*Fim do Manual do Programador — Parser LL(1)*
