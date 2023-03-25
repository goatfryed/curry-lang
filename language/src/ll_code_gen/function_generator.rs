use std::cell::{RefCell};
use inkwell::values::{FunctionValue, PointerValue};
use crate::ll_code_gen::assignment::{Assignment};
use anyhow::Context as AnyhowContext;
use inkwell::basic_block::BasicBlock;
use super::*;

pub struct FunctionGenerator<'gen: 'module, 'module: 'func, 'func> {
    parent: &'func ModuleGenerator<'gen, 'module>,
    symbols: RefCell<HashMap<String, PointerValue<'gen>>>,
    builder: Builder<'gen>,
    function: FunctionValue<'gen>,
    entry: BasicBlock<'gen>,
}

impl <'gen: 'module, 'module: 'func, 'func> FunctionGenerator<'gen, 'module, 'func> {

    pub fn generate(module_gen: &'func ModuleGenerator<'gen, 'module>, name: &str, statements: Vec<Statement>) {
        let function = FunctionGenerator::declare_void_fn(module_gen, name);
        let fn_gen = FunctionGenerator::create_generator(module_gen, function);

        for statement in statements {
            fn_gen.add_statement(statement);
        }

        fn_gen.builder.build_return(None);
    }

    pub fn declare_void_fn(context: &'func ModuleGenerator<'gen, 'module>, name: &str) -> FunctionValue<'gen> {
        let type_main = context.parent.context.void_type().fn_type(&[], false);
        context.module.add_function(name, type_main, None)
    }

    pub fn create_generator(module: &'func ModuleGenerator<'gen, 'module>, function: FunctionValue<'gen>) -> FunctionGenerator<'gen, 'module, 'func> {
        let builder = module.parent.context.create_builder();

        let entry = module.parent.context.append_basic_block(function, ENTRY_BLOCK_NAME);
        builder.position_at_end(entry);

        FunctionGenerator {
            parent: module,
            function,
            builder,
            entry,
            symbols: RefCell::new(HashMap::new()),
        }
    }

    pub fn add_statement(&self, statement: Statement) {
        match statement.kind {
            StatementKind::Assignment(pair) => {
                let assignment : Assignment = pair.try_into().context("resolve assignment").unwrap();
                self.generate_string_assignment(assignment).unwrap();
            },
            StatementKind::FunctionCall(call) => {
                let mut inner = call.into_inner();
                let symbol_ref = inner.next().expect("function call requires symbol ref").as_str();
                let args = inner.next().map(|args| self.build_fn_args(args.into_inner())).unwrap_or(Vec::new());

                self.builder.build_call(
                    self.parent.module.get_function(symbol_ref).unwrap_or_else(|| panic!("{} not defined", symbol_ref)),
                    args.as_ref(),
                    symbol_ref
                );
            }
        }
    }

    pub fn generate_string_assignment(&self, assignment: Assignment) -> anyhow::Result<()> {
        let name = assignment.symbol_ref.name;
        let mut symbols = self.symbols.borrow_mut();
        if symbols.contains_key(name.as_str()) {
            return Err(Error::msg(format!("variable '{}' is already defined", name)))
        }
        let array_value = self.parent.parent.context.const_string(assignment.value.value.as_bytes(), true);
        let pointer = self.builder.build_alloca(array_value.get_type(), name.as_str());
        self.builder.build_store(pointer, array_value);

        symbols.insert(name, pointer);
        Ok(())
    }

    fn build_fn_args(&self, pairs: Pairs<Rule>) -> Vec<BasicMetadataValueEnum<'gen>> {
        pairs.into_iter()
            .map(|arg| self.build_fn_arg(arg))
            .collect::<Vec<BasicMetadataValueEnum>>()
    }

    fn build_fn_arg(&self, pair: Pair<Rule>) -> BasicMetadataValueEnum<'gen> {
        match pair.as_rule() {
            Rule::value => {
                let value_expr = pair.into_inner().expect_unique_pair();
                match value_expr.as_rule() {
                    Rule::string => {
                        let str_val = value_expr.into_inner().expect_unique_pair().as_str();
                        self.builder.build_global_string_ptr(str_val, "str_val")
                            .as_basic_value_enum().try_into().expect("")
                    },
                    _ => unreachable!()
                }
            },
            _ => unreachable!()
        }
    }
}

pub mod libc {
    use inkwell::AddressSpace;
    use crate::ll_code_gen::ModuleGenerator;

    pub fn declare_libc_builtin(module_gen: &ModuleGenerator<'_, '_>) {
        declare_println(module_gen);
    }

    fn declare_println(module_gen: &ModuleGenerator<'_, '_>) {
        let char_array = module_gen.parent.context
            .i8_type().ptr_type(AddressSpace::default());
        let param_types = &[char_array.into()];
        let void = module_gen.parent.context.void_type();
        let type_printf = void.fn_type(param_types, true);
        module_gen.module.add_function("printf", type_printf, None);
    }
}
