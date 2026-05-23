# Manual do Usuario (MU)

## 1. Objetivo

Este programa e um analisador lexico para um subconjunto de C/C++. Ele le um arquivo de codigo-fonte, percorre caractere por caractere e converte o texto em uma sequencia de tokens.

Um token representa uma unidade lexico-sintatica, por exemplo:

- identificador (nome de variavel)
- literal inteiro
- literal string
- operador (+, ==, &&, etc.)
- delimitador (;, (), {}, etc.)
- palavra reservada (if, while, return, class, ...)

## 2. Estrutura esperada de projeto

A implementacao atual le o arquivo de entrada em caminho fixo:

- src/files/main.cpp

Portanto, para usar o analisador sem alterar codigo, o arquivo de teste deve estar nesse caminho.

## 3. Como executar

### 3.1. Pre-requisitos

- Rust e Cargo instalados
- Projeto aberto na pasta compilator_cplusplus

### 3.2. Comando de execucao

No diretorio raiz do projeto, execute:

cargo run

## 4. Formato de entrada

A entrada e um arquivo texto com codigo C/C++ (ou proximo disso), por exemplo:

```cpp
#include <stdio.h>
int main() {
    int k3;
    k3 = 10;
    for (int i = 1; i < k3; i++) {
        printf("HELLO WORLD!\\n");
    }
    return 0;
}
```

### 4.1. O que o analisador reconhece

- identificadores e palavras reservadas
- inteiros e floats simples
- strings e chars (com escape)
- operadores aritmeticos, relacionais, logicos e bitwise
- comentarios de linha e bloco
- delimitadores e pontuacao

## 5. Formato de saida

A saida e textual no console e possui duas partes:

1. Lista de tokens, um por linha, no formato:

KIND => "lexema"

Na implementacao atual, a impressao usa Debug:

TokenKind => "lexema"

2. Em seguida, o programa imprime novamente o conteudo completo do arquivo de entrada.

### 5.1. Exemplo de saida (resumido)

```text
Int => "int"
Identificador => "main"
AbreParen => "("
FechaParen => ")"
AbreChave => "{"
Return => "return"
Inteiro => "0"
PtVirgula => ";"
FechaChave => "}"
```

## 6. Como interpretar os resultados

- Se um lexema de nome de variavel aparecer como Identificador, o reconhecimento esta correto.
- Se aparecer como Int, While, Return, etc., ele foi classificado como palavra reservada.
- Operadores compostos como ==, +=, >> sao reconhecidos como tokens especificos.
- Comentarios tambem viram tokens (ComentarioLinha e ComentarioBloco).

## 7. Limitacoes atuais

- Caminho de entrada fixo em src/files/main.cpp.
- Nao existe ainda tratamento formal de erro lexico com linha/coluna.
- O analisador foca na tokenizacao; ele nao faz analise sintatica.
- Alguns casos de literais complexos de C++ (sufixos, notacao cientifica, unicode, raw string) nao estao modelados completamente.

## 8. Solucao de problemas

### 8.1. Erro ao ler arquivo

Mensagem tipica: nao foi possivel ler o ficheiro

Verifique:

- se src/files/main.cpp existe
- se o processo tem permissao de leitura
- se esta executando no diretorio correto

### 8.2. Cargo nao encontrado

Instale Rust e valide:

rustc --version
cargo --version

### 8.3. Tokens inesperados

- revise escapes de string/char
- revise comentarios de bloco nao fechados
- confira caracteres especiais fora do conjunto esperado

## 9. Fluxo rapido recomendado

1. Coloque/edite codigo em src/files/main.cpp.
2. Execute cargo run.
3. Verifique a lista de tokens no console.
4. Ajuste o codigo de teste e execute novamente.

## 10. Resumo

Este analisador lexico transforma texto-fonte em tokens de forma deterministica, servindo como base para etapas futuras de compilador (parser, analise semantica e geracao de codigo).
