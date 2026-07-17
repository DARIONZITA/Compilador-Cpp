use crate::scope::Symbol;
use crate::scope::SymbolType;
use crate::scope::SymbolCategory;
use crate::scope::Scope;
use crate::scope::BlockType;
use crate::ast::AstNode;
use crate::ast::AstKind;
use crate::scope::SemanticAnalyzer;
use crate::token::Token;

fn tamanho_do_tipo(tipo: &str) -> usize {
    match tipo {
        "int" => 4,
        "float" => 4,
        "double" => 8,
        "char" => 1,
        "bool" => 1,
        "string" => 8,
        _ => 4,
    }
}

fn tipo_para_symbol_type(tipo: &str) -> SymbolType {
    match tipo {
        "int" => SymbolType::Int("int"),
        "float" => SymbolType::Float("float"),
        "double" => SymbolType::Double("double"),
        "char" => SymbolType::Char("char"),
        "bool" => SymbolType::Bool("bool"),
        "string" => SymbolType::String("string"),
        _ => SymbolType::Int("int"),
    }
}

fn symbol_type_to_string(st: &SymbolType) -> String {
    match st {
        SymbolType::Int(s) | SymbolType::Float(s) | SymbolType::Double(s)
        | SymbolType::Char(s) | SymbolType::Bool(s) | SymbolType::String(s) => s.to_string(),
    }
}

fn lookup<'a>(analyzer: &'a SemanticAnalyzer, nome: &str) -> Option<&'a Symbol> {
    let mut idx = Some(analyzer.current_scope_idx);
    while let Some(i) = idx {
        if let Some(sym) = analyzer.scopes[i].symbols.get(nome) {
            return Some(sym);
        }
        idx = analyzer.scopes[i].parent_idx;
    }
    None
}

fn promover_tipo(a: &str, b: &str) -> Option<String> {
    if a == b {
        return Some(a.to_string());
    }
    match (a, b) {
        ("int", "float")  | ("float", "int")  => Some("float".to_string()),
        ("int", "double") | ("double", "int") => Some("double".to_string()),
        ("float", "double") | ("double", "float") => Some("double".to_string()),
        _ => None,
    }
}

fn semantic_analysis(node: &mut AstNode, analyzer: &mut SemanticAnalyzer) {
    match node.kind {
        AstKind::Block => {
            let parent_idx = analyzer.current_scope_idx;
            let parent_level = analyzer.scopes[parent_idx].scope_level;

            analyzer.scopes.push(Scope {
                symbols: std::collections::HashMap::new(),
                parent_idx: Some(parent_idx),
                scope_level: parent_level + 1,
                next_memory_offset: 0,
            });
            analyzer.current_scope_idx = analyzer.scopes.len() - 1;

            let pending: Vec<Symbol> = analyzer.pending_parameters.drain(..).collect();
            for param in pending {
                let size = param.size_in_bytes;
                let addr = analyzer.scopes[analyzer.current_scope_idx].next_memory_offset;
                let mut param_final = param;
                param_final.scope_level = analyzer.scopes[analyzer.current_scope_idx].scope_level;
                param_final.memory_address = addr;
                analyzer.scopes[analyzer.current_scope_idx]
                    .symbols.insert(param_final.name.clone(), param_final);
                analyzer.scopes[analyzer.current_scope_idx].next_memory_offset += size;
            }

            for child in node.children.iter_mut() {
                semantic_analysis(child, analyzer);
            }

            analyzer.current_scope_idx = parent_idx;
        }
        AstKind::VarDecl => {
            let mut tipo = String::new();
            let mut nome = String::new();
            for child in &node.children {
                match child.kind {
                    AstKind::Type => tipo = child.token.clone().unwrap_or_default(),
                    AstKind::Identifier => nome = child.token.clone().unwrap_or_default(),
                    _ => {}
                }
            }

            if analyzer.for_init_mode {
                let size = tamanho_do_tipo(&tipo);
                analyzer.pending_parameters.push(Symbol {
                    name: nome,
                    symbol_type: tipo_para_symbol_type(&tipo),
                    category: SymbolCategory::Variable,
                    scope_level: 0,
                    line_declared: node.line,
                    memory_address: 0,
                    size_in_bytes: size,
                    dimension: 0,
                    parameter_types: vec![],
                });
                node.inferred_type = tipo;
                return;
            }

            {
                let scope = &analyzer.scopes[analyzer.current_scope_idx];
                if scope.symbols.contains_key(&nome) {
                    let existente = &scope.symbols[&nome];
                    panic!("Variavel '{}' ja declarada na linha {}, redeclarada na linha {}", nome, existente.line_declared, node.line);
                }
            }

            let size = tamanho_do_tipo(&tipo);
            let addr = analyzer.scopes[analyzer.current_scope_idx].next_memory_offset;
            let scope_level = analyzer.scopes[analyzer.current_scope_idx].scope_level;

            analyzer.scopes[analyzer.current_scope_idx].symbols.insert(nome.clone(), Symbol {
                name: nome,
                symbol_type: tipo_para_symbol_type(&tipo),
                category: SymbolCategory::Variable,
                scope_level,
                line_declared: node.line,
                memory_address: addr,
                size_in_bytes: size,
                dimension: 0,
                parameter_types: vec![],
            });

            analyzer.scopes[analyzer.current_scope_idx].next_memory_offset += size;

            for child in node.children.iter_mut() {
                match child.kind {
                    AstKind::Type | AstKind::Identifier | AstKind::Modifier => {}
                    _ => semantic_analysis(child, analyzer),
                }
            }

            if let Some(init) = node.children.iter().find(|c|
                !matches!(c.kind, AstKind::Type | AstKind::Identifier | AstKind::Modifier)
            ) {
                let tipo_init = &init.inferred_type;
                if !tipo_init.is_empty() && promover_tipo(&tipo, tipo_init).is_none() {
                    panic!("Inicializacao incompativel na linha {}: '{}' = '{}'", node.line, tipo, tipo_init);
                }
            }

            node.inferred_type = tipo;
        }
        AstKind::ArrayDecl => {
            let mut tipo = String::new();
            let mut nome = String::new();
            let mut dimensoes = 0usize;
            let mut total_elementos = 1usize;

            for child in &node.children {
                match child.kind {
                    AstKind::Type => tipo = child.token.clone().unwrap_or_default(),
                    AstKind::Identifier => nome = child.token.clone().unwrap_or_default(),
                    _ => {}
                }
            }

            {
                let scope = &analyzer.scopes[analyzer.current_scope_idx];
                if scope.symbols.contains_key(&nome) {
                    let existente = &scope.symbols[&nome];
                    panic!("Array '{}' ja declarado na linha {}, redeclarado na linha {}", nome, existente.line_declared, node.line);
                }
            }

            for child in node.children.iter_mut() {
                if child.kind == AstKind::ArrayDimension {
                    dimensoes += 1;
                    semantic_analysis(child, analyzer);
                    if let Some(expr) = child.children.first() {
                        if expr.inferred_type != "int" {
                            panic!("Indice do array deve ser 'int', encontrou '{}' na linha {}", expr.inferred_type, node.line);
                        }
                        if expr.kind == AstKind::Literal {
                            if let Some(valor_str) = &expr.token {
                                if let Ok(valor_num) = valor_str.as_str().parse::<usize>() {
                                    total_elementos *= valor_num;
                                }
                            }
                        }
                    }
                }
            }

            let base_size = tamanho_do_tipo(&tipo);
            let size = base_size * total_elementos;
            let addr = analyzer.scopes[analyzer.current_scope_idx].next_memory_offset;
            let scope_level = analyzer.scopes[analyzer.current_scope_idx].scope_level;

            analyzer.scopes[analyzer.current_scope_idx].symbols.insert(nome.clone(), Symbol {
                name: nome,
                symbol_type: tipo_para_symbol_type(&tipo),
                category: SymbolCategory::Array,
                scope_level,
                line_declared: node.line,
                memory_address: addr,
                size_in_bytes: size,
                dimension: dimensoes,
                parameter_types: vec![],
            });

            analyzer.scopes[analyzer.current_scope_idx].next_memory_offset += size;

            for child in node.children.iter_mut() {
                match child.kind {
                    AstKind::Type | AstKind::Identifier | AstKind::Modifier | AstKind::ArrayDimension => {}
                    _ => semantic_analysis(child, analyzer),
                }
            }

            node.inferred_type = tipo;
        }
        AstKind::ArrayDimension => {
            for child in node.children.iter_mut() {
                semantic_analysis(child, analyzer);
            }
        }
        AstKind::Param => {
            let mut tipo = String::new();
            let mut nome = String::new();
            for child in &node.children {
                match child.kind {
                    AstKind::Type => tipo = child.token.clone().unwrap_or_default(),
                    AstKind::Identifier => nome = child.token.clone().unwrap_or_default(),
                    _ => {}
                }
            }

            let size = tamanho_do_tipo(&tipo);
            analyzer.pending_parameters.push(Symbol {
                name: nome,
                symbol_type: tipo_para_symbol_type(&tipo),
                category: SymbolCategory::Parameter,
                scope_level: 0,
                line_declared: node.line,
                memory_address: 0,
                size_in_bytes: size,
                dimension: 0,
                parameter_types: vec![],
            });
            node.inferred_type = tipo;
        }
        AstKind::ClassDecl => {
            let mut nome = String::new();
            for child in &node.children {
                if child.kind == AstKind::Identifier {
                    nome = child.token.clone().unwrap_or_default();
                    break;
                }
            }

            {
                let scope = &analyzer.scopes[analyzer.current_scope_idx];
                if scope.symbols.contains_key(&nome) {
                    let existente = &scope.symbols[&nome];
                    panic!("Classe '{}' ja declarada na linha {}, redeclarada na linha {}", nome, existente.line_declared, node.line);
                }
            }

            let scope_level = analyzer.scopes[analyzer.current_scope_idx].scope_level;
            analyzer.scopes[analyzer.current_scope_idx].symbols.insert(nome.clone(), Symbol {
                name: nome.clone(),
                symbol_type: SymbolType::Int(""),
                category: SymbolCategory::Class,
                scope_level,
                line_declared: node.line,
                memory_address: 0,
                size_in_bytes: 0,
                dimension: 0,
                parameter_types: vec![],
            });

            let previous = analyzer.block_type.clone();
            analyzer.block_type = BlockType::Class(nome);

            for child in node.children.iter_mut() {
                semantic_analysis(child, analyzer);
            }

            analyzer.block_type = previous;
        }
        AstKind::FunctionDecl => {
            let mut nome = String::new();
            let mut tipo = String::new();
            for child in &node.children {
                match child.kind {
                    AstKind::Type => tipo = child.token.clone().unwrap_or_default(),
                    AstKind::Identifier => nome = child.token.clone().unwrap_or_default(),
                    _ => {}
                }
            }

            {
                let scope = &analyzer.scopes[analyzer.current_scope_idx];
                if scope.symbols.contains_key(&nome) {
                    let existente = &scope.symbols[&nome];
                    panic!("Funcao '{}' ja declarada na linha {}, redeclarada na linha {}", nome, existente.line_declared, node.line);
                }
            }

            let param_types: Vec<String> = node.children.iter()
                .filter(|c| c.kind == AstKind::ParamList)
                .flat_map(|c| c.children.iter())
                .filter(|p| p.kind == AstKind::Param)
                .filter_map(|p| {
                    p.children.iter()
                        .find(|c| c.kind == AstKind::Type)
                        .and_then(|t| t.token.clone())
                })
                .collect();

            let scope_level = analyzer.scopes[analyzer.current_scope_idx].scope_level;
            analyzer.scopes[analyzer.current_scope_idx].symbols.insert(nome.clone(), Symbol {
                name: nome.clone(),
                symbol_type: tipo_para_symbol_type(&tipo),
                category: SymbolCategory::Function,
                scope_level,
                line_declared: node.line,
                memory_address: 0,
                size_in_bytes: 0,
                dimension: 0,
                parameter_types: param_types,
            });

            let previous = analyzer.block_type.clone();
            analyzer.block_type = BlockType::Function(nome.clone());

            let previous_ret = analyzer.current_function_return_type.clone();
            analyzer.current_function_return_type = Some(tipo.clone());

            for child in node.children.iter_mut() {
                semantic_analysis(child, analyzer);
            }

            analyzer.current_function_return_type = previous_ret;
            analyzer.block_type = previous;
            node.inferred_type = tipo;
        }
        AstKind::ConstructorDecl => {
            let mut nome = String::new();
            for child in &node.children {
                if child.kind == AstKind::Identifier {
                    nome = child.token.clone().unwrap_or_default();
                    break;
                }
            }

            match &analyzer.block_type {
                BlockType::Class(class_name) => {
                    if nome != *class_name {
                        panic!("Construtor '{}' na linha {} nao corresponde ao nome da classe '{}'",
                            nome, node.line, class_name);
                    }
                }
                _ => {
                    panic!("Construtor '{}' declarado fora de uma classe na linha {}",
                        nome, node.line);
                }
            }

            {
                let scope = &analyzer.scopes[analyzer.current_scope_idx];
                if scope.symbols.contains_key(&nome) {
                    let existente = &scope.symbols[&nome];
                    panic!("Construtor '{}' ja declarado na linha {}, redeclarado na linha {}", nome, existente.line_declared, node.line);
                }
            }

            let param_types: Vec<String> = node.children.iter()
                .filter(|c| c.kind == AstKind::ParamList)
                .flat_map(|c| c.children.iter())
                .filter(|p| p.kind == AstKind::Param)
                .filter_map(|p| {
                    p.children.iter()
                        .find(|c| c.kind == AstKind::Type)
                        .and_then(|t| t.token.clone())
                })
                .collect();

            let scope_level = analyzer.scopes[analyzer.current_scope_idx].scope_level;
            analyzer.scopes[analyzer.current_scope_idx].symbols.insert(nome.clone(), Symbol {
                name: nome.clone(),
                symbol_type: SymbolType::Int(""),
                category: SymbolCategory::Function,
                scope_level,
                line_declared: node.line,
                memory_address: 0,
                size_in_bytes: 0,
                dimension: 0,
                parameter_types: param_types,
            });

            let previous = analyzer.block_type.clone();
            analyzer.block_type = BlockType::Constructor(nome);

            let previous_ret = analyzer.current_function_return_type.clone();
            analyzer.current_function_return_type = Some("void".to_string());

            for child in node.children.iter_mut() {
                semantic_analysis(child, analyzer);
            }

            analyzer.current_function_return_type = previous_ret;
            analyzer.block_type = previous;
        }
        AstKind::For => {
            let previous = analyzer.block_type.clone();
            analyzer.block_type = BlockType::For;

            let parent_idx = analyzer.current_scope_idx;
            let parent_level = analyzer.scopes[parent_idx].scope_level;

            analyzer.for_init_mode = true;
            if let Some(init) = node.children.get_mut(0) {
                semantic_analysis(init, analyzer);
            }
            analyzer.for_init_mode = false;

            analyzer.scopes.push(Scope {
                symbols: std::collections::HashMap::new(),
                parent_idx: Some(parent_idx),
                scope_level: parent_level + 1,
                next_memory_offset: 0,
            });
            analyzer.current_scope_idx = analyzer.scopes.len() - 1;

            let pending: Vec<Symbol> = analyzer.pending_parameters.drain(..).collect();
            for mut sym in pending {
                let size = sym.size_in_bytes;
                let addr = analyzer.scopes[analyzer.current_scope_idx].next_memory_offset;
                sym.scope_level = analyzer.scopes[analyzer.current_scope_idx].scope_level;
                sym.memory_address = addr;
                analyzer.scopes[analyzer.current_scope_idx]
                    .symbols.insert(sym.name.clone(), sym);
                analyzer.scopes[analyzer.current_scope_idx].next_memory_offset += size;
            }

            for child in node.children.iter_mut().skip(1) {
                semantic_analysis(child, analyzer);
            }

            if node.children.len() >= 2 {
                let cond_idx = node.children.len() - 2;
                let cond_type = &node.children[cond_idx].inferred_type;
                if !cond_type.is_empty() && !matches!(cond_type.as_str(), "bool" | "int") {
                    panic!("Condicao do for deve ser 'bool' ou 'int', encontrou '{}' na linha {}",
                        cond_type, node.children[cond_idx].line);
                }
            }

            analyzer.current_scope_idx = parent_idx;
            analyzer.block_type = previous;
        }
        AstKind::While => {
            let previous = analyzer.block_type.clone();
            analyzer.block_type = BlockType::While;
            for child in node.children.iter_mut() {
                semantic_analysis(child, analyzer);
            }
            let cond_type = &node.children[0].inferred_type;
            if !cond_type.is_empty() && !matches!(cond_type.as_str(), "bool" | "int") {
                panic!("Condicao do while deve ser 'bool' ou 'int', encontrou '{}' na linha {}",
                    cond_type, node.children[0].line);
            }
            analyzer.block_type = previous;
        }
        AstKind::DoWhile => {
            let previous = analyzer.block_type.clone();
            analyzer.block_type = BlockType::DoWhile;
            for child in node.children.iter_mut() {
                semantic_analysis(child, analyzer);
            }
            let cond_type = &node.children[1].inferred_type;
            if !cond_type.is_empty() && !matches!(cond_type.as_str(), "bool" | "int") {
                panic!("Condicao do do-while deve ser 'bool' ou 'int', encontrou '{}' na linha {}",
                    cond_type, node.children[1].line);
            }
            analyzer.block_type = previous;
        }
        AstKind::If => {
            let previous = analyzer.block_type.clone();
            analyzer.block_type = BlockType::If;
            for child in node.children.iter_mut() {
                semantic_analysis(child, analyzer);
            }
            let cond_type = &node.children[0].inferred_type;
            if !cond_type.is_empty() && !matches!(cond_type.as_str(), "bool" | "int") {
                panic!("Condicao do if deve ser 'bool' ou 'int', encontrou '{}' na linha {}",
                    cond_type, node.children[0].line);
            }
            analyzer.block_type = previous;
        }
        AstKind::Switch => {
            let previous = analyzer.block_type.clone();
            analyzer.block_type = BlockType::Switch;
            for child in node.children.iter_mut() {
                semantic_analysis(child, analyzer);
            }
            analyzer.block_type = previous;
        }
        AstKind::Break => {
            match &analyzer.block_type {
                BlockType::For | BlockType::While | BlockType::DoWhile | BlockType::Switch => {}
                _ => panic!("Break na linha {} fora de loop ou switch", node.line),
            }
        }
        AstKind::Continue => {
            match &analyzer.block_type {
                BlockType::For | BlockType::While | BlockType::DoWhile => {}
                _ => panic!("Continue na linha {} fora de loop", node.line),
            }
        }
        AstKind::Identifier => {
            let nome = node.token.clone().unwrap_or_default();
            match lookup(analyzer, &nome) {
                Some(sym) => {
                    node.inferred_type = symbol_type_to_string(&sym.symbol_type);
                }
                None => {
                    panic!("Variavel '{}' nao declarada na linha {}", nome, node.line);
                }
            }
        }
        AstKind::Literal => {
            match node.original_token {
                Some(Token::TrueLiteral) | Some(Token::FalseLiteral) => {
                    node.inferred_type = "bool".to_string();
                }
                Some(Token::String) => {
                    node.inferred_type = "string".to_string();
                }
                Some(Token::Char) => {
                    node.inferred_type = "char".to_string();
                }
                Some(Token::Float) => {
                    node.inferred_type = "double".to_string();
                }
                Some(Token::Inteiro) => {
                    node.inferred_type = "int".to_string();
                }
                _ => {
                    node.inferred_type = "int".to_string();
                }
            }
        }
        AstKind::Assign => {
            for child in node.children.iter_mut() {
                semantic_analysis(child, analyzer);
            }
            let lvalue_ok = matches!(
                node.children[0].kind,
                AstKind::Identifier | AstKind::Index | AstKind::MemberAccess | AstKind::PtrAccess
            );
            if !lvalue_ok {
                panic!("Lado esquerdo da atribuicao na linha {} nao e atribuivel", node.line);
            }
            let tipo_esq = node.children[0].inferred_type.clone();
            let tipo_dir = node.children[1].inferred_type.clone();
            if node.children[0].kind == AstKind::Identifier && tipo_esq.is_empty() {
                let nome = node.children[0].token.clone().unwrap_or_default();
                panic!("Variavel '{}' nao declarada na linha {}", nome, node.line);
            }
            if !tipo_esq.is_empty() && !tipo_dir.is_empty() {
                if promover_tipo(&tipo_esq, &tipo_dir).is_none() {
                    panic!("Atribuicao incompativel na linha {}: '{}' = '{}'", node.line, tipo_esq, tipo_dir);
                }
            }
            node.inferred_type = tipo_esq;
        }
        AstKind::BinaryOp => {
            for child in node.children.iter_mut() {
                semantic_analysis(child, analyzer);
            }
            let tipo_esq = node.children[0].inferred_type.clone();
            let tipo_dir = node.children[1].inferred_type.clone();
            let op = node.token.clone().unwrap_or_default();
            if matches!(op.as_str(), "==" | "!=" | "<" | ">" | "<=" | ">=" | "&&" | "||") {
                if promover_tipo(&tipo_esq, &tipo_dir).is_none() {
                    panic!("Operador '{}' incompativel com '{}' e '{}' na linha {}",
                        op, tipo_esq, tipo_dir, node.line);
                }
                node.inferred_type = "bool".to_string();
            } else {
                match promover_tipo(&tipo_esq, &tipo_dir) {
                    Some(tipo_resultado) => { node.inferred_type = tipo_resultado; }
                    None => {
                        panic!("Operador '{}' incompativel com '{}' e '{}' na linha {}",
                            op, tipo_esq, tipo_dir, node.line);
                    }
                }
            }
        }
        AstKind::UnaryOp => {
            for child in node.children.iter_mut() {
                semantic_analysis(child, analyzer);
            }
            let tipo = node.children[0].inferred_type.clone();
            let op = node.token.clone().unwrap_or_default();
            match op.as_str() {
                "-" | "++" | "--" | "post++" | "post--" => {
                    node.inferred_type = tipo;
                }
                "!" => {
                    node.inferred_type = "bool".to_string();
                }
                "&" | "*" => {
                    node.inferred_type = tipo;
                }
                _ => {
                    node.inferred_type = tipo;
                }
            }
        }
        AstKind::Call => {
            for child in node.children.iter_mut() {
                semantic_analysis(child, analyzer);
            }
            let nome = if node.children[0].kind == AstKind::Identifier {
                node.children[0].token.clone().unwrap_or_default()
            } else {
                panic!("Chamada de metodo na linha {} ainda nao suportada", node.line);
            };
            let sym = match lookup(analyzer, &nome) {
                Some(s) => s,
                None => {
                    panic!("Funcao '{}' nao declarada na linha {}", nome, node.line);
                }
            };
            if sym.category != SymbolCategory::Function {
                panic!("'{}' na linha {} nao e uma funcao", nome, node.line);
            }
            let args_enviados: Vec<&str> = node.children[1..]
                .iter()
                .map(|c| c.inferred_type.as_str())
                .collect();
            let params_esperados: Vec<&str> = sym.parameter_types.iter()
                .map(|s: &String| s.as_str())
                .collect();
            if args_enviados.len() != params_esperados.len() {
                panic!("Funcao '{}' na linha {} espera {} argumentos, mas {} foram passados",
                    nome, node.line, params_esperados.len(), args_enviados.len());
            }
            for (i, (enviado, esperado)) in args_enviados.iter().zip(params_esperados.iter()).enumerate() {
                if promover_tipo(enviado, esperado).is_none() {
                    panic!("Argumento {} da funcao '{}' na linha {}: esperado '{}', obtido '{}'",
                        i + 1, nome, node.line, esperado, enviado);
                }
            }
            node.inferred_type = symbol_type_to_string(&sym.symbol_type);
        }
        AstKind::Inherit | AstKind::AccessSection | AstKind::ParamList
            | AstKind::Case | AstKind::Default => {
            for child in node.children.iter_mut() {
                semantic_analysis(child, analyzer);
            }
        }
        AstKind::Return => {
            for child in node.children.iter_mut() {
                semantic_analysis(child, analyzer);
            }
            let func_type = match &analyzer.current_function_return_type {
                Some(t) => t.clone(),
                None => panic!("Return na linha {} fora de uma funcao", node.line),
            };
            if func_type == "void" {
                if !node.children.is_empty() {
                    panic!("Funcao retorna void, mas return na linha {} retorna valor", node.line);
                }
                return;
            }
            if node.children.is_empty() {
                panic!("Funcao espera retorno '{}', mas return na linha {} nao retorna valor", func_type, node.line);
            }
            let ret_type = node.children[0].inferred_type.clone();
            if !ret_type.is_empty() && promover_tipo(&func_type, &ret_type).is_none() {
                panic!("Tipo de retorno incompativel na linha {}: funcao retorna '{}', mas expressao e '{}'",
                    node.line, func_type, ret_type);
            }
        }
        _ => {
            for child in node.children.iter_mut() {
                semantic_analysis(child, analyzer);
            }
        }
    }
}

pub fn decorar_ast(mut ast: AstNode) -> (AstNode, Vec<Scope>) {
    let mut analyzer = SemanticAnalyzer {
        scopes: vec![Scope {
            symbols: std::collections::HashMap::new(),
            parent_idx: None,
            scope_level: 0,
            next_memory_offset: 0,
        }],
        current_scope_idx: 0,
        pending_parameters: vec![],
        block_type: BlockType::None,
        current_function_return_type: None,
        for_init_mode: false,
    };
    semantic_analysis(&mut ast, &mut analyzer);
    (ast, analyzer.scopes)
}
