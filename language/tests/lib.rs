use std::path::Path;
use inkwell::context::Context;
use curry_lang_language::engine::CodeGen;

#[test]
fn it_compiles() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/lib/resources")
        .join("hello_world.cry");
    let context = Context::create();
    let code_gen = CodeGen::new(&context);
    code_gen.compile_source(path);
}