use std::fs;
use std::path::Path;
use std::process::{Command, Output};

use assert_cmd::prelude::*;
use pretty_assertions::assert_eq;

fn test_tardi_file(tardi_file: &Path) -> datatest_stable::Result<()> {
    let output = Command::cargo_bin(env!["CARGO_PKG_NAME"])
        .unwrap()
        .arg("--print-stack")
        .arg(tardi_file)
        .output()
        .unwrap();

    // Validate results
    validate_status(tardi_file, &output);
    validate_print_output(
        tardi_file,
        "stdout",
        &String::from_utf8_lossy(&output.stdout),
    );
    validate_print_output(
        tardi_file,
        "stderr",
        &String::from_utf8_lossy(&output.stderr),
    );

    Ok(())
}

fn validate_status(tardi_file: &Path, output: &Output) {
    let status_file = tardi_file.with_extension("status");

    let expected_status = if status_file.exists() {
        fs::read_to_string(status_file)
            .unwrap()
            .trim()
            .parse::<i32>()
            .unwrap()
    } else {
        0
    };

    assert_eq!(
        output.status.code().unwrap(),
        expected_status,
        "Test file '{}' failed. Error message: {}",
        tardi_file.display(),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn validate_print_output(tardi_file: &Path, extension: &str, actual_output: &str) {
    let output_file = tardi_file.with_extension(extension);

    if output_file.exists() {
        let expected = fs::read_to_string(output_file).unwrap();
        
        // Normalize line endings to \n
        let expected_normalized = expected.replace("\r\n", "\n");
        let actual_normalized = actual_output.replace("\r\n", "\n");
        
        assert_eq!(
            actual_normalized,
            expected_normalized,
            "Test file '{}' {} mismatch",
            tardi_file.display(),
            extension
        );
    }
}

datatest_stable::harness! (
    {
        test = test_tardi_file,
        root = "tests/fixtures",
        pattern = r"^.*\.tardi$",
    }
);
