use std::env;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use pretty_assertions::assert_eq;

use super::*;
use crate::error::{Error, Result};

fn setup() -> Result<ModuleManager> {
    let pwd = env::current_dir()?;
    let paths = vec![&pwd, Path::new("./tests/modules")];

    let loader = ModuleManager::new(&paths);

    Ok(loader)
}

#[test]
fn test_module_loader_finds_file_in_current() {
    let expected = PathBuf::from_str("tests/fixtures/basic-test.tardi").unwrap();
    let expected = expected.canonicalize().unwrap();
    let expected = ("tests/fixtures/basic-test".to_string(), expected);
    let loader = setup().unwrap();

    let result = loader.find("tests/fixtures/basic-test", None);
    assert!(result.is_ok(), "error: {:?}", result);
    let target = result.unwrap();

    assert!(target.is_some(), "option: {:?}", target);
    let target = target.unwrap();
    assert_eq!(expected, target);
}

#[test]
fn test_module_loader_finds_file_in_path() {
    let expected = PathBuf::from_str("tests/modules/hello.tardi").unwrap();
    let expected = expected.canonicalize().unwrap();
    let expected = ("hello".to_string(), expected);
    let loader = setup().unwrap();

    let result = loader.find("hello", None);
    assert!(result.is_ok(), "error: {:?}", result);
    let target = result.unwrap();

    assert!(target.is_some(), "option: {:?}", target);
    let target = target.unwrap();
    assert_eq!(expected, target);
}

#[test]
fn test_module_loader_does_not_find_files() {
    let loader = setup().unwrap();

    let result = loader.find("does/not/exist", None);
    assert!(result.is_ok(), "error: {:?}", result);
    let target = result.unwrap();

    assert!(target.is_none(), "option: {:?}", target);
}

#[test]
fn test_module_loader_finds_relative_files() {
    let context = PathBuf::from_str("tests/modules/hello.tardi").unwrap();
    let context = context.canonicalize().unwrap();
    let expected = PathBuf::from_str("tests/fixtures/basic-test.tardi").unwrap();
    let expected = expected.canonicalize().unwrap();
    let expected = ("basic-test".to_string(), expected);
    let loader = setup().unwrap();

    let result = loader.find("../fixtures/basic-test", Some(&context));
    assert!(result.is_ok(), "error: {:?}", result);
    let target = result.unwrap();

    assert!(target.is_some(), "option: {:?}", target);
    let target = target.unwrap();
    assert_eq!(expected, target);
}

#[test]
fn test_module_loader_does_not_find_relative_files_without_context() {
    let loader = setup().unwrap();

    let result = loader.find("../fixtures/basic-test", None);
    assert!(result.is_err(), "ok: {:?}", result);
    assert!(matches!(
        result.unwrap_err(),
        Error::CompilerError(CompilerError::ModuleNotFound(name)) if name == "../fixtures/basic-test"
    ));
}

#[test]
fn test_module_loader_does_not_find_relative_files_that_dont_exist() {
    let context = PathBuf::from_str("tests/modules/hello.tardi").unwrap();
    let context = context.canonicalize().unwrap();
    let loader = setup().unwrap();

    let result = loader.find("../does-not-exist/basic-test", Some(&context));
    assert!(result.is_ok(), "error: {:?}", result);
    let target = result.unwrap();

    assert!(target.is_none(), "option: {:?}", target);
}

#[test]
fn test_find_abs_module_when_file_missing() {
    let loader = ModuleManager::new(&[env::current_dir().unwrap().join("tests/modules")]);

    let result = loader.find_abs_module(
        Path::new("tests/fixtures/test-fixture.tardi"),
        "../modules/does-not-exist",
    );

    assert!(result.is_ok(), "error: {:?}", result);
    let result = result.unwrap();
    assert!(result.is_none(), "option: {:?}", result);
}

#[test]
fn test_find_abs_module_when_outside_search_path() {
    let cwd = env::current_dir().unwrap();
    let expected = PathBuf::from_str("src/bootstrap/00-core-macros.tardi")
        .unwrap()
        .canonicalize()
        .unwrap();
    let loader = ModuleManager::new(&[cwd.join("tests/modules")]);

    let result = loader.find_abs_module(
        &cwd.join("tests/fixtures/test-fixture.tardi"),
        "../../src/bootstrap/00-core-macros",
    );

    assert!(
        result
            .as_ref()
            .is_err_and(|err| matches!(err, CompilerError::InvalidModulePath(p) if p == &expected)),
        "result: {:?}",
        result
    );
}

#[test]
fn test_find_abs_module_finds_relative_uses() {
    let cwd = env::current_dir().unwrap();
    let expected = PathBuf::from_str("tests/modules/basic-test.tardi")
        .unwrap()
        .canonicalize()
        .unwrap();
    let loader = ModuleManager::new(&[cwd.join("tests/modules")]);

    let result = loader.find_abs_module(
        &cwd.join("tests/fixtures/test-fixture.tardi"),
        "../modules/basic-test",
    );

    assert!(
        result.as_ref().is_ok_and(|r| r.is_some()),
        "result: {:?}",
        result
    );
    let (name, target_path) = result.unwrap().unwrap();
    assert_eq!(name, "basic-test".to_string());
    assert_eq!(target_path, expected);
}
