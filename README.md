# Compilador C++ em Rust

Compilador / analisador para um subconjunto da linguagem C++, construído em Rust. Actualmente implementa três fases: análise léxica, análise sintática (com construção de AST) e análise semântica.

## Estrutura do Projeto

```
src/
├── main.rs                 ← ponto de entrada
├── lexer.rs                ← tokenização (analisador léxico)
├── token.rs                ← definição de tokens
├── ast.rs                  ← tipos de nós da AST (AstKind, AstNode)
├── analise_sintatica.rs    ← parser LL(1) → AST
├── analise_semantica.rs    ← validação semântica → AST decorada
├── scope.rs                ← tabela de símbolos e tipos de escopo
├── utils.rs                ← funções auxiliares
└── files/                  ← ficheiros de teste .cpp
```

## Fases Implementadas

### Fase 1 — Análise Léxica
Converte código-fonte numa sequência de tokens (identificadores, literais, operadores, palavras-chave, delimitadores).

### Fase 2 — Análise Sintática
Parser LL(1) que valida a estrutura gramatical e constrói a Árvore de Sintaxe Abstrata (AST). Suporta classes, herança, funções, arrays, estruturas de controlo, expressões e E/S.

### Fase 3 — Análise Semântica
Valida o significado do código-fonte:
- Variáveis e funções declaradas antes de usar
- Escopos e variable shadowing
- Compatibilidade de tipos (com promoção int↔float↔double)
- Tipos em argumentos de função
- Condições em estruturas de controlo (if/while/for/do-while)
- Tipo de retorno em funções
- Break/Continue em contextos válidos
- Declarações duplicadas
- Construtores dentro de classes

## Como Executar

### Pré-requisitos
- [Rust e Cargo](https://www.rust-lang.org/pt-BR/tools/install)

### Comandos

```bash
# Compilar e executar — processa todos os .cpp em src/files/ → relatorio.txt
cargo run

# Processar um ficheiro específico — imprime apenas erros no stderr
cargo run -- src/files/test_ok.cpp

# Build de produção
cargo build --release
```

### Modo ficheiro único

```bash
cargo run -- src/files/test_undeclared.cpp
```

| Resultado | Saída | Exit code |
|-----------|-------|-----------|
| Sem erros | (nada) | 0 |
| Erro semântico | `nome.cpp: mensagem de erro` (stderr) | 1 |
| Ficheiro não existe | `Ficheiro '...' nao encontrado` (stderr) | 1 |

### Modo todos os ficheiros

```bash
cargo run
```

Processa todos os `.cpp` em `src/files/` e gera `relatorio.txt` com a AST decorada, tabela de símbolos e erros semânticos.

## Documentação

- `mds/MU.md` — Manual do utilizador (lexer)
- `mds/MP.md` — Manual do programador (lexer)
- `mds/MU_PARSER.md` — Manual do utilizador (parser)
- `mds/MP_PARSER.md` — Manual do programador (parser)
- `mds/MU_SEMANTICO.md` — Manual do utilizador (analisador semântico)
- `mds/MP_SEMANTICO.md` — Manual do programador (analisador semântico)
- `documentos/pp2-analise-sintatica/AST.md` — Especificação da AST
