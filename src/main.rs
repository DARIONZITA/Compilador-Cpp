use std::fs;

mod lexer;
mod token;
mod utils;
mod analise_sintatica;

fn main() {
    let conteudo = fs::read_to_string("src/files/main.cpp")
        .expect("não foi possível ler o ficheiro");

    let tokens = lexer::tokenizar(&conteudo);
    for token in &tokens {
        println!("{:?} => {:?} : {}", token.kind, token.lexema, token.linha);
    }
    analise_sintatica::analisar(tokens);
}
