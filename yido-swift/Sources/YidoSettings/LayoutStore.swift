import Foundation

public struct LayoutDescriptor: Equatable, Sendable {
    public let id: String
    public let name: String
    public let isBundled: Bool

    public init(id: String, name: String, isBundled: Bool) {
        self.id = id
        self.name = name
        self.isBundled = isBundled
    }
}

public enum LayoutStoreError: Error, Equatable {
    case duplicateLayoutID(String)
    case cannotRemoveBundledLayout(String)
    case missingLayoutMetadata
    case unsupportedEngine(String)
}

public final class LayoutStore {
    public static let bundledDubeolsikID = "ko-dubeolsik"

    private let bundledLayoutSource: String
    private let userLayoutsDirectory: URL
    private let fileManager: FileManager

    public init(
        bundledLayoutSource: String,
        userLayoutsDirectory: URL,
        fileManager: FileManager = .default
    ) {
        self.bundledLayoutSource = bundledLayoutSource
        self.userLayoutsDirectory = userLayoutsDirectory
        self.fileManager = fileManager
    }

    public func layouts() throws -> [LayoutDescriptor] {
        var descriptors = [try bundledDescriptor()]
        descriptors.append(contentsOf: try userLayoutDescriptors())
        return descriptors
    }

    @discardableResult
    public func importLayout(from sourceURL: URL) throws -> LayoutDescriptor {
        try fileManager.createDirectory(
            at: userLayoutsDirectory,
            withIntermediateDirectories: true
        )

        let source = try String(contentsOf: sourceURL, encoding: .utf8)
        let metadata = try LayoutMetadata.parse(source)
        try validate(metadata: metadata)

        let destination = userLayoutsDirectory.appendingPathComponent("\(metadata.id).toml")
        try source.write(to: destination, atomically: true, encoding: .utf8)

        return LayoutDescriptor(id: metadata.id, name: metadata.name, isBundled: false)
    }

    public func removeLayout(id: String) throws {
        if id == Self.bundledDubeolsikID {
            throw LayoutStoreError.cannotRemoveBundledLayout(id)
        }

        let destination = userLayoutsDirectory.appendingPathComponent("\(id).toml")
        if fileManager.fileExists(atPath: destination.path) {
            try fileManager.removeItem(at: destination)
        }
    }

    private func bundledDescriptor() throws -> LayoutDescriptor {
        let metadata = try LayoutMetadata.parse(bundledLayoutSource)
        return LayoutDescriptor(id: metadata.id, name: metadata.name, isBundled: true)
    }

    private func userLayoutDescriptors() throws -> [LayoutDescriptor] {
        guard fileManager.fileExists(atPath: userLayoutsDirectory.path) else {
            return []
        }

        return try fileManager
            .contentsOfDirectory(
                at: userLayoutsDirectory,
                includingPropertiesForKeys: nil
            )
            .filter { $0.pathExtension == "toml" }
            .sorted { $0.lastPathComponent < $1.lastPathComponent }
            .map { url in
                let source = try String(contentsOf: url, encoding: .utf8)
                let metadata = try LayoutMetadata.parse(source)
                return LayoutDescriptor(id: metadata.id, name: metadata.name, isBundled: false)
            }
    }

    private func validate(metadata: LayoutMetadata) throws {
        if metadata.id == Self.bundledDubeolsikID {
            throw LayoutStoreError.duplicateLayoutID(metadata.id)
        }

        if metadata.engine != "hangul" {
            throw LayoutStoreError.unsupportedEngine(metadata.engine)
        }

        let existingIDs = try Set(userLayoutDescriptors().map(\.id))
        if existingIDs.contains(metadata.id) {
            throw LayoutStoreError.duplicateLayoutID(metadata.id)
        }
    }
}

private struct LayoutMetadata {
    let id: String
    let name: String
    let engine: String

    static func parse(_ source: String) throws -> Self {
        var inLayoutSection = false
        var values: [String: String] = [:]

        for rawLine in source.split(separator: "\n", omittingEmptySubsequences: false) {
            let line = rawLine.trimmingCharacters(in: .whitespaces)
            if line == "[layout]" {
                inLayoutSection = true
                continue
            }

            if line.hasPrefix("[") {
                inLayoutSection = false
                continue
            }

            guard inLayoutSection, let separator = line.firstIndex(of: "=") else {
                continue
            }

            let key = line[..<separator].trimmingCharacters(in: .whitespaces)
            let value = line[line.index(after: separator)...]
                .trimmingCharacters(in: .whitespaces)
                .trimmingCharacters(in: CharacterSet(charactersIn: "\""))
            values[key] = value
        }

        guard
            let id = values["id"],
            let name = values["name"],
            let engine = values["engine"]
        else {
            throw LayoutStoreError.missingLayoutMetadata
        }

        return LayoutMetadata(id: id, name: name, engine: engine)
    }
}
