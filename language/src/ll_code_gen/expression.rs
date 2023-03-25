use anyhow::*;
use inkwell::values::{BasicMetadataValueEnum, BasicValue};
use pest::iterators::Pair;
use crate::ll_code_gen::function_generator::FunctionGenerator;
use crate::parser::curry_pest::{Rule, PairHelper};

pub fn generate_expression<'gen>(pair: Pair<Rule>, scope: &FunctionGenerator<'gen,'_,'_>) -> anyhow::Result<BasicMetadataValueEnum<'gen>> {
    match pair.as_rule() {
        Rule::value => {
            let value_expr = pair.unique_inner()?;
            match value_expr.as_rule() {
                Rule::string => {
                    let string_expr : StringConstant = value_expr.into();
                    string_expr.generate(scope)
                },
                it => todo!("build_fn_arg value - {}", it)
            }
        },
        it => todo!("build_fn_arg {}", it)
    }
}

#[derive(derive_more::From)]
struct VariableValue<'i> {
    pair: Pair<'i,Rule>
}
impl <'i> VariableValue<'i> {
    fn resolve_variable<'gen>(&self, scope: &FunctionGenerator<'gen,'_,'_>) -> Result<BasicMetadataValueEnum<'gen>> {
        let variable_name = self.pair.as_str();
        let symbols = scope.symbols.borrow();
        let variable = symbols.get(variable_name)
            .with_context(|| format!("usage of unknown variable '{}'", variable_name))?;
        Ok(BasicMetadataValueEnum::from(variable.as_basic_value_enum()))
    }
}

#[derive(derive_more::From)]
struct StringConstant<'i> {
    pair: Pair<'i,Rule>
}
impl <'i> StringConstant<'i> {
    fn generate<'gen>(mut self, scope: &FunctionGenerator<'gen,'_,'_>) -> Result<BasicMetadataValueEnum<'gen>> {
        let pair = self.pair.unique_inner().context("access string_value")?;
        let value = pair.as_str();
        // let string_ref = scope.parent.parent.context.const_string(value.as_bytes(),true);
        let string_ref = scope.builder.build_global_string_ptr(value, "string");
        Ok(BasicMetadataValueEnum::from(string_ref.as_basic_value_enum()))
    }
}