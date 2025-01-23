// swift-tools-version:5.3
import PackageDescription

let package = Package(
    name: "TreeSitterTardi",
    products: [
        .library(name: "TreeSitterTardi", targets: ["TreeSitterTardi"]),
    ],
    dependencies: [
        .package(url: "https://github.com/ChimeHQ/SwiftTreeSitter", from: "0.8.0"),
    ],
    targets: [
        .target(
            name: "TreeSitterTardi",
            dependencies: [],
            path: ".",
            sources: [
                "src/parser.c",
                // NOTE: if your language has an external scanner, add it here.
            ],
            resources: [
                .copy("queries")
            ],
            publicHeadersPath: "bindings/swift",
            cSettings: [.headerSearchPath("src")]
        ),
        .testTarget(
            name: "TreeSitterTardiTests",
            dependencies: [
                "SwiftTreeSitter",
                "TreeSitterTardi",
            ],
            path: "bindings/swift/TreeSitterTardiTests"
        )
    ],
    cLanguageStandard: .c11
)
