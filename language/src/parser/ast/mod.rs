pub mod statement;
pub mod errors;

pub use crate::parser::curry_pest::*;
pub use crate::parser::errors::*;
pub use anyhow::{Error,Context};

use std::convert::TryInto;
use crate::parser::ast::statement::Statement;

#[allow(dead_code)]
pub fn parse_to_ast(input: &str) -> Result<Statement,Error> {
    let source_pair = CurryParser::parse(Rule::source, input)
        .context("parsing cst")?
        .unique_pair()?;

    source_pair.try_into().context("build source")
}
