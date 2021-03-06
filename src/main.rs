use anyhow::{anyhow, Context, Result};
use std::iter::Peekable;
use std::rc::Rc;
use std::cell::RefCell;

use clap::{
    crate_authors, crate_description, crate_name, crate_version,
    Arg, App,
};

#[derive(Debug)]
struct TokenList {
    head: TokenLink,
    tail: TokenLink,
}

type TokenLink = Option<Rc<RefCell<Token>>>;

#[derive(Debug)]
struct Token {
    next: TokenLink,
    content: TokenContent,
}

#[derive(Debug)]
enum TokenContent {
    Symbol(Symbol),
    Number(i32),
}

#[derive(Debug, Clone, Copy)]
enum Symbol {
    Add,
    Sub,
}

impl Symbol {
    fn parse(c: char) -> Option<Symbol> {
        match c {
            '+' => Some(Symbol::Add),
            '-' => Some(Symbol::Sub),
            _ => None
        }
    }
}

impl TokenList {
    fn new() -> Self {
        Self { head: None, tail: None }
    }

    fn push(&mut self, content: TokenContent) {
        let new_token = Token::new(content);
        match self.tail.take() {
            Some(tail) => {
                tail.borrow_mut().next = Some(new_token.clone());
                self.tail = Some(new_token);
            },
            None => {
                self.head = Some(new_token.clone());
                self.tail = Some(new_token);
            }
        }
    }
}


impl Token {
    fn new(content: TokenContent) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Token { next: None, content }))
    }
}

impl TokenList {
    fn parse(expr: &str) -> Result<TokenList> {
        let mut it = expr.chars().peekable();
        let mut list = TokenList::new();

        while let Some(&c) = it.peek() {
            match c {
                c if c.is_whitespace() => { it.next(); },
                c if c.is_ascii_digit() => {
                    let number = get_number(&mut it)?;
                    list.push(TokenContent::Number(number));
                },
                c if Symbol::parse(c).is_some() => {
                    let symbol = Symbol::parse(c).unwrap();
                    list.push(TokenContent::Symbol(symbol));
                    it.next();
                }
                _ => {
                    return Err(anyhow!("unexpected symbol"));
                }
            }
        }

        Ok(list)
    }

    fn compile(&self) -> Result<()> {
        println!(".intel_syntax noprefix");
        println!(".global main");
        println!("main:");

        let head = self.head.as_ref()
            .with_context(|| "parse result is null")?;

        if let TokenContent::Number(number) = head.borrow().content {
            println!("    mov rax, {}", number);
        } else {
            return Err(anyhow!("first token is not number"));
        }

        let mut prev_symbol: Option<Symbol> = None;
        let mut token_link = head.borrow().next.clone();
        while let Some(token) = token_link {
            match &token.borrow().content {
                TokenContent::Number(number) => {
                    match prev_symbol {
                        Some(Symbol::Add) => println!("    add rax, {}", number),
                        Some(Symbol::Sub) => println!("    sub rax, {}", number),
                        None => return Err(anyhow!("unexpected hoge")),
                    }
                    prev_symbol = None;
                },
                TokenContent::Symbol(symbol) => {
                    prev_symbol = Some(*symbol);
                },
            }
            token_link = token.borrow().next.clone();
        }

        println!("    ret");

        Ok(())
    }
}

fn get_number<T: Iterator<Item=char>>(it: &mut Peekable<T>) -> Result<i32> {
    let mut number = 0;

    if !it.peek().with_context(|| "number is expected")?.is_ascii_digit() {
        return Err(anyhow!("number is expected"));
    }

    while let Some(Ok(digit)) = it.peek().map(|c| c.to_string().parse::<i32>()) {
        number = number * 10 + digit;
        it.next();
    }
    Ok(number)
}

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

    let token_list = TokenList::parse(expr)?;
    token_list.compile()?;

    Ok(())
}