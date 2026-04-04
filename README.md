# Compilador C++ em Rust

Este projeto é um compilador / analisador léxico (lexer) para subconjuntos da linguagem C++ construído utilizando a linguagem Rust.

## Estrutura do Projeto

- `src/main.rs` - Ponto de entrada (entrypoint) principal da aplicação.
- `src/lexer.rs` - Implementação do analisador léxico responsável por converter o código-fonte num fluxo de tokens.
- `src/token.rs` - Definição dos tipos, palavras-chave e estruturas de tokens suportados do C++.
- `src/utils.rs` - Funções e macros utilitárias auxiliares.
- `src/files/main.cpp` - Código-fonte C++ de exemplo usado para testar a análise léxica.

## Como Executar

Para rodar este projeto, você precisará ter o [Rust (rustc e cargo) instalados](https://www.rust-lang.org/pt-BR/tools/install) em sua máquina.

1. Navegue até a pasta do projeto (onde está o `Cargo.toml`):
   ```bash
   cd compilator_cplusplus
   ```

2. Para construir e executar o analisador léxico diretamente:
   ```bash
   cargo run
   ```

3. Para gerar a build de produção (otimizada):
   ```bash
   cargo build --release
   ```

## Objetivos e Funcionamento
O projeto lê o arquivo alvo (como `src/files/main.cpp`) e o analisa caractere por caractere ou em blocos, classificando-os de acordo com as regras de gramática léxica do C++ definidas (palavras-chave, identificadores, literais, operadores, etc). A saída esperada é a listagem dos tokens reconhecidos.
