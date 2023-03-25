use anyhow::*;
use inkwell::values::{BasicMetadataValueEnum, BasicValue, IntValue, PointerValue};
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
        _ => {
            let variable_expr: VariableValue = pair.into();
            variable_expr.resolve_variable(scope)
        }
    }
}

#[derive(derive_more::From)]
struct VariableValue<'i> {
    pair: Pair<'i,Rule>
}
impl <'i> VariableValue<'i> {
    fn resolve_variable<'gen>(&self, scope: &FunctionGenerator<'gen,'_,'_>) -> Result<BasicMetadataValueEnum<'gen>> {
        let context = scope.parent.parent.context;
        let variable_name = self.pair.as_str();
        let symbols = scope.symbols.borrow();
        let string_pointer = *symbols.get(variable_name)
            .with_context(|| format!("usage of unknown variable '{}'", variable_name))?;
        let i8_type = context.i8_type();
        let indices = [i8_type.const_zero(), i8_type.const_zero()];
        unsafe {
            let pointer_value = scope.builder.build_in_bounds_gep(string_pointer, &indices, "tmp");
            Ok(BasicMetadataValueEnum::from(pointer_value.as_basic_value_enum()))
        }
    }
}

#[derive(derive_more::From)]
struct StringConstant<'i> {
    pair: Pair<'i,Rule>
}
impl <'i> StringConstant<'i> {
    fn generate<'gen>(self, scope: &FunctionGenerator<'gen,'_,'_>) -> Result<BasicMetadataValueEnum<'gen>> {
        let pair = self.pair.unique_inner().context("access string_value")?;
        let raw_string = pair.as_str();
        let value = Self::decode_user_string(raw_string);
        let context = scope.parent.parent.context;
        let string_value = context.const_string(value.as_bytes(), true);
        let string_pointer = scope.builder.build_alloca(string_value.get_type(), "str_val");
        scope.builder.build_store(string_pointer, string_value);
        let i8_type = context.i8_type();
        let indices = [i8_type.const_zero(), i8_type.const_zero()];
        unsafe {
            let pointer_value = scope.builder.build_in_bounds_gep(string_pointer, &indices, "tmp");
            Ok(BasicMetadataValueEnum::from(pointer_value.as_basic_value_enum()))
        }
    }

    fn decode_user_string(raw_string: &str) -> String {
        raw_string.to_string()
            .replace("\\\"", "\"")
            .replace("\\\\", "\\")
            .replace("\\n", "\n")
            .replace("\\r", "\r")
            .replace("\\t", "\t")
            .replace("\\0", "\0")
    }
}