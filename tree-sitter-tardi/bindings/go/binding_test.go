package tree_sitter_tardi_test

import (
	"testing"

	tree_sitter "github.com/tree-sitter/go-tree-sitter"
	tree_sitter_tardi "github.com/erochest/tardi/bindings/go"
)

func TestCanLoadGrammar(t *testing.T) {
	language := tree_sitter.NewLanguage(tree_sitter_tardi.Language())
	if language == nil {
		t.Errorf("Error loading Tardi grammar")
	}
}
