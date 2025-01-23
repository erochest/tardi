/**
 * @file A tree-sitter for Tardi, a small stack-based language.
 * @author Eric Rochester <erochest@gmail.com>
 * @license MIT
 */

/// <reference types="tree-sitter-cli/dsl" />
// @ts-check

module.exports = grammar({
  name: "tardi",

  rules: {
    // TODO: add the actual grammar rules
    source_file: $ => "hello"
  }
});
