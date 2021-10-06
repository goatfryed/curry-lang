use inkwell::{AddressSpace, OptimizationLevel};
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::execution_engine::{ExecutionEngine, JitFunction};
use inkwell::module::Module;
use inkwell::values::BasicValue;

type MainFn = unsafe extern "C" fn();

pub struct CodeGen<'ctx> {
    pub context: &'ctx Context,
    pub entry: Module<'ctx>,
    builder: Builder<'ctx>,
    engine: ExecutionEngine<'ctx>
}

impl<'ctx> CodeGen<'ctx> {
    pub fn new(context: &Context) -> CodeGen {
        let system = context.create_module("system");
        let builder = context.create_builder();
        let engine =
            system.create_jit_execution_engine(OptimizationLevel::None)
            .expect("failed to create execution engine");
        return  CodeGen { context, entry: system, builder, engine };
    }

    pub fn compile_printer(&self) -> JitFunction<'ctx, MainFn> {

        self.add_builtins();

        let type_main = self.context.void_type().fn_type(&[], false);
        let main = self.entry.add_function("main", type_main, None);
        let entry = self.context.append_basic_block(main, "entry");
        self.builder.position_at_end(entry);

        let format = self.builder.build_global_string_ptr("Hello World! Greetings from %s!\n", "format");
        let value = self.builder.build_global_string_ptr("curry-lang", "value");

        self.builder.build_call(
            self.entry.get_function("printf").expect("printf not defined"),
            &[format.as_basic_value_enum(), value.as_basic_value_enum()],
            "printf"
        );
        self.builder.build_return(None);

        let main = unsafe { self.engine.get_function("main").expect("unable to compile").to_owned() };
        return main;
    }

    fn add_builtins(&self) {
        let char_array = self.context.i8_type().ptr_type(AddressSpace::Generic);
        let param_types = &[char_array.into()];
        let void = self.context.void_type();
        let type_printf = void.fn_type(param_types, true);
        self.entry.add_function("printf", type_printf, None);
    }
}