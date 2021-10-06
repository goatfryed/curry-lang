use pest_derive::Parser;

#[cfg(test)]
mod tests;

#[derive(Parser)]
#[grammar = "parser/grammar.pest"]
pub struct CurryParser;

