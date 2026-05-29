import Foundation

public struct InputEffect: Equatable, Sendable {
    public let committed: String
    public let composing: String
    public let handled: Bool

    public init(committed: String, composing: String, handled: Bool) {
        self.committed = committed
        self.composing = composing
        self.handled = handled
    }
}

public protocol InputEngine: AnyObject {
    func inputKey(_ key: String, shift: Bool) throws -> InputEffect
    func backspace() throws -> InputEffect
    func flush() throws -> InputEffect
    func cancel() throws -> InputEffect
}

public protocol TextClient: AnyObject {
    func insertText(_ text: String)
    func setMarkedText(_ text: String)
    func clearMarkedText()
}

public final class InputSession {
    private let engine: InputEngine

    public init(engine: InputEngine) {
        self.engine = engine
    }

    @discardableResult
    public func handlePrintableKey(_ key: String, shift: Bool, client: TextClient) throws -> Bool {
        try apply(engine.inputKey(key, shift: shift), to: client)
    }

    @discardableResult
    public func handleBackspace(client: TextClient) throws -> Bool {
        try apply(engine.backspace(), to: client)
    }

    @discardableResult
    public func flush(client: TextClient) throws -> Bool {
        try apply(engine.flush(), to: client)
    }

    @discardableResult
    public func cancel(client: TextClient) throws -> Bool {
        try apply(engine.cancel(), to: client)
    }

    @discardableResult
    private func apply(_ effect: InputEffect, to client: TextClient) -> Bool {
        if !effect.committed.isEmpty {
            client.insertText(effect.committed)
        }

        if effect.composing.isEmpty {
            client.clearMarkedText()
        } else {
            client.setMarkedText(effect.composing)
        }

        return effect.handled
    }
}
