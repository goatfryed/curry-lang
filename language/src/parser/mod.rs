use pest::Parser;
use pest_derive::Parser;

#[cfg(test)]
mod tests;

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct CurryParser;

