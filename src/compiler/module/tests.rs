use std::env;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use pretty_assertions::assert_eq;

use super::*;
use crate::error::{Error, Result};

fn setup() -> Result<Loader> {
    let pwd = env::current_dir()?;
    let paths = vec![&pwd, Path::new("./tests/modules")];

    let loader = Loader::new(&paths);

    Ok(loader)
}

#[test]
fn test_module_loader_finds_file_in_current() {
    let expected = PathBuf::from_str("tests/fixtures/basic-test.tardi").unwrap();
    let expected = expected.canonicalize().unwrap();
    let loader = setup().unwrap();

    let result = loader.find("tests/fixtures/basic-test", None);
    assert!(result.is_ok(), "error: {:?}", result);
    let target = result.unwrap();

    assert!(target.is_some(), "option: {:?}", target);
    let target = target.unwrap().canonicalize().unwrap();
    assert_eq!(expected, target);
}

#[test]
fn test_module_loader_finds_file_in_path() {
    let expected = PathBuf::from_str("tests/modules/hello.tardi").unwrap();
    let expected = expected.canonicalize().unwrap();
    let loader = setup().unwrap();

    let result = loader.find("hello", None);
    assert!(result.is_ok(), "error: {:?}", result);
    let target = result.unwrap();

    assert!(target.is_some(), "option: {:?}", target);
    let target = target.unwrap().canonicalize().unwrap();
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
    let loader = setup().unwrap();

    let result = loader.find("../fixtures/basic-test", Some(&context));
    assert!(result.is_ok(), "error: {:?}", result);
    let target = result.unwrap();

    assert!(target.is_some(), "option: {:?}", target);
    let target = target.unwrap().canonicalize().unwrap();
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
