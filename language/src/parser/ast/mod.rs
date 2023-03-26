pub mod statement;
pub mod errors;

pub use crate::parser::errors::*;
pub use crate::parser::curry_pest::*;
pub use crate::parser::ast::statement::*;

use anyhow::*;

pub fn parse_to_ast(input: &str) -> Result<Pair<Rule>,Error> {
    CurryParser::parse(Rule::source, input)
        .context("parsing cst")?
        .unique_pair()?
        .unique_inner()
}