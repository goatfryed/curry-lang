use std::path::{Path, PathBuf};
use std::fs::{create_dir_all, File, OpenOptions};
use std::process::{Command, Stdio};
use curry_lang_language::{LLIRCodeGenerator,Context};

#[test]
fn hello_world() {
    run_baseline_test("hello_world");
}
#[test]
fn n_statements() {
    run_baseline_test("n_statements");
}
#[test]
fn calumni() {
    run_baseline_test("calumni");
}
#[test]
fn minimal_program() {
    run_baseline_test("minimal_program");
}



fn run_baseline_test(key: &str) {
    let test_dir = &Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/baseline_tests")
        .join(key);
    create_dir_all(test_dir).unwrap();

    let input = test_dir.join( key.to_owned() + ".cry");
    let ir_file = key.to_owned() + ".ll";
    let ir_path = test_dir.join(&ir_file);
    let binary = "binary";

    let actual_output = &format!("{}.actual.{}", key.to_owned(), "out");
    let actual_errors = &format!("{}.actual.{}", key.to_owned(), "err");

    generate_code_for_source(input, ir_path);


    Command::new("clang")
        .current_dir(test_dir)
        .arg(ir_file)
        .args(["-o", binary])
        .status()
        .unwrap_or_else(|e| panic!("failed to compile generated llir\n{}", e));

    Command::new(format!("./{}", binary))
        .current_dir(test_dir)
        .stdout(get_output(test_dir, actual_output))
        .stderr(get_output(test_dir, actual_errors))
        .status()
        .unwrap_or_else(|e| panic!("failed to execute compiled programm\n{}", e));

    diff_output(test_dir, key, "err", actual_errors);
    diff_output(test_dir, key, "out", actual_output);
}

fn generate_code_for_source(input: PathBuf, ir_path: PathBuf) {
    let context = Context::create();
    let mut code_gen = LLIRCodeGenerator::new(&context);
    code_gen.compile_source_file(input).unwrap();
    let modules = &mut code_gen.modules;
    assert_eq!(1, modules.len());
    modules.remove("main").unwrap().print_to_file(ir_path).unwrap();
}

fn diff_output(test_dir: &Path, key: &str, output_type: &str, actual: &str) {
    let expected_output = &format!("{}.expected.{}", key.to_owned(), output_type);
    let expected_output_path = &test_dir.join(expected_output);

    if !expected_output_path.exists() {
        File::create(expected_output_path)
            .unwrap_or_else(|e| panic!("failed to ensure {} exists\n{}", expected_output_path.display(), e));
    }

    let status = Command::new("diff")
        .current_dir(test_dir)
        .arg("-u")
        .arg(expected_output)
        .arg(actual)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .unwrap_or_else(|e| panic!("failed to invoke diff. Are you missing diff?\n{}", e));

    if !status.success() {
        panic!("{} did not meet expectation, see above", expected_output);
    }
}

fn get_output(path: &Path, file: &str) -> Stdio {
    let mut options = OpenOptions::new();
    options
        .create(true)
        .write(true)
        .truncate(true);

    let file = options
        .open(path.join(file))
        .unwrap();

    return Stdio::from(file);
}

macro_rules! _run_baseline_test {
    ($name:ident) => {
        #[test]
        fn $name() {
            run_baseline_test(stringify!($name));
        }
    };
}