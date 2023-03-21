use std::collections::HashMap;
use anyhow::*;
use anyhow::Context as AnyhowContext;
use inkwell::{AddressSpace};
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::values::{BasicValue, BasicMetadataValueEnum};
use pest::iterators::{Pair, Pairs};
use std::convert::TryInto;
use std::fs::read_to_string;
use std::path::Path;

use std::rc::{Rc};
use inkwell::basic_block::BasicBlock;
use crate::parser::ast::*;

const ENTRY_BLOCK_NAME: &str = "entry";
const MAIN_FN_NAME: &str = "main";

#[derive(Debug)]
pub struct LLIRCodeGenerator<'a: 'b, 'b> {
    pub context: &'a Context,
    pub modules: HashMap<String, Rc<Module<'b>>>
}

#[derive(Debug)]
pub struct ModuleGeneration<'a: 'b, 'b> {
    pub context: &'a Context,
    pub module: &'b Module<'a>,
    pub builder: &'b Builder<'a>,
}

impl <'a, 'b> LLIRCodeGenerator<'a, 'b> {
    pub fn new(context: &'a Context) -> LLIRCodeGenerator<'a, 'b> {
        LLIRCodeGenerator {
            context,
            modules: HashMap::new(),
        }
    }

    pub fn  compile_source_file<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let input = read_to_string(&path)
            .with_context(|| format!("reading {}", path.as_ref().display()))?;

        self.compile_source(input)
    }

    pub fn compile_source(&mut self, input: String) -> Result<()> {

        let source = parse_to_ast(input.as_ref())?;
        create_main_module(self, source)
            .context("create main module")?;

        Ok(())
    }
}

pub fn create_main_module<'a: 'b, 'b>(ctx: &mut LLIRCodeGenerator<'a,'b>, statement: Statement) -> anyhow::Result<()> {
    create_module(ctx, MAIN_FN_NAME, |ctx: &ModuleGeneration| {
        add_builtins(ctx);
        build_main(ctx, statement)
    })
}

pub fn create_module<'a,F>
(
    ctx: &mut LLIRCodeGenerator,
    name: &str,
    build_module: F
) -> anyhow::Result<()>
    where F: FnOnce(&ModuleGeneration)
{
    let module = Rc::new(ctx.context.create_module(name));
    let builder = ctx.context.create_builder();
    let module_gen = ModuleGeneration {module: &module, context: &ctx.context, builder: &builder};
    build_module(&module_gen);
    ctx.modules.insert(name.to_string(), module);

    Ok(())
}


fn build_main(context: &ModuleGeneration, statement: Statement) {
    let block = create_fn(context, MAIN_FN_NAME);
    context.builder.position_at_end(block);

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
            context.builder.build_return(None);
        }
    }

}

fn add_builtins(ctx: &ModuleGeneration) {
    let char_array = ctx.context
        .i8_type().ptr_type(AddressSpace::default());
    let param_types = &[char_array.into()];
    let void = ctx.context.void_type();
    let type_printf = void.fn_type(param_types, true);
    ctx.module.add_function("printf", type_printf, None);
}

fn create_fn<'a>(context: &'a ModuleGeneration, name: &str) -> BasicBlock<'a> {
    let type_main = context.context.void_type().fn_type(&[], false);
    let main = context.module.add_function(name, type_main, None);

    context.context.append_basic_block(main, ENTRY_BLOCK_NAME)
}

fn build_fn_args<'a: 'b, 'b>(context: &ModuleGeneration<'a, 'b>, pairs: Pairs<Rule>) -> Vec<BasicMetadataValueEnum<'a>> {
    pairs.into_iter()
        .map(|arg| build_fn_arg(context, arg))
        .collect::<Vec<BasicMetadataValueEnum>>()
}

fn build_fn_arg<'a: 'b, 'b>(context: &ModuleGeneration<'a,'b>, pair: Pair<Rule>) -> BasicMetadataValueEnum<'a> {
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