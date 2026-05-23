# Manual do Programador (MP)

## 1. Visao geral tecnica

Este documento descreve a arquitetura interna do analisador lexico implementado em Rust, incluindo:

- lista completa dos tokens
- automato finito deterministico (AFD)
- expressoes regulares de categorias lexicas
- pseudo-codigo de cada funcao

Arquivos principais:

- src/main.rs
- src/lexer.rs
- src/token.rs
- src/utils.rs
- dfa-definition (2).yaml

O fluxo geral do programa e:

1. Ler arquivo fonte
2. Tokenizar com automato
3. Imprimir tokens
4. Imprimir conteudo original

---

## 2. Arquitetura em modulos

### 2.1. Modulo token

Responsabilidade:

- definir o enum Token
- definir a estrutura TokenInfo
- definir o enum Estado do automato

### 2.2. Modulo utils

Responsabilidade:

- funcoes auxiliares de classificacao de caracteres
- insercao de token com limpeza de lexema

### 2.3. Modulo lexer

Responsabilidade:

- logica completa do AFD
- funcoes de leitura/retrocesso de caractere
- identificacao de palavras reservadas
- funcao principal de tokenizacao

### 2.4. Modulo main

Responsabilidade:

- ponto de entrada
- leitura de arquivo
- chamada do lexer
- impressao do resultado

---

## 3. Estruturas de dados

### 3.1. TokenInfo

Representa uma ocorrencia lexico:

- kind: Token
- lexema: String

### 3.2. Estado

Modela o estado atual do AFD.

Estados definidos:

- Inicio
- Identificador
- Inteiro
- FloatPonto
- FloatDigitos
- StringAberta
- StringEscape
- CharAberto
- CharEscape
- CharConteudo
- OpMais
- OpMenos
- OpVezes
- OpDiv
- OpMod
- OpIgual
- OpMenor
- OpMaior
- OpExclamacao
- OpE
- OpOu
- OpDoisPontos
- ComentarioLinha
- ComentarioBloco
- ComentarioBlocoAst
- Outro

O estado Outro indica que o caractere atual nao foi consumido pelo token finalizado e precisa ser reprocessado no proximo ciclo.

---

## 4. Lista completa de tokens utilizados

Abaixo esta a lista completa do enum Token conforme implementacao atual.

### 4.1. Tokens gerais

- Identificador
- Inteiro
- Float
- String
- Char

### 4.2. Operadores e pontuacao base

- Adicao
- Incremento
- MaisIgual
- Subtracao
- Decremento
- MenosIgual
- Seta
- Multiplicacao
- VezesIgual
- Divisao
- DivIgual
- Modulo
- ModIgual
- Atribuicao
- Igual
- Menor
- Maior
- MenorIgual
- MaiorIgual
- Not
- Diferente
- And
- Or
- BitAnd
- BitOr
- BitXor
- BitNot
- ShiftEsq
- ShiftDir
- PontoMembro
- Escopo
- DoisPontos
- Interrogacao

### 4.3. Comentarios

- ComentarioLinha
- ComentarioBloco

### 4.4. Delimitadores

- PtVirgula
- Virgula
- AbreParen
- FechaParen
- AbreChave
- FechaChave
- AbreColch
- FechaColch

### 4.5. Palavras reservadas e termos de linguagem

- If
- Else
- While
- For
- Return
- Int
- FloatType
- CharType
- Void
- Class
- Struct
- Enum
- Const
- Static
- Public
- Private
- Protected
- Virtual
- Override
- Abstract
- Template
- Typedef
- Namespace
- Using
- Include
- Long
- Short
- Signed
- Unsigned

### 4.6. Controle de fluxo

- Do
- Switch
- Case
- Break
- Continue
- Goto
- Default

### 4.7. Tipos primitivos adicionais

- Bool
- Double

### 4.8. Memoria e ponteiros

- New
- Delete
- Sizeof

### 4.9. Operadores por palavras-chave

- AndEq
- OrEq
- XorEq
- Xor
- Bitand
- Bitor
- Compl
- NotEq

### 4.10. OOP e tipos especiais

- This
- Inline
- Explicit
- Friend
- Operator
- Typename

### 4.11. Excecoes

- Try
- Catch
- Throw

### 4.12. Cast

- StaticCast
- DynamicCast
- ConstCast
- ReinterpretCast

### 4.13. Outros

- Auto
- Register
- Extern
- Volatile
- Mutable
- Export
- TrueLiteral
- FalseLiteral
- Typeid

---

## 5. Automato finito deterministico

## 5.1. Definicao formal

A especificacao formal principal esta em:

- dfa-definition (2).yaml

O lexer em src/lexer.rs implementa o comportamento equivalente em codigo, com o detalhe adicional de retrocesso de caractere via estado Outro.

## 5.2. Ideia operacional

O automato consome um caractere por iteracao e decide:

- continuar acumulando lexema no estado atual
- mudar de estado para completar token composto
- finalizar token e ir para Outro para reprocessar caractere delimitador

## 5.3. Diagrama simplificado (alto nivel)

```mermaid
stateDiagram-v2
    [*] --> Inicio

    Inicio --> Identificador: letra/_
    Inicio --> Inteiro: digito
    Inicio --> StringAberta: "
    Inicio --> CharAberto: '

    Inicio --> OpMais: +
    Inicio --> OpMenos: -
    Inicio --> OpVezes: *
    Inicio --> OpDiv: /
    Inicio --> OpMod: %
    Inicio --> OpIgual: =
    Inicio --> OpMenor: <
    Inicio --> OpMaior: >
    Inicio --> OpExclamacao: !
    Inicio --> OpE: &
    Inicio --> OpOu: |
    Inicio --> OpDoisPontos: :

    Identificador --> Identificador: letra/digito/_
    Identificador --> Outro: delimitador

    Inteiro --> Inteiro: digito
    Inteiro --> FloatPonto: .
    Inteiro --> Outro: delimitador

    FloatPonto --> FloatDigitos: digito
    FloatPonto --> Outro: delimitador

    FloatDigitos --> FloatDigitos: digito
    FloatDigitos --> Outro: delimitador

    StringAberta --> StringEscape: \\
    StringEscape --> StringAberta: qualquer
    StringAberta --> Inicio: " (fecha)

    CharAberto --> CharEscape: \\
    CharEscape --> CharConteudo: qualquer
    CharAberto --> CharConteudo: qualquer
    CharConteudo --> Inicio: ' (fecha)
    CharConteudo --> Outro: sem fechamento

    OpDiv --> ComentarioLinha: /
    OpDiv --> ComentarioBloco: *
    ComentarioLinha --> Outro: \n ou EOF
    ComentarioBloco --> ComentarioBlocoAst: *
    ComentarioBlocoAst --> Inicio: /

    Outro --> Inicio: voltar_caractere + reset
```

## 5.4. Papel do estado Outro

Sem o estado Outro, um delimitador lido apos um token poderia ser perdido.

Exemplo conceitual:

- entrada: abc;
- ao sair de Identificador em ;, o lexer fecha abc
- ; ainda precisa virar token PtVirgula

Solucao implementada:

1. Estado Identificador finaliza token e vai para Outro.
2. Loop chama voltar_caractere(i).
3. Estado e resetado para Inicio.
4. Mesmo caractere ; e lido novamente e tokenizado.

---

## 6. Expressoes regulares das classes lexicas

As expressoes abaixo representam o comportamento pretendido do lexer (abstracao do AFD):

### 6.1. Identificador

```regex
[A-Za-z_][A-Za-z0-9_]*
```

### 6.2. Inteiro decimal

```regex
[0-9]+
```

### 6.3. Float simples

```regex
[0-9]+\.[0-9]*
```

Observacao: no codigo atual, um numero seguido de ponto sem digitos adicionais tambem pode ser finalizado como Float.

### 6.4. String com escape

```regex
"([^"\\]|\\.)*"
```

### 6.5. Char com ou sem escape

```regex
'([^'\\]|\\.)'
```

### 6.6. Comentario de linha

```regex
//[^\n]*
```

### 6.7. Comentario de bloco

```regex
/\*([\s\S]*?)\*/
```

### 6.8. Operadores compostos relevantes

```regex
\+\+|\+=|--|-=|->|\*=|/=|%=|==|<=|>=|!=|&&|\|\||<<|>>|::
```

### 6.9. Delimitadores

```regex
[;,(){}\[\]\.\?:]
```

### 6.10. Espacos em branco

```regex
[ \t\n\r]+
```

---

## 7. Pseudo-codigo de cada funcao

A seguir, pseudo-codigo das funcoes implementadas nos modulos atuais.

## 7.1. src/main.rs

### 7.1.1 main

```text
funcao main()
    conteudo <- ler arquivo "src/files/main.cpp"
    tokens <- tokenizar(conteudo)

    para cada token em tokens
        imprimir token.kind e token.lexema
    fim

    imprimir conteudo
fim
```

## 7.2. src/utils.rs

### 7.2.1 eh_letra(c)

```text
funcao eh_letra(c)
    retornar c e letra ASCII
fim
```

### 7.2.2 eh_digito(c)

```text
funcao eh_digito(c)
    retornar c e digito ASCII
fim
```

### 7.2.3 eh_espaco(c)

```text
funcao eh_espaco(c)
    retornar verdadeiro se c for ' ', '\t', '\n' ou '\r'
fim
```

### 7.2.4 push_token(tokens, kind, lexema)

```text
funcao push_token(tokens, kind, lexema)
    se lexema nao estiver vazio
        adicionar TokenInfo { kind, copia(lexema) } em tokens
        limpar lexema
    fim
fim
```

## 7.3. src/lexer.rs

### 7.3.1 ler_caractere(chars, index)

```text
funcao ler_caractere(chars, index)
    se index < tamanho(chars)
        retornar chars[index]
    senao
        retornar '\0'
fim
```

### 7.3.2 voltar_caractere(index)

```text
funcao voltar_caractere(index)
    se index > 0
        retornar index - 1
    senao
        retornar 0
fim
```

### 7.3.3 identificadores_reservados(lexema)

```text
funcao identificadores_reservados(lexema)
    comparar lexema com tabela de palavras reservadas
    se houver correspondencia
        retornar token reservado correspondente
    senao
        retornar Token::Identificador
fim
```

### 7.3.4 analex(atual, tokens, estado, lexema)

```text
funcao analex(atual, tokens, estado, lexema)
    escolha estado atual

    caso Inicio:
        decidir classe de atual
        iniciar lexema e mudar para estado adequado
        ou emitir token imediato para pontuacao simples

    caso Identificador:
        se atual for [letra|digito|_]
            acumular
        senao
            token <- identificadores_reservados(lexema)
            emitir token
            estado <- Outro

    caso Inteiro/FloatPonto/FloatDigitos:
        acumular numeros e ponto
        ao fechar, emitir Inteiro ou Float
        usar Outro quando delimitador nao consumido

    caso StringAberta/StringEscape:
        tratar escapes
        fechar no delimitador "

    caso CharAberto/CharEscape/CharConteudo:
        tratar literal de char
        fechar em '
        em fechamento irregular, emitir e usar Outro

    casos OpMais, OpMenos, OpVezes, OpDiv, OpMod,
          OpIgual, OpMenor, OpMaior, OpExclamacao,
          OpE, OpOu, OpDoisPontos:
        reconhecer operador simples/composto
        emitir token
        quando proximo caractere nao pertence ao operador,
        transicionar para Outro

    casos ComentarioLinha, ComentarioBloco, ComentarioBlocoAst:
        acumular ate fechamento apropriado
        emitir token de comentario

    caso Outro:
        nao processa, apenas aguardando loop externo
fim
```

### 7.3.5 tokenizar(conteudo)

```text
funcao tokenizar(conteudo)
    tokens <- lista vazia
    chars <- vetor de caracteres de conteudo

    estado <- Inicio
    lexema <- string vazia
    i <- 0

    enquanto i <= tamanho(chars)
        atual <- ler_caractere(chars, i)

        analex(atual, tokens, estado, lexema)

        se estado == Outro
            i <- voltar_caractere(i)
            estado <- Inicio
        fim

        i <- i + 1
    fim

    retornar tokens
fim
```

---

## 8. Tabela de palavras reservadas mapeadas

O mapeamento atual em identificadores_reservados cobre:

- if, else, while, for, return
- int, float, char, void, bool, double
- class, struct, enum, const, static
- public, private, protected, virtual, override, abstract
- template, typedef, namespace, using, include
- long, short, signed, unsigned
- do, switch, case, break, continue, goto, default
- new, delete, sizeof
- and, or, not, and_eq, or_eq, xor_eq, xor, bitand, bitor, compl, not_eq
- this, inline, explicit, friend, operator, typename
- try, catch, throw
- static_cast, dynamic_cast, const_cast, reinterpret_cast
- auto, register, extern, volatile, mutable, export
- true, false, typeid

---

## 9. Complexidade e comportamento

### 9.1. Complexidade temporal

A tokenizacao e linear no tamanho da entrada na maior parte dos casos:

- O(n), onde n e o numero de caracteres.

O retrocesso de um caractere no estado Outro nao altera a ordem assintotica, pois cada caractere e reprocessado no maximo poucas vezes conforme fronteira de token.

### 9.2. Complexidade espacial

- O(n) para armazenar chars e tokens.
- O lexema corrente cresce conforme token atual.

---

## 10. Pontos de extensao

Sugestoes de evolucao sem quebrar arquitetura:

1. Incluir posicao linha/coluna em TokenInfo.
2. Padronizar erros lexicos com enum proprio.
3. Expandir literais numericos (hex, octal, expoente, sufixos).
4. Tratar preprocessing real de C/C++ (#include, #define) em camada separada.
5. Transformar tabela de reservadas em estrutura hash para manutencao.
6. Criar suite de testes unitarios por categoria de token.

---

## 11. Estrategia de teste recomendada

### 11.1. Testes por classe

- Identificador valido/invalido
- Inteiro e float
- String com escapes
- Char com escapes
- Operadores simples e compostos
- Comentarios linha e bloco
- Palavras reservadas vs identificadores proximos

### 11.2. Testes de fronteira

- Arquivo vazio
- Apenas espacos
- Comentario de bloco sem fechamento
- String sem fechamento
- Sequencias de delimitadores

### 11.3. Testes de regressao

Sempre que alterar o AFD, validar:

- lista de tokens emitidos
- lexema associado
- ausencia de perda de caractere em fronteiras (estado Outro)

---

## 12. Conclusao

A implementacao atual oferece uma base solida para analise lexico de estilo C/C++, com AFD explicito, modularizacao clara e ponto de extensao para evolucao em compilador completo. O estado Outro e a funcao voltar_caractere sao o mecanismo central para preservar delimitadores na fronteira entre tokens.
