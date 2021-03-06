use anyhow::{anyhow, Context, Result};
use std::iter::Peekable;

use clap::{
    crate_authors, crate_description, crate_name, crate_version,
    Arg, App,
};

fn main() -> Result<()> {
    let matches = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .arg(Arg::with_name("expr")
            .required(true))
        .get_matches();

    let expr = matches.value_of("expr")
        .with_context(|| "not found")?;

    println!(".intel_syntax noprefix");
    println!(".global main");
    println!("main:");

    let mut it = expr.chars().peekable();
    let number = get_number(&mut it)?;
    println!("    mov rax, {}", number);

    while let Some(&c) = it.peek() {
        match c {
            '+' => {
                it.next();
                let number = get_number(&mut it)?;
                println!("    add rax, {}", number);
            },
            '-' => {
                it.next();
                let number = get_number(&mut it)?;
                println!("    sub rax, {}", number);
            },
            _ => {
                return Err(anyhow!("unexpected token"));
            }
        }
    }

    println!("    ret");

    Ok(())
}

fn get_number<T: Iterator<Item=char>>(it: &mut Peekable<T>) -> Result<i32> {
    let mut number = 0;

    if !it.peek().with_context(|| "number is expected")?.is_digit(10) {
        return Err(anyhow!("number is expected"));
    }

    while let Some(Ok(digit)) = it.peek().map(|c| c.to_string().parse::<i32>()) {
        number = number * 10 + digit;
        it.next();
    }
    Ok(number)
}
