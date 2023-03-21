pub mod statement;
pub mod errors;

pub use crate::parser::errors::*;
pub use crate::parser::curry_pest::{Pair, Pairs, PairsHelper, Rule};
pub use crate::parser::ast::statement::*;


use std::convert::TryInto;
use anyhow::*;
use crate::parser::curry_pest::*;

pub fn parse_to_ast(input: &str) -> Result<Statement,Error> {
    let source_pair = CurryParser::parse(Rule::source, input)
        .context("parsing cst")?
        .unique_pair()?;

    source_pair.try_into().context("build source")
}
