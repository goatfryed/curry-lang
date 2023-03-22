use inkwell::values::FunctionValue;
use super::*;

pub fn generate(module_gen: &ModuleGenerator, name: &str, statements: Vec<Statement>) {
    let fn_val = create_void_fn(module_gen, name);
    generate_fn_body(module_gen, statements, fn_val);
}

fn generate_fn_body(module_gen: &ModuleGenerator, statements: Vec<Statement>, fn_val: FunctionValue) {
    let block = module_gen.context.append_basic_block(fn_val, ENTRY_BLOCK_NAME);
    module_gen.builder.position_at_end(block);
    for statement in statements {
        add_statement(module_gen, statement);
    }
    module_gen.builder.build_return(None);
}

fn add_statement(context: &ModuleGenerator, statement: Statement) {
    match statement.kind {
        StatementKind::Assignment(_) => println!("We are aware of assignments, but didn't implement them yet :)"),
        StatementKind::FunctionCall(call) => {
            let mut inner = call.into_inner();
            let symbol_ref = inner.next().expect("function call requires symbol ref").as_str();
            let args = inner.next().map(|args| build_fn_args(context, args.into_inner())).unwrap_or(Vec::new());
            context.builder.build_call(
                context.module.get_function(symbol_ref).unwrap_or_else(|| panic!("{} not defined", symbol_ref)),
                args.as_ref(),
                symbol_ref
            );
        }
    }
}

fn create_void_fn<'a>(context: &'a ModuleGenerator, name: &str) -> FunctionValue<'a> {
    let type_main = context.context.void_type().fn_type(&[], false);
    context.module.add_function(name, type_main, None)
}

fn build_fn_args<'a: 'b, 'b>(context: &ModuleGenerator<'a, 'b>, pairs: Pairs<Rule>) -> Vec<BasicMetadataValueEnum<'a>> {
    pairs.into_iter()
        .map(|arg| build_fn_arg(context, arg))
        .collect::<Vec<BasicMetadataValueEnum>>()
}

fn build_fn_arg<'a: 'b, 'b>(context: &ModuleGenerator<'a,'b>, pair: Pair<Rule>) -> BasicMetadataValueEnum<'a> {
    match pair.as_rule() {
        Rule::expression => {},
        _ => panic!("unsupported pair {:?}", pair)
    }
    let pair = pair.into_inner().unique_pair().expect("expression without a single token below");
    return match pair.as_rule() {
        Rule::value => context.builder
            .build_global_string_ptr(pair.as_str(), "arg")
            .as_basic_value_enum().try_into().unwrap()
        ,
        Rule::function_call => todo!("function call as function argument"),
        _ => panic!("unsupported pair {:?}", pair)
    }
}

pub mod libc {
    use inkwell::AddressSpace;
    use crate::ll_code_gen::ModuleGenerator;

    pub fn declare_libc_builtin(module_gen: &ModuleGenerator) {
        declare_println(module_gen);
    }

    fn declare_println(module_gen: &ModuleGenerator) {
        let char_array = module_gen.context
            .i8_type().ptr_type(AddressSpace::default());
        let param_types = &[char_array.into()];
        let void = module_gen.context.void_type();
        let type_printf = void.fn_type(param_types, true);
        module_gen.module.add_function("printf", type_printf, None);
    }
}
