use pest_derive::Parser;
pub use pest::Parser;

mod pest_utils;
mod ast;

#[cfg(test)]
mod tests;

#[derive(Parser)]
#[grammar = "parser/grammar.pest"]
pub struct CurryParser;

