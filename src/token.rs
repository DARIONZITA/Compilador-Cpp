#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Token {
    Identificador,
    Inteiro,
    Float,
    String,
    Char,
    Adicao,
    Incremento,
    MaisIgual,
    Subtracao,
    Decremento,
    MenosIgual,
    Seta,
    Multiplicacao,
    VezesIgual,
    Divisao,
    DivIgual,
    Modulo,
    ModIgual,
    Atribuicao,
    Igual,
    Menor,
    Maior,
    MenorIgual,
    MaiorIgual,
    Not,
    Diferente,
    And,
    Or,
    BitAnd,
    BitOr,
    BitXor,
    BitNot,
    ShiftEsq,
    ShiftDir,
    PontoMembro,
    Escopo,
    DoisPontos,
    Interrogacao,
    ComentarioLinha,
    ComentarioBloco,
    PtVirgula,
    Virgula,
    AbreParen,
    FechaParen,
    AbreChave,
    FechaChave,
    AbreColch,
    FechaColch,
    If,
    Else,
    While,
    For,
    Return,
    Int,
    FloatType,
    CharType,
    StringType,
    Void,
    Class,
    Struct,
    Enum,
    Const,
    Static,
    Public,
    Private,
    Protected,
    Virtual,
    Override,
    Abstract,
    Template,
    Typedef,
    Namespace,
    Using,
    Include,
    Long,
    Short,
    Signed,
    Unsigned,

    // --- Controle de fluxo ---
    Do,
    Switch,
    Case,
    Break,
    Continue,
    Goto,
    Default,

    // --- Tipos primitivos ---
    Bool,
    Double,  // para diferenciar de FloatType
    // --- Memória e ponteiros ---
    New,
    Delete,
    Sizeof,

    // --- Operadores lógicos como palavras ---
    AndEq,     // &=
    OrEq,      // |=
    XorEq,     // ^=
    Xor,       // ^
    Bitand,    // &
    Bitor,     // |
    Compl,     // ~
    NotEq,     // !=

    // --- OOP / tipos especiais ---
    This,
    Inline,
    Explicit,
    Friend,
    Operator,
    Typename,

    // --- Exceções ---
    Try,
    Catch,
    Throw,

    // --- Casting ---
    StaticCast,
    DynamicCast,
    ConstCast,
    ReinterpretCast,

    // --- Outros ---
    Auto,
    Register,
    Extern,
    Volatile,
    Mutable,
    Export,
    TrueLiteral,   // true
    FalseLiteral,  // false
    Typeid,
    Cin,
    Cout
}

#[derive(Debug)]
pub struct TokenInfo {
    pub kind: Token,
    pub lexema: String,
    pub linha: usize,
}

#[derive(Debug, Clone, Copy)]
pub enum Estado {
    Inicio,
    Identificador,
    Inteiro,
    FloatPonto,
    FloatDigitos,
    StringAberta,
    StringEscape,
    CharAberto,
    CharEscape,
    CharConteudo,
    OpMais,
    OpMenos,
    OpVezes,
    OpDiv,
    OpMod,
    OpIgual,
    OpMenor,
    OpMaior,
    OpExclamacao,
    OpE,
    OpOu,
    OpDoisPontos,
    IncludeAngle,
    ComentarioLinha,
    ComentarioBloco,
    ComentarioBlocoAst,
    Outro,
}

