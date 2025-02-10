use std::io::Write;
use std::process::{Command, Stdio};

use assert_cmd::prelude::*;

#[ignore = "fixing this test"]
#[test]
fn test_repl() {
    let mut cmd = Command::cargo_bin("tardi").unwrap()
        .stdin(Stdio::piped())
        .spawn()
        .unwrap();
    let mut child_stdin = cmd.stdin.take().unwrap();
    child_stdin.write_all(b"1 2 +\n/quit\n").unwrap();
    let output = cmd.wait_with_output().unwrap();
    output.assert()
        .success()
        .stdout(predicates::str::contains("3\n"));
}
