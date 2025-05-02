use std::{env, str::FromStr};

use pretty_assertions::*;

use super::*;

#[test]
fn test_config_module_paths_accumulate() {
    let pwd = env::current_dir().unwrap();
    let path = Path::new("tests/fixtures/tardi.toml");

    let result = read_config_sources(&Some(path));
    assert!(result.is_ok(), "error: {:?}", result);
    let config = result.unwrap();

    assert!(config.module_path.contains(&pwd));
    assert!(config
        .module_path
        .contains(&PathBuf::from_str("tests/modules").unwrap()));
}
