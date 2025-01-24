use std::fs;
use std::process::Command;

use serde::Deserialize;
use toml;

use assert_cmd::prelude::*;
use assert_fs::prelude::*;
use assert_fs::NamedTempFile;

static TEST_SUITE_FILE: &str = "tests/test_suite.toml";

#[derive(Debug, Deserialize)]
struct TestCase {
    name: String,
    input: String,
    status: Option<isize>,
    stdout: Option<String>,
    stderr: Option<String>,
}

#[test]
fn test_run() {
    let test_suite_spec = fs::read_to_string(TEST_SUITE_FILE).unwrap();
    let test_suite: Vec<TestCase> = toml::from_str(&test_suite_spec).unwrap();

    for test_case in test_suite {
        // Create temporary file with .tardi extension
        let temp_file = NamedTempFile::new("test.tardi").unwrap();
        temp_file.write_str(&test_case.input).unwrap();
        
        // Run command and capture output
        let output = Command::cargo_bin(env!["CARGO_PKG_NAME"])
            .unwrap()
            .arg(temp_file.path())
            .output()
            .unwrap();

        // Validate results
        if let Some(expected_status) = test_case.status {
            assert_eq!(
                output.status.code().unwrap() as isize,
                expected_status,
                "Test case '{}' status mismatch",
                test_case.name
            );
        }

        if let Some(expected_stdout) = test_case.stdout {
            assert_eq!(
                String::from_utf8_lossy(&output.stdout),
                expected_stdout,
                "Test case '{}' stdout mismatch",
                test_case.name
            );
        }

        if let Some(expected_stderr) = test_case.stderr {
            assert_eq!(
                String::from_utf8_lossy(&output.stderr),
                expected_stderr,
                "Test case '{}' stderr mismatch",
                test_case.name
            );
        }
    }
}
