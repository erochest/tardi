use std::{env, str::FromStr};

use pretty_assertions::assert_eq;

use super::*;

#[test]
fn edit_mode_converts_to_rustline() {
    assert_eq!(
        rustyline::EditMode::Emacs,
        crate::config::EditMode::Emacs.into()
    );
    assert_eq!(rustyline::EditMode::Vi, crate::config::EditMode::Vi.into());
}

#[test]
fn repl_config_default_uses_emacs_keybindings() {
    let config = ReplConfig::default();
    assert_eq!(EditMode::Emacs, config.edit_mode);
}

#[test]
fn config_default_includes_current_directory_in_module_path() {
    let config = Config::default();
    let pwd = env::current_dir().unwrap();
    assert!(config.module_path.contains(&pwd));
}

#[test]
fn config_default_includes_repl_settings_defaults() {
    let config = Config::default();
    assert_eq!(EditMode::Emacs, config.repl.edit_mode);
    assert!(config.repl.history_file.is_none());
}

// TODO: Config::try_from
// TODO: Config::figment
// TODO: rustyline::Config::from
// TODO: Config Provider
