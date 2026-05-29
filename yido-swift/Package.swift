// swift-tools-version: 6.2

import PackageDescription

let package = Package(
    name: "YidoInputMethod",
    platforms: [
        .macOS("26.0"),
    ],
    products: [
        .library(name: "YidoInputCore", targets: ["YidoInputCore"]),
        .library(name: "YidoSettings", targets: ["YidoSettings"]),
        .library(name: "YidoRustFFI", targets: ["YidoRustFFI"]),
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
            name: "YidoInputCore"
        ),
        .target(
            name: "YidoSettings"
        ),
        .target(
            name: "YidoRustFFI",
            dependencies: [
                "CYidoFFI",
                "YidoInputCore",
            ]
        ),
        .target(
            name: "YidoInputMethod",
            dependencies: [
                "YidoInputCore",
                "YidoRustFFI",
                "YidoSettings",
                .product(name: "IMKSwift", package: "IMKSwift"),
            ]
        ),
        .testTarget(
            name: "YidoInputCoreTests",
            dependencies: ["YidoInputCore"]
        ),
        .testTarget(
            name: "YidoSettingsTests",
            dependencies: ["YidoSettings"]
        ),
    ]
)
