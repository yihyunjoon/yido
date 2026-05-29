import Foundation
import Testing

@testable import YidoSettings

private let bundledLayout = """
[layout]
id = "ko-dubeolsik"
name = "Korean Dubeolsik"
engine = "hangul"

[keys.r]
normal = { jamo = "ㄱ", role = "auto" }
"""

private let customLayout = """
[layout]
id = "custom"
name = "Custom"
engine = "hangul"

[keys.r]
normal = { jamo = "ㄱ", role = "auto" }
"""

@Test
func bundledDubeolsikIsAlwaysListed() throws {
    let store = try makeStore()

    let layouts = try store.layouts()

    #expect(layouts == [
        LayoutDescriptor(id: "ko-dubeolsik", name: "Korean Dubeolsik", isBundled: true),
    ])
}

@Test
func bundledDubeolsikCannotBeRemoved() throws {
    let store = try makeStore()

    #expect(throws: LayoutStoreError.cannotRemoveBundledLayout("ko-dubeolsik")) {
        try store.removeLayout(id: "ko-dubeolsik")
    }
}

@Test
func importsUserLayoutAndRejectsDuplicateID() throws {
    let store = try makeStore()
    let sourceURL = try writeTemporaryLayout(customLayout)

    let imported = try store.importLayout(from: sourceURL)

    #expect(imported == LayoutDescriptor(id: "custom", name: "Custom", isBundled: false))
    #expect(try store.layouts().map(\.id) == ["ko-dubeolsik", "custom"])
    #expect(throws: LayoutStoreError.duplicateLayoutID("custom")) {
        try store.importLayout(from: sourceURL)
    }
}

@Test
func rejectsBundledIDAndUnsupportedEngine() throws {
    let store = try makeStore()
    let duplicateBundled = try writeTemporaryLayout(bundledLayout)
    let unsupported = try writeTemporaryLayout(
        """
        [layout]
        id = "romaja"
        name = "Romaja"
        engine = "romaja"
        """
    )

    #expect(throws: LayoutStoreError.duplicateLayoutID("ko-dubeolsik")) {
        try store.importLayout(from: duplicateBundled)
    }
    #expect(throws: LayoutStoreError.unsupportedEngine("romaja")) {
        try store.importLayout(from: unsupported)
    }
}

private func makeStore() throws -> LayoutStore {
    LayoutStore(
        bundledLayoutSource: bundledLayout,
        userLayoutsDirectory: try temporaryDirectory()
    )
}

private func writeTemporaryLayout(_ source: String) throws -> URL {
    let url = try temporaryDirectory().appendingPathComponent(UUID().uuidString + ".toml")
    try source.write(to: url, atomically: true, encoding: .utf8)
    return url
}

private func temporaryDirectory() throws -> URL {
    let url = FileManager.default.temporaryDirectory
        .appendingPathComponent("YidoSettingsTests")
        .appendingPathComponent(UUID().uuidString)
    try FileManager.default.createDirectory(at: url, withIntermediateDirectories: true)
    return url
}
