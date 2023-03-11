use pest_derive::Parser;
pub use pest::Parser;

#[cfg(test)]
mod tests;

#[derive(Parser)]
#[grammar = "parser/grammar.pest"]
pub struct CurryParser;

