use std::fs::create_dir_all;
use std::path::{Path};
use inkwell::context::Context;
use curry_lang_language::engine::CodeGen;

#[test]
fn it_compiles_hello_world() {
    let context = Context::create();
    let code_gen = CodeGen::new(&context);
    code_gen.compile_printer();
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/engine")
        .join("generated");
    create_dir_all(&path).unwrap();
    let llvm_ir_file = path.join("it_works.ll");
    code_gen.entry.print_to_file(&llvm_ir_file).unwrap();

    std::process::Command::new("clang")
        .current_dir(&path)
        .arg("it_works.ll")
        .args(["-o","it_works"])
        .status()
        .expect("failed to compile it_works");

    let output = std::process::Command::new("./it_works")
        .current_dir(&path)
        .output()
        .expect("failed to execute compiled programm");

    let msg = String::from_utf8(output.stdout)
        .expect("failed to write programs stdout to string");
    assert_eq!("Hello World! Greetings from curry-lang!\n", msg)

}