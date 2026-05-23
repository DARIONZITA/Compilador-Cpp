# AST Homogênea — Mini-Compilador C++

> **Versão:** 2.0 — AST pura  
> **Base:** Gramática BNF LL(1)  
> **Escopo deste documento:** estrutura de nós, filhos e normalizações apenas.  
> Regras de escopo, tipos e verificações são responsabilidade do Semantic Checker (documento separado).

---

## 1. Modelo Universal do Nó

Todos os nós obedecem **exactamente** a este contrato:

```
AstNode:
  kind     : AstKind     -- discriminante obrigatório
  children : AstNode[]   -- lista ordenada por semântica executável
  token    : Token?      -- presente apenas em nós folha
```

> **Dois papéis, nunca misturados:**  
> — Nó **interno** → tem `children`, não usa `token`.  
> — Nó **folha** → tem `token`, `children` está vazio.

---

## 2. Catálogo de `AstKind`

```
── Programa ──────────────────
PROGRAM
INCLUDE

── Declarações ───────────────
CLASS_DECL
INHERIT
ACCESS_SECTION
CONSTRUCTOR_DECL
FUNCTION_DECL
VAR_DECL
ARRAY_DECL
PARAM_LIST
PARAM

── Comandos ──────────────────
BLOCK
IF
SWITCH
CASE
DEFAULT
WHILE
DO_WHILE
FOR
RETURN
BREAK
CONTINUE
IO_IN
IO_OUT

── Expressões ────────────────
ASSIGN
BINARY_OP
UNARY_OP
CALL
INDEX
MEMBER_ACCESS
PTR_ACCESS
NEW
THIS

── Terminais ─────────────────
TYPE
MODIFIER
IDENTIFIER
LITERAL

── Recuperação ───────────────
ERROR
```

---

## 3. Programa e Includes

### 3.1 `PROGRAM`

```
PROGRAM
├── INCLUDE*
└── ( CLASS_DECL | FUNCTION_DECL | VAR_DECL | ARRAY_DECL )*
```

**Normalização:** `<include-seq>` e `<declaracao-seq>` são listas recursivas auxiliares — colapsam em `children[]` directos, preservando a ordem do ficheiro fonte.

---

### 3.2 `INCLUDE`

```
INCLUDE
└── LITERAL   (token: caminho — e.g. "iostream" ou <vector>)
```

**Normalização:** `<include-alvo>` eliminado. O delimitador (`<>` vs `""`) é atributo do `token`, não um nó.

---

## 4. Declarações

### 4.1 `CLASS_DECL`

```
CLASS_DECL
├── IDENTIFIER        (nome da classe)
├── INHERIT?
└── ACCESS_SECTION*
```

**Normalização:** `<membros-classe>` e `<lista-membros>` são listas recursivas — colapsam nos `ACCESS_SECTION` directos.  
O `;` terminal é descartado — não representa estrutura.

---

### 4.2 `INHERIT`

```
INHERIT
├── MODIFIER   (token: "public" | "private" | "protected")
└── IDENTIFIER (token: nome da classe-mãe)
```

---

### 4.3 `ACCESS_SECTION`

```
ACCESS_SECTION
├── MODIFIER
└── ( CONSTRUCTOR_DECL | FUNCTION_DECL | VAR_DECL | ARRAY_DECL )*
```

**Normalização:** `<seccao-acesso>` e `<lista-membros>` colapsam. O modificador de acesso é sempre o primeiro filho.

---

### 4.4 `CONSTRUCTOR_DECL`

```
CONSTRUCTOR_DECL
├── IDENTIFIER   (token: nome do construtor)
├── PARAM_LIST
└── BLOCK
```

**Decisão:** nó próprio, distinto de `FUNCTION_DECL`, porque não possui tipo de retorno.

---

### 4.5 `FUNCTION_DECL`

```
FUNCTION_DECL
├── MODIFIER?    (opcional)
├── TYPE
├── IDENTIFIER   (token: nome da função)
├── PARAM_LIST
└── BLOCK?       (ausente = protótipo / declaração forward)
```

**Normalização:** `<sufixo-decl>` e `<corpo-funcao>` eliminados. A presença ou ausência do `BLOCK` distingue definição de protótipo directamente na estrutura.

---

### 4.6 `VAR_DECL`

```
VAR_DECL
├── MODIFIER?
├── TYPE
├── IDENTIFIER
└── expressao?   (inicializador — opcional)
```

**Normalização:** `int a, b = 0, c` gera **três** `VAR_DECL` independentes.  
`<var-resto>` e `<lista-variaveis'>` eliminados.

---

### 4.7 `ARRAY_DECL`

```
ARRAY_DECL
├── MODIFIER?
├── TYPE
├── IDENTIFIER
├── expressao*   (dimensões, uma por `[]`; ausente se `[]` sem valor)
└── LITERAL*     (valores do inicializador `{...}` — opcional)
```

**Normalização:** `<mais-dims>`, `<init-array>`, `<lista-init>` e `<lista-init'>` eliminados. Dimensões e inicializadores são listas planas de filhos, nessa ordem.

---

### 4.8 `PARAM_LIST`

```
PARAM_LIST
└── PARAM*
```

**Decisão:** contentor explícito para representar aridade. Lista vazia = `PARAM_LIST` com `children = []`, nunca omitida.

---

### 4.9 `PARAM`

```
PARAM
├── TYPE
└── IDENTIFIER   (token: nome do parâmetro; atributo is_array=true se sufixo `[]`)
```

**Normalização:** `<param-sufixo>` eliminado. O `[]` é atributo do nó, não um filho.

---

## 5. Terminais de Tipo e Modificador

### 5.1 `TYPE`

```
TYPE
└── token: "int" | "float" | "double" | "char" | "bool" | "void" | "string" | <identificador>
```

Nó folha.

---

### 5.2 `MODIFIER`

```
MODIFIER
└── token: "public" | "private" | "protected" | "static" | "const"
```

Nó folha.

---

## 6. Bloco e Comandos

### 6.1 `BLOCK`

```
BLOCK
└── comando*
```

**Normalização:** `<comando-seq>` é recursão de lista — colapsa em array plano.  
`BLOCK` com `children = []` é válido e preservado (representa bloco vazio).

---

### 6.2 `IF`

```
IF
├── expressao    (condição)          ← filho[0]
├── BLOCK        (ramo then)         ← filho[1]
└── BLOCK | IF   (ramo else/else-if) ← filho[2], opcional
```

O **else-if** é um `IF` no terceiro filho:

```
IF
├── condição A
├── BLOCK (then A)
└── IF
    ├── condição B
    ├── BLOCK (then B)
    └── BLOCK (else final)
```

**Normalização:** `<else-parte>` e `<else-corpo>` eliminados. A estrutura recursiva emerge naturalmente da posição do terceiro filho.

---

### 6.3 `SWITCH`

```
SWITCH
├── expressao          (expressão discriminante)
└── ( CASE | DEFAULT )*
```

**Normalização:** `<casos>` eliminado — filhos directos do `SWITCH`.

---

### 6.4 `CASE`

```
CASE
├── LITERAL
└── BLOCK
```

**Normalização:** `<comando-seq>` do case é embrulhado num `BLOCK` implícito para uniformidade estrutural.

---

### 6.5 `DEFAULT`

```
DEFAULT
└── BLOCK
```

**Decisão:** nó próprio, distinto de `CASE` — não tem `LITERAL`, distinguível sem inspeccionar filhos.

---

### 6.6 `WHILE`

```
WHILE
├── expressao   (condição)
└── BLOCK
```

---

### 6.7 `DO_WHILE`

```
DO_WHILE
├── BLOCK       (corpo) ← filho[0]: executa primeiro
└── expressao   (condição) ← filho[1]: avaliada depois
```

**Decisão:** ordem dos filhos reflecte a **ordem de execução**, não a ordem textual.

---

### 6.8 `FOR`

```
FOR
├── ( VAR_DECL | expressao )?   (init — omitido se `;` vazio)
├── expressao?                  (condição — omitida se vazia)
├── expressao?                  (incremento — omitido se vazio)
└── BLOCK
```

**Normalização:** `<for-init>` e `<expressao-opt>` eliminados. Filhos opcionais ausentes são simplesmente omitidos da lista — não são `ERROR`.

---

### 6.9 `RETURN`

```
RETURN
└── expressao?   (children = [] se return void)
```

---

### 6.10 `BREAK`

```
BREAK   (nó folha — children = [], token = null)
```

---

### 6.11 `CONTINUE`

```
CONTINUE   (nó folha — children = [], token = null)
```

---

### 6.12 `IO_IN`

```
IO_IN
└── expressao*   (alvos do >>, por ordem de leitura)
```

**Normalização:** `<cin-cadeia>` e `<cin-cadeia'>` são recursão de lista — colapsam em `children[]`.

**Exemplo:** `cin >> a >> b >> c`
```
IO_IN
├── IDENTIFIER (a)
├── IDENTIFIER (b)
└── IDENTIFIER (c)
```

---

### 6.13 `IO_OUT`

```
IO_OUT
└── ( expressao | IDENTIFIER("endl") )*
```

**Normalização:** `<cout-cadeia>`, `<cout-cadeia'>` e `<cout-item>` eliminados. `endl` é tratado como `IDENTIFIER` com token `"endl"` — não é um nó especial.

**Exemplo:** `cout << "Olá" << nome << endl`
```
IO_OUT
├── LITERAL     ("Olá")
├── IDENTIFIER  (nome)
└── IDENTIFIER  (endl)
```

---

## 7. Expressões

### 7.1 `ASSIGN`

```
ASSIGN
├── lvalue      (IDENTIFIER | INDEX | MEMBER_ACCESS | PTR_ACCESS)
├── token       (operador: "=" | "+=" | "-=" | "*=" | "/=" | "%=")
└── expressao
```

**Decisão:** nó próprio — distinto de `BINARY_OP` porque o filho esquerdo tem papel estrutural diferente (alvo de escrita).

---

### 7.2 `BINARY_OP`

```
BINARY_OP
├── expressao   (LEFT)
├── token       (operador: "+" | "-" | "*" | "/" | "%" | "==" | "!=" | "<" | ">" | "<=" | ">=" | "&&" | "||")
└── expressao   (RIGHT)
```

**Normalização:** `<atribuicao>`, `<expr-logica>`, `<expr-relacional>`, `<expr-aritmetica>` e `<termo>` eliminados. A precedência é codificada pela **profundidade da árvore** — operadores de menor precedência ficam mais perto da raiz.

**Exemplo:** `a + b * c`
```
BINARY_OP (+)
├── IDENTIFIER (a)
└── BINARY_OP (*)
    ├── IDENTIFIER (b)
    └── IDENTIFIER (c)
```

---

### 7.3 `UNARY_OP`

Cobre **prefixo** e **postfixo** no mesmo nó. O token distingue os dois casos:

```
UNARY_OP
├── token       (operador — ver tabela abaixo)
└── expressao   (operando)
```

| Token no nó   | Origem na gramática | Posição |
|---------------|---------------------|---------|
| `"!"`         | `<unario>`          | prefixo |
| `"-"`         | `<unario>`          | prefixo |
| `"++"`        | `<unario>`          | prefixo |
| `"--"`        | `<unario>`          | prefixo |
| `"&"`         | `<unario>`          | prefixo |
| `"*"`         | `<unario>`          | prefixo (desref.) |
| `"post++"`    | `<postfixo'>`       | postfixo |
| `"post--"`    | `<postfixo'>`       | postfixo |

**Decisão:** prefixo e postfixo reutilizam `UNARY_OP` — o prefixo `"post"` no token resolve a ambiguidade sem criar um novo tipo de nó.

---

### 7.4 `CALL`

```
CALL
├── callee      (IDENTIFIER | MEMBER_ACCESS | PTR_ACCESS)
└── expressao*  (argumentos, por ordem)
```

**Normalização:** `<argumentos>`, `<lista-args>` e `<lista-args'>` eliminados.

**Exemplo:** `foo(a, b + 1)`
```
CALL
├── IDENTIFIER (foo)
├── IDENTIFIER (a)
└── BINARY_OP (+)
    ├── IDENTIFIER (b)
    └── LITERAL (1)
```

---

### 7.5 `INDEX`

```
INDEX
├── base        (expressão — o array ou ponteiro)
└── expressao   (índice)
```

**Exemplo:** `arr[i + 1]`
```
INDEX
├── IDENTIFIER (arr)
└── BINARY_OP (+)
    ├── IDENTIFIER (i)
    └── LITERAL (1)
```

---

### 7.6 `MEMBER_ACCESS`

```
MEMBER_ACCESS
├── expressao   (objecto)
└── IDENTIFIER  (token: nome do membro)
```

Representa o operador `.`

---

### 7.7 `PTR_ACCESS`

```
PTR_ACCESS
├── expressao   (ponteiro)
└── IDENTIFIER  (token: nome do membro)
```

Representa o operador `->`.

**Decisão:** nó próprio — distinto de `MEMBER_ACCESS` pela presença do operador `->` (desreferenciação implícita).  
**Normalização:** `<postfixo'>` eliminado — cada aplicação postfixa gera directamente o nó correspondente.

---

### 7.8 `NEW`

```
NEW
├── TYPE
└── CALL | INDEX   (sufixo: argumentos de construção ou tamanho de array)
```

**Exemplos:**

`new Pessoa("Ana", 30)`
```
NEW
├── TYPE (Pessoa)
└── CALL
    ├── LITERAL ("Ana")
    └── LITERAL (30)
```

`new int[10]`
```
NEW
├── TYPE (int)
└── INDEX
    └── LITERAL (10)
```

**Normalização:** `<new-sufixo>` eliminado — o sufixo é directamente `CALL` ou `INDEX`.

---

### 7.9 `THIS`

```
THIS   (nó folha — children = [], token = null)
```

---

## 8. Terminais

### 8.1 `IDENTIFIER`

```
IDENTIFIER
└── token: lexema   (e.g. "x", "minhaClasse", "_val")
```

Nó folha.

---

### 8.2 `LITERAL`

```
LITERAL
└── token: valor bruto do lexema
            inteiro | real | char | cadeia | "true" | "false" | "nullptr"
```

Nó folha. O token preserva o lexema exacto do ficheiro fonte, sem conversão.

---

## 9. Recuperação de Erros

### 9.1 `ERROR`

```
ERROR
├── token?      (token problemático, se disponível)
└── children?   (sub-árvore parcial, se o parser recuperou algo)
```

**Decisão:** `ERROR` é injectado no lugar de qualquer nó esperado que não pôde ser construído. Não interrompe a árvore envolvente — a construção continua nos irmãos seguintes.

---

## 10. Tabela de Correspondência — Gramática → AST

| Produção gramatical                                         | Nó AST             | Produções auxiliares eliminadas                                           |
|-------------------------------------------------------------|--------------------|---------------------------------------------------------------------------|
| `<programa>`                                                | `PROGRAM`          | `<include-seq>`, `<declaracao-seq>`                                       |
| `<include>`                                                 | `INCLUDE`          | `<include-alvo>`                                                          |
| `<declaracao-classe>`                                       | `CLASS_DECL`       | `<membros-classe>`, `<lista-membros>`                                     |
| `<heranca>`                                                 | `INHERIT`          | —                                                                         |
| `<seccao-acesso>`                                           | `ACCESS_SECTION`   | `<lista-membros>`                                                         |
| `<declaracao-construtor>`                                   | `CONSTRUCTOR_DECL` | —                                                                         |
| `<modificador> <tipo> <id> "(" ... ")" <bloco>`             | `FUNCTION_DECL`    | `<sufixo-decl>`, `<corpo-funcao>`                                         |
| `<modificador> <tipo> <id> <var-resto>` sem `[`             | `VAR_DECL`         | `<var-resto>`, `<lista-variaveis'>`                                       |
| `<modificador> <tipo> <id> <var-resto>` com `[`             | `ARRAY_DECL`       | `<var-resto>`, `<mais-dims>`, `<init-array>`, `<lista-init>`, `<lista-init'>` |
| `<parametros>`                                              | `PARAM_LIST`       | `<lista-parametros>`, `<lista-parametros'>`                               |
| `<parametro>`                                               | `PARAM`            | `<param-sufixo>`                                                          |
| `<bloco>`                                                   | `BLOCK`            | `<comando-seq>`                                                           |
| `<comando-seleccao>` — if                                   | `IF`               | `<else-parte>`, `<else-corpo>`                                            |
| `<comando-seleccao>` — switch                               | `SWITCH`           | `<casos>`                                                                 |
| `<caso>` — case                                             | `CASE`             | —                                                                         |
| `<caso>` — default                                          | `DEFAULT`          | —                                                                         |
| `<comando-repeticao>` — while                               | `WHILE`            | —                                                                         |
| `<comando-repeticao>` — do-while                            | `DO_WHILE`         | —                                                                         |
| `<comando-repeticao>` — for                                 | `FOR`              | `<for-init>`, `<expressao-opt>`                                           |
| `<comando-io>` — cin                                        | `IO_IN`            | `<cin-cadeia>`, `<cin-cadeia'>`                                           |
| `<comando-io>` — cout                                       | `IO_OUT`           | `<cout-cadeia>`, `<cout-cadeia'>`, `<cout-item>`                          |
| `<atribuicao>` com operador de atribuição                   | `ASSIGN`           | `<atribuicao'>`, `<op-atrib>`                                             |
| `<expr-logica>` / `<expr-relacional>` / `<expr-aritmetica>` / `<termo>` | `BINARY_OP` | Todas as produções intermédias de expressão                   |
| `<unario>`                                                  | `UNARY_OP`         | `<factor>`                                                                |
| `<postfixo'>` — `++` / `--`                                 | `UNARY_OP`         | `<postfixo'>`                                                             |
| `<postfixo'>` — `[...]`                                     | `INDEX`            | `<postfixo'>`                                                             |
| `<postfixo'>` — `(...)`                                     | `CALL`             | `<postfixo'>`, `<argumentos>`, `<lista-args>`, `<lista-args'>`            |
| `<postfixo'>` — `.`                                         | `MEMBER_ACCESS`    | `<postfixo'>`                                                             |
| `<postfixo'>` — `->`                                        | `PTR_ACCESS`       | `<postfixo'>`                                                             |
| `<primario>` — new                                          | `NEW`              | `<new-sufixo>`                                                            |
| `<primario>` — this                                         | `THIS`             | —                                                                         |
| `<identificador>`                                           | `IDENTIFIER`       | —                                                                         |
| `<literal>`                                                 | `LITERAL`          | `<inteiro>`, `<real>`, `<caracter>`, `<cadeia>`                           |
| — recuperação de erros                                      | `ERROR`            | —                                                                         |

---

## 11. Exemplos

### 11.1 Classe com construtor e método

```cpp
class Contador {
public:
    int valor;
    Contador(int v) { valor = v; }
    int incrementar(int n) {
        valor += n;
        return valor;
    }
};
```

```
PROGRAM
└── CLASS_DECL
    ├── IDENTIFIER (Contador)
    └── ACCESS_SECTION
        ├── MODIFIER (public)
        ├── VAR_DECL
        │   ├── TYPE (int)
        │   └── IDENTIFIER (valor)
        ├── CONSTRUCTOR_DECL
        │   ├── IDENTIFIER (Contador)
        │   ├── PARAM_LIST
        │   │   └── PARAM
        │   │       ├── TYPE (int)
        │   │       └── IDENTIFIER (v)
        │   └── BLOCK
        │       └── ASSIGN (=)
        │           ├── IDENTIFIER (valor)
        │           └── IDENTIFIER (v)
        └── FUNCTION_DECL
            ├── TYPE (int)
            ├── IDENTIFIER (incrementar)
            ├── PARAM_LIST
            │   └── PARAM
            │       ├── TYPE (int)
            │       └── IDENTIFIER (n)
            └── BLOCK
                ├── ASSIGN (+=)
                │   ├── IDENTIFIER (valor)
                │   └── IDENTIFIER (n)
                └── RETURN
                    └── IDENTIFIER (valor)
```

---

### 11.2 For com else-if encadeado

```cpp
for (int i = 0; i < 10; i++) {
    if (i % 2 == 0) {
        cout << i << endl;
    } else if (i == 5) {
        break;
    } else {
        continue;
    }
}
```

```
FOR
├── VAR_DECL
│   ├── TYPE (int)
│   ├── IDENTIFIER (i)
│   └── LITERAL (0)
├── BINARY_OP (<)
│   ├── IDENTIFIER (i)
│   └── LITERAL (10)
├── UNARY_OP (post++)
│   └── IDENTIFIER (i)
└── BLOCK
    └── IF
        ├── BINARY_OP (==)
        │   ├── BINARY_OP (%)
        │   │   ├── IDENTIFIER (i)
        │   │   └── LITERAL (2)
        │   └── LITERAL (0)
        ├── BLOCK
        │   └── IO_OUT
        │       ├── IDENTIFIER (i)
        │       └── IDENTIFIER (endl)
        └── IF                         ← else-if como IF no terceiro filho
            ├── BINARY_OP (==)
            │   ├── IDENTIFIER (i)
            │   └── LITERAL (5)
            ├── BLOCK
            │   └── BREAK
            └── BLOCK
                └── CONTINUE
```

---

### 11.3 Switch / Case

```cpp
switch (op) {
    case 1: x = a + b; break;
    case 2: x = a - b; break;
    default: x = 0;
}
```

```
SWITCH
├── IDENTIFIER (op)
├── CASE
│   ├── LITERAL (1)
│   └── BLOCK
│       ├── ASSIGN (=)
│       │   ├── IDENTIFIER (x)
│       │   └── BINARY_OP (+)
│       │       ├── IDENTIFIER (a)
│       │       └── IDENTIFIER (b)
│       └── BREAK
├── CASE
│   ├── LITERAL (2)
│   └── BLOCK
│       ├── ASSIGN (=)
│       │   ├── IDENTIFIER (x)
│       │   └── BINARY_OP (-)
│       │       ├── IDENTIFIER (a)
│       │       └── IDENTIFIER (b)
│       └── BREAK
└── DEFAULT
    └── BLOCK
        └── ASSIGN (=)
            ├── IDENTIFIER (x)
            └── LITERAL (0)
```

---

### 11.4 Acesso a membro e chamada encadeada

```cpp
obj.metodo(x)->campo[i]
```

```
INDEX
├── PTR_ACCESS
│   ├── CALL
│   │   ├── MEMBER_ACCESS
│   │   │   ├── IDENTIFIER (obj)
│   │   │   └── IDENTIFIER (metodo)
│   │   └── IDENTIFIER (x)
│   └── IDENTIFIER (campo)
└── IDENTIFIER (i)
```

---

*Fim da especificação — AST Homogênea v2.0*