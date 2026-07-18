# C++ Compiler in Rust

A compiler / analyzer for a subset of the C++ language, built in Rust. Currently implements three phases: lexical analysis, syntactic analysis (with AST construction), and semantic analysis.

## Project Structure

```
src/
├── main.rs                 ← entry point
├── lexer.rs                ← tokenizer (lexical analyzer)
├── token.rs                ← token definitions
├── ast.rs                  ← AST node types (AstKind, AstNode)
├── analise_sintatica.rs    ← LL(1) parser → AST
├── analise_semantica.rs    ← semantic validation → decorated AST
├── scope.rs                ← symbol table and scope types
├── utils.rs                ← utility functions
└── files/                  ← .cpp test files
```

## Implemented Phases

### Phase 1 — Lexical Analysis
Converts source code into a stream of tokens (identifiers, literals, operators, keywords, delimiters).

### Phase 2 — Syntactic Analysis
LL(1) parser that validates grammatical structure and builds the Abstract Syntax Tree (AST). Supports classes, inheritance, functions, arrays, control flow, expressions, and I/O.

### Phase 3 — Semantic Analysis
Validates the meaning of source code:
- Variables and functions declared before use
- Scopes and variable shadowing
- Type compatibility (with int↔float↔double promotion)
- Function argument types
- Conditions in control structures (if/while/for/do-while)
- Function return types
- Break/Continue in valid contexts
- Duplicate declarations
- Constructors inside classes

## How to Run

### Prerequisites
- [Rust and Cargo](https://www.rust-lang.org/tools/install)

### Commands

```bash
# Build and run — process all .cpp files in src/files/ → relatorio.txt
cargo run

# Process a specific file — print only errors to stderr
cargo run -- src/files/test_ok.cpp

# Production build
cargo build --release
```

### Single File Mode

```bash
cargo run -- src/files/test_undeclared.cpp
```

| Result | Output | Exit code |
|--------|--------|-----------|
| No errors | (nothing) | 0 |
| Semantic error | `filename.cpp: error message` (stderr) | 1 |
| File not found | `File '...' not found` (stderr) | 1 |

### All Files Mode

```bash
cargo run
```

Processes all `.cpp` files in `src/files/` and generates `relatorio.txt` with decorated AST, symbol table, and semantic errors.

## Documentation

- `mds/MU.md` — User manual (lexer)
- `mds/MP.md` — Programmer manual (lexer)
- `mds/MU_PARSER.md` — User manual (parser)
- `mds/MP_PARSER.md` — Programmer manual (parser)
- `mds/MU_SEMANTICO.md` — User manual (semantic analyzer)
- `mds/MP_SEMANTICO.md` — Programmer manual (semantic analyzer)
- `documentos/pp2-analise-sintatica/AST.md` — AST specification
