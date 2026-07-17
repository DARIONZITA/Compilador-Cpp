use std::fs;
use std::path::Path;
use std::panic;
use std::cell::RefCell;
use std::env;

mod ast;
mod lexer;
mod token;
mod utils;
mod analise_sintatica;
mod analise_semantica;
mod scope;

thread_local! {
    static ULTIMO_ERRO: RefCell<Option<String>> = RefCell::new(None);
}

fn formatar_tabela_simbolos(scopes: &[scope::Scope]) -> String {
    let mut resultado = String::new();
    for (i, scp) in scopes.iter().enumerate() {
        if scp.symbols.is_empty() {
            continue;
        }
        resultado.push_str(&format!("  Escopo {} (nivel {}):\n", i, scp.scope_level));
        for (nome, sym) in &scp.symbols {
            let tipo = format!("{:?}", sym.symbol_type);
            let cat = format!("{:?}", sym.category);
            resultado.push_str(&format!(
                "    {} | tipo: {} | cat: {} | linha: {} | addr: {} | bytes: {}\n",
                nome, tipo, cat, sym.line_declared, sym.memory_address, sym.size_in_bytes
            ));
            if !sym.parameter_types.is_empty() {
                resultado.push_str(&format!("      params: {:?}\n", sym.parameter_types));
            }
        }
    }
    resultado
}

fn processar_ficheiro(caminho: &Path) -> String {
    let nome = caminho.file_name().unwrap().to_string_lossy().to_string();
    let conteudo = match fs::read_to_string(caminho) {
        Ok(c) => c,
        Err(e) => return format!("--- {} ---\nERRO ao ler ficheiro: {}\n\n", nome, e),
    };

    let tokens = lexer::tokenizar(&conteudo);
    let ast = analise_sintatica::analisar(tokens);

    ULTIMO_ERRO.with(|e| *e.borrow_mut() = None);

    let old_hook = panic::take_hook();
    panic::set_hook(Box::new(|info| {
        let msg = if let Some(s) = info.payload().downcast_ref::<String>() {
            s.clone()
        } else if let Some(s) = info.payload().downcast_ref::<&str>() {
            s.to_string()
        } else {
            "Erro desconhecido".to_string()
        };
        ULTIMO_ERRO.with(|e| *e.borrow_mut() = Some(msg));
    }));

    let resultado = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        analise_semantica::decorar_ast(ast)
    }));

    panic::set_hook(old_hook);

    let mut saida = format!("--- {} ---\n", nome);

    match resultado {
        Ok((ast_decorado, scopes)) => {
            saida.push_str("Resultado: SUCESSO\n\n");
            saida.push_str("AST Decorada (com inferred_type):\n");
            saida.push_str(&ast::format_ast(&ast_decorado));
            saida.push_str("\nTabela de Simbolos:\n");
            saida.push_str(&formatar_tabela_simbolos(&scopes));
            saida.push_str("\n");
        }
        Err(_) => {
            let msg = ULTIMO_ERRO.with(|e| e.borrow().clone().unwrap_or_else(|| "Erro desconhecido".to_string()));
            saida.push_str("Resultado: ERRO SEMANTICO\n");
            saida.push_str(&msg);
            saida.push_str("\n\n");
        }
    }
    saida
}

fn processar_ficheiro_erro(caminho: &Path) -> Option<String> {
    let nome = caminho.file_name().unwrap().to_string_lossy().to_string();
    let conteudo = match fs::read_to_string(caminho) {
        Ok(c) => c,
        Err(e) => return Some(format!("{}: ERRO ao ler ficheiro: {}", nome, e)),
    };

    let tokens = lexer::tokenizar(&conteudo);
    let ast = analise_sintatica::analisar(tokens);

    ULTIMO_ERRO.with(|e| *e.borrow_mut() = None);

    let old_hook = panic::take_hook();
    panic::set_hook(Box::new(|info| {
        let msg = if let Some(s) = info.payload().downcast_ref::<String>() {
            s.clone()
        } else if let Some(s) = info.payload().downcast_ref::<&str>() {
            s.to_string()
        } else {
            "Erro desconhecido".to_string()
        };
        ULTIMO_ERRO.with(|e| *e.borrow_mut() = Some(msg));
    }));

    let resultado = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        analise_semantica::decorar_ast(ast)
    }));

    panic::set_hook(old_hook);

    match resultado {
        Ok(_) => None,
        Err(_) => {
            let msg = ULTIMO_ERRO.with(|e| e.borrow().clone().unwrap_or_else(|| "Erro desconhecido".to_string()));
            Some(format!("{}: {}", nome, msg))
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        let caminho = Path::new(&args[1]);
        if !caminho.exists() {
            eprintln!("Ficheiro '{}' nao encontrado", args[1]);
            std::process::exit(1);
        }
        match processar_ficheiro_erro(caminho) {
            Some(erro) => { eprintln!("{}", erro); std::process::exit(1); }
            None => {}
        }
        return;
    }
    let dir = Path::new("src/files");
    let mut relatorio = String::from("=== RELATORIO DE ANALISE SEMANTICA ===\n\n");

    let mut ficheiros: Vec<_> = fs::read_dir(dir)
        .expect("Nao foi possivel ler src/files/")
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path().extension()
                .map(|ext| ext == "cpp")
                .unwrap_or(false)
        })
        .collect();

    ficheiros.sort_by_key(|e| e.file_name());

    for entry in &ficheiros {
        relatorio.push_str(&processar_ficheiro(&entry.path()));
    }

    fs::write("relatorio.txt", &relatorio).expect("Nao foi possivel escrever relatorio.txt");
}
