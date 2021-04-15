mod ast;
mod error;
mod scanner;
mod token;

fn run(source: String) -> anyhow::Result<()> {
    let scanner = scanner::Scanner::new(source);
    let tokens = scanner.scan_tokens()?;

    for token in &tokens {
        println!("{:?}", token);
    }

    Ok(())
}

fn run_file(path: &str) -> anyhow::Result<()> {
    let bytes = std::fs::read(path)?;
    run(std::str::from_utf8(&bytes)?.into())
}

fn run_prompt() -> anyhow::Result<()> {
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();
    use std::io::{BufRead, Write};
    loop {
        print!("> ");
        stdout.flush()?;
        let mut line = String::new();
        let mut reader = stdin.lock();
        if reader.read_line(&mut line)? == 0 {
            break;
        }
        if let Err(error) = run(line) {
            println!("{}", error);
        }
    }
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let args = std::env::args().collect::<Vec<_>>();
    if args.len() > 2 {
        println!("Usage: lox [script]");
        std::process::exit(64);
    } else if args.len() == 2 {
        run_file(&args[1])?;
    } else {
        run_prompt()?;
    }
    Ok(())
}
