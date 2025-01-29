use std::fs;
use std::process::Command;

use serde::Deserialize;
use toml;

use assert_cmd::prelude::*;
use assert_fs::prelude::*;
use assert_fs::NamedTempFile;
use pretty_assertions::assert_eq;

static TEST_SUITE_FILE: &str = "tests/test_suite.toml";

#[derive(Debug, Deserialize)]
struct TestSuite {
    #[serde(rename = "TestCase")]
    test_cases: Vec<TestCase>
}

#[derive(Debug, Deserialize)]
struct TestCase {
    name: String,
    input: String,
    status: Option<i32>,
    stdout: Option<String>,
    stderr: Option<String>,
}

#[test]
fn test_run() {
    let test_suite_spec = fs::read_to_string(TEST_SUITE_FILE).unwrap();
    let test_suite: TestSuite = toml::from_str(&test_suite_spec).unwrap();

    for test_case in test_suite.test_cases {
        // TODO: better integrate these into the testing framework. Maybe make this a macro that reads the test suite at compile time?

        // Create temporary file with .tardi extension
        let temp_file = NamedTempFile::new("test.tardi").unwrap();
        temp_file.write_str(&test_case.input).unwrap();
        
        // Run command and capture output
        let output = Command::cargo_bin(env!["CARGO_PKG_NAME"])
            .unwrap()
            .arg("--print-stack")
            .arg(temp_file.path())
            .output()
            .unwrap();

        // Validate results
        validate_status(&test_case, &output);
        validate_stdout(&test_case, &output);
        validate_stderr(&test_case, &output);
    }
}

fn validate_status(test_case: &TestCase, output: &std::process::Output) {
    if let Some(expected_status) = test_case.status {
        assert_eq!(
            output.status.code().unwrap(),
            expected_status,
            "Test case '{}' status mismatch. Error message: {}",
            test_case.name,
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

fn validate_stdout(test_case: &TestCase, output: &std::process::Output) {
    if let Some(ref expected_stdout) = test_case.stdout {
        assert_eq!(
            &String::from_utf8_lossy(&output.stdout),
            expected_stdout,
            "Test case '{}' stdout mismatch",
            test_case.name
        );
    }
}

fn validate_stderr(test_case: &TestCase, output: &std::process::Output) {
    if let Some(ref expected_stderr) = test_case.stderr {
        assert_eq!(
            &String::from_utf8_lossy(&output.stderr),
            expected_stderr,
            "Test case '{}' stderr mismatch",
            test_case.name
        );
    }
}
