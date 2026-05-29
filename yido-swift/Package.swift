// swift-tools-version: 6.2

import PackageDescription

let package = Package(
    name: "YidoInputMethod",
    platforms: [
        .macOS("26.0"),
    ],
    products: [
        .executable(name: "Yido", targets: ["Yido"]),
        .library(name: "YidoCore", targets: ["YidoCore"]),
        .library(name: "YidoInputMethod", targets: ["YidoInputMethod"]),
    ],
    dependencies: [
        .package(url: "https://github.com/vChewing/IMKSwift.git", from: "26.03.07"),
    ],
    targets: [
        .target(
            name: "CYidoFFI",
            publicHeadersPath: "include"
        ),
        .target(
            name: "YidoCore",
            dependencies: [
                "CYidoFFI",
            ]
        ),
        .target(
            name: "YidoInputMethod",
            dependencies: [
                "YidoCore",
                .product(name: "IMKSwift", package: "IMKSwift"),
            ],
            resources: [
                .process("Resources"),
            ]
        ),
        .executableTarget(
            name: "Yido",
            dependencies: [
                "YidoInputMethod",
                .product(name: "IMKSwift", package: "IMKSwift"),
            ]
        ),
        .testTarget(
            name: "YidoCoreTests",
            dependencies: ["YidoCore"]
        ),
        .testTarget(
            name: "YidoInputMethodTests",
            dependencies: ["YidoInputMethod"]
        ),
    ]
)
