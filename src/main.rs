mod token;

use anyhow::{Result, Context};
use crate::token::Expr;
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

    let expr = Expr::parse(expr)
        .expect("parse error");

    expr.compile();

    Ok(())
}