pub mod statement;
pub mod errors;

pub use crate::parser::errors::*;
pub use crate::parser::curry_pest::{Pair, Pairs, PairsHelper, Rule};
pub use crate::parser::ast::statement::*;


use std::convert::TryInto;
use anyhow::*;
use crate::parser::curry_pest::*;

pub fn parse_to_ast(input: &str) -> Result<Vec<Statement>,Error> {
    let source_pair: Pair<Rule> = CurryParser::parse(Rule::source, input)
        .context("parsing cst")?
        .unique_pair()?;

    source_pair.into_inner()
        .filter(|pair| pair.as_rule() != Rule::EOI)
        .map(|pair| -> Result<Statement,Error> {
            pair.try_into().context("couldn't parse stmt")
        })
        .collect()
}