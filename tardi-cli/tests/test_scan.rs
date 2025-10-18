use std::path::PathBuf;
use std::process::Command;

use assert_cmd::prelude::*;
use predicates::prelude::*;

#[test]
fn test_run() {
    let dir_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let hello_tardi = dir_path.join("tests/fixtures/hello.tardi");
    Command::cargo_bin("tardi-next")
        .unwrap()
        .arg("scan")
        .arg(hello_tardi)
        .assert()
        .stdout(predicate::eq(r#":\nhello\n"Hello, "\nprint\nprintln\n;\n"world"\nhello\n"#));
}

