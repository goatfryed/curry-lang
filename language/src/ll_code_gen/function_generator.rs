use std::cell::{RefCell};
use inkwell::values::{BasicValueEnum, FunctionValue, PointerValue};
use crate::ll_code_gen::assignment::Assignment;
use anyhow::Context as AnyhowContext;
use inkwell::basic_block::BasicBlock;
use crate::ll_code_gen::expression::generate_expression;
use super::*;

pub struct FunctionGenerator<'gen: 'module, 'module: 'func, 'func> {
    pub parent: &'func ModuleGenerator<'gen, 'module>,
    pub symbols: RefCell<HashMap<String, PointerValue<'gen>>>,
    pub builder: Builder<'gen>,
    function: FunctionValue<'gen>,
    _entry: BasicBlock<'gen>,
    last_value: RefCell<Option<BasicValueEnum<'gen>>>,
}

impl <'gen: 'module, 'module: 'func, 'func> FunctionGenerator<'gen, 'module, 'func> {

    pub fn generate(module_gen: &'func ModuleGenerator<'gen, 'module>, name: &str, statements: Vec<Statement>) {
        let function = FunctionGenerator::declare_void_fn(module_gen, name);
        let fn_gen = FunctionGenerator::create_generator(module_gen, function);

        for statement in statements {
            fn_gen.add_statement(statement);
        }

        fn_gen.complete();
    }

    pub fn complete(self) {
        let last_value: Option<BasicValueEnum<'gen>> = self.last_value.into_inner();
        // self.builder.build_return(last_value.as_ref()); // <- does not work

        match last_value {
            Some(value) => {
                self.builder.build_return(Some(&value));
            },
            None => { self.builder.build_return(None); },
        }

        self.function.verify(true);
    }

    pub fn declare_void_fn(context: &'func ModuleGenerator<'gen, 'module>, name: &str) -> FunctionValue<'gen> {
        let fn_type = context.parent.context.void_type().fn_type(&[], false);
        context.module.add_function(name, fn_type, None)
    }

    pub fn create_generator(module: &'func ModuleGenerator<'gen, 'module>, function: FunctionValue<'gen>) -> FunctionGenerator<'gen, 'module, 'func> {
        let builder = module.parent.context.create_builder();

        let entry = module.parent.context.append_basic_block(function, ENTRY_BLOCK_NAME);
        builder.position_at_end(entry);

        FunctionGenerator {
            parent: module,
            function,
            builder,
            _entry: entry,
            symbols: RefCell::new(HashMap::new()),
            last_value: RefCell::new(None),
        }
    }

    pub fn add_statement(&self, statement: Statement) {
        match statement.kind {
            StatementKind::Assignment(pair) => {
                let assignment : Assignment = pair.try_into().context("resolve assignment").unwrap();
                let value_enum = assignment.process(self).context("function block -> statement").unwrap();
                self.last_value.replace(Some(value_enum));
            },
            StatementKind::FunctionCall(call) => {
                self.process_function_call(call);
            }
        }
    }

    fn build_fn_args(&self, pairs: Pairs<Rule>) -> Result<Vec<BasicMetadataValueEnum<'gen>>> {
        pairs.into_iter()
            .map(|arg| generate_expression(arg, self))
            .collect::<Result<Vec<BasicMetadataValueEnum>>>()
    }


    pub fn process_function_call(&self, call: Pair<Rule>) -> Option<BasicValueEnum<'gen>> {
        let mut inner = call.into_inner();
        let symbol_ref = inner.next().expect("function call requires symbol ref").as_str();
        let fn_args = inner.next().expect("function call requires arguments");
        let args = self.build_fn_args(fn_args.into_inner())
            .context("resolve function arguments")
            .unwrap();

        self.create_function_call(symbol_ref, args)
    }

    pub fn create_function_call(&self, symbol_ref: &str, args: Vec<BasicMetadataValueEnum<'gen>>) -> Option<BasicValueEnum<'gen>> {
        let value = self.builder.build_call(
            self.parent.module.get_function(symbol_ref).unwrap_or_else(|| panic!("{} not defined", symbol_ref)),
            args.as_ref(),
            symbol_ref
        );
        value.try_as_basic_value().left()
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
