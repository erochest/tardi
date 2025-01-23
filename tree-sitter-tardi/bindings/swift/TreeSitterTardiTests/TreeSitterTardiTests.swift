import XCTest
import SwiftTreeSitter
import TreeSitterTardi

final class TreeSitterTardiTests: XCTestCase {
    func testCanLoadGrammar() throws {
        let parser = Parser()
        let language = Language(language: tree_sitter_tardi())
        XCTAssertNoThrow(try parser.setLanguage(language),
                         "Error loading Tardi grammar")
    }
}
