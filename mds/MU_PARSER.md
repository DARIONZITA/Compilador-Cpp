# Manual do Utilizador (MU) — Analisador Sintático LL(1)

## 1. Objetivo

Este documento descreve como usar o **analisador sintático** (parser) do compilador C++. O parser recebe uma sequência de tokens do lexer e **valida a estrutura sintática** do programa.

**Nota importante**: A construção de uma Árvore de Sintaxe Abstrata (AST) foi conceitualmente arquitetada em `AST.md`, mas ainda **não foi implementada**. O parser atual realiza apenas validação sintática, sem produzir AST.

As próximas fases do compilador (análise semântica, otimização, geração de código) aguardam a implementação do AST Builder.

---

## 2. Fluxo geral do compilador

```
Código-fonte (arquivo .cpp)
        ↓
   [LEXER]  → tokens
        ↓
   [PARSER] → validação sintática ✓ (implementado)
        ↓
   [AST BUILDER] → Árvore de Sintaxe Abstrata (em desenvolvimento)
        ↓
[SEMANTIC CHECKER] → validações de tipo, escopo, etc.
        ↓
[CODE GENERATOR] → código executável
```

**Estado atual**: Fase 1 (Lexer) e Fase 2 (Parser validação) funcionais. Fase 3+ não implementadas.

---

## 3. Como usar o parser

### 3.1 Pré-requisitos

- Rust e Cargo instalados
- Arquivo de entrada em `src/files/main.cpp`
- Execução via `cargo run`

### 3.2 Execução

No diretório raiz do projeto:

```bash
cargo run
```

**Saída esperada:**

1. **Lista de tokens** (do lexer)
2. **Resultado da validação sintática** (sucesso ou erros)
3. **Mensagens informativas**

### 3.3 Exemplo de entrada (src/files/main.cpp)

```cpp
#include <stdio.h>

int main() {
    int x = 5;
    if (x > 3) {
        printf("Maior que 3\n");
    }
    return 0;
}
```

### 3.4 Exemplo de saída esperada

```
─── TOKENS ───
Include => "include" : 1
String => "stdio.h" : 1
Int => "int" : 3
Identificador => "main" : 3
AbreParen => "(" : 3
FechaParen => ")" : 3
AbreChave => "{" : 3
...
─── FIM DOS TOKENS ───

Iniciando analise sintatica...
Analise sintatica bem-sucedida!
```

**Notas:**
- Se houver erros, aparecem mensagens do tipo `Erro [linha X]: Esperado Y, mas encontrado Z`
- Se aparecer `Aviso: O analisador parou prematuramente...`, significa que nem todos os tokens foram consumidos

---

## 4. Conceitual: Estrutura da AST (planeada)

A AST será construída em fases futuras. Consulte [AST.md](AST.md) para compreender a arquitetura planeada:

- **Estrutura de nós** — kind, children, token
- **Catálogo de 44 tipos de nós** — Program, Block, If, BinaryOp, etc.
- **Mapeamento gramática → AST** — como cada construção C++ será representada
- **Normalização** — eliminação de produções auxiliares
- **Exemplos práticos** — If/Else, For loops, Switch/Case, classes

**Importante**: O documento `AST.md` é a especificação completa arquitetada para a AST homogênea, mas a implementação ainda não foi realizada no código.

---

## 5. Tratamento de erros

### 5.1 Erros de parsing

Quando o parser encontra um token inesperado, emite mensagem:

```
Erro [linha 5]: Esperado ';', mas encontrado 'int'
```

**Componentes da mensagem:**
- `[linha X]` — localização do erro
- `Esperado Y` — o que o parser esperava
- `encontrado Z` — o que foi recebido

### 5.2 Recuperação de erros (Panic Mode)

O parser **não interrompe** a análise quando encontra erro. Ao invés disso:

1. Emite mensagem de erro
2. Pula tokens até encontrar um "token seguro" (FOLLOW set)
3. Continua a análise

**Exemplo**: Falta de `;`
```cpp
int x = 5    // falta ;
int y = 10;
```

**Saída:**
```
Erro [linha 1]: Esperado ';', mas encontrado 'int'
```

O parser recupera-se e continua analisando `int y = 10;` normalmente.

---

## 6. Limitações atuais

1. **Sem construção de AST** — parser apenas valida sintaxe
   - Não gera árvore intermediária
   - AST foi arquitetada em `AST.md`, implementação planeada

2. **Sem análise semântica** — o parser não verifica tipos ou escopo
   - `int x = "string"` é sintacticamente válido
   - `int x; int x;` não causa erro (duplicação permitida)

3. **Subconjunto de C++** — não suporta:
   - Templates
   - Namespaces
   - Herança múltipla (apenas herança simples)
   - Operador ternário `? :`
   - Algumas construções OOP avançadas

4. **Parâmetros-padrão** não suportados
   - `void foo(int x = 5)` não funciona

5. **Atributos de array** limitados
   - `int arr[10][20]` funciona, mas nem todos os casos multidimensionais

---

## 7. Estrutura de ficheiros

```
compilator_cplusplus/
├── src/
│   ├── main.rs                    ← ponto de entrada
│   ├── lexer.rs                   ← tokenização (DFA)
│   ├── token.rs                   ← definição de tokens
│   ├── analise_sintatica.rs       ← parser LL(1) de validação (atual)
│   ├── utils.rs                   ← funções auxiliares
│   └── files/
│       └── main.cpp               ← arquivo de entrada para teste
├── AST.md                         ← especificação da AST (arquitetura)
├── MU_PARSER.md                   ← este ficheiro (manual do utilizador)
├── MP_PARSER.md                   ← manual do programador
├── gramatica.bnf                  ← gramática formal
└── Cargo.toml                     ← configuração do projeto
```

---

## 8. Fluxo rápido de teste

1. **Edite** `src/files/main.cpp` com código C++
2. Execute `cargo run`
3. Verifique a saída:
   - Se houver `Analise sintatica bem-sucedida!`, validação OK
   - Se houver `Erro [linha X]`, revise o código nessa linha
4. Observe se faltam tokens no final (aviso de parada prematura)

---

## 9. Exemplos de código testável

### 9.1 Programa simples

```cpp
#include <stdio.h>

int main() {
    int x = 10;
    printf("%d\n", x);
    return 0;
}
```

### 9.2 Classes

```cpp
class Pessoa {
public:
    int idade;
    Pessoa(int i) {
        idade = i;
    }
};

int main() {
    Pessoa p(25);
    return 0;
}
```

### 9.3 Expressões complexas

```cpp
int main() {
    int a = 5, b = 3;
    int c = a + b * 2;
    if (c > 10 && a < b) {
        c = 100;
    }
    return 0;
}
```

### 9.4 Loops

```cpp
int main() {
    for (int i = 0; i < 10; i++) {
        if (i % 2 == 0) {
            continue;
        }
        printf("%d\n", i);
    }
    return 0;
}
```

---

## 10. Glossário

| Termo | Significado |
|-------|------------|
| **Token** | unidade léxica (palavra-chave, identificador, operador) |
| **AST** | Árvore de Sintaxe Abstrata (arquitetada em AST.md, não implementada) |
| **Parser** | analisador sintático (valida sintaxe a partir de tokens) |
| **LL(1)** | tipo de parser: lê de esquerda para direita, decididas com 1 token de lookahead |
| **FOLLOW set** | conjunto de tokens que podem seguir uma produção gramatical |
| **Panic mode** | técnica de recuperação de erro (pula tokens até encontrar token seguro) |
| **BNF** | Backus-Naur Form (notação para gramática formal) |
| **Validação sintática** | verificação se código segue as regras gramaticais (sem análise semântica) |

---

## 11. Próximas etapas

As próximas fases do compilador são:

1. **AST Builder** — construir Árvore de Sintaxe Abstrata
   - Implementar módulo `ast.rs` com tipos de nós
   - Reescrever `analise_sintatica_new.rs` com construção de nós
   - Normalizar produções auxiliares (ver `AST.md`)

2. **Semantic Checker** — valida tipos, escopo, declarações
   - Verificar declarações antes do uso
   - Validar operações de tipo
   - Resolver escopo de nomes

3. **Otimizador** — simplifica código intermediário

4. **Code Generator** — traduz AST para código-máquina

---

## 12. Suporte e debugging

### 12.1 Se houver muitos erros de parsing

- Comece com código mínimo (`#include <stdio.h>` + `int main() { return 0; }`)
- Adicione construções incrementalmente
- Consulte `MP_PARSER.md` para entender a gramática LL(1)

### 12.2 Se o parser parar prematuramente

- Verifique se há tokens não consumidos
- Mensagem: `Aviso: O analisador parou prematuramente...`
- Significa que a análise terminou antes de alcançar EOF

### 12.3 Estrutura da AST planeada

Consulte `AST.md` para ver a forma esperada de cada construção na AST que será implementada.

---

*Fim do Manual do Utilizador — Analisador Sintático LL(1)*
