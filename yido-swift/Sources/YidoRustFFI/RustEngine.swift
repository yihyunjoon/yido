import CYidoFFI
import Foundation
import YidoInputCore

public enum RustEngineError: Error, Equatable {
    case createFailed(String)
    case inputFailed(String)
    case missingEngine
}

public final class RustEngine: InputEngine {
    private var engine: UnsafeMutablePointer<YidoEngine>?

    public init(layoutToml: String) throws {
        let result = layoutToml.withCString { yido_engine_new($0) }
        defer { yido_engine_create_result_free(result) }

        if let error = result.error {
            throw RustEngineError.createFailed(String(cString: error))
        }

        guard let engine = result.engine else {
            throw RustEngineError.missingEngine
        }

        self.engine = engine
    }

    deinit {
        if let engine {
            yido_engine_free(engine)
        }
    }

    public func inputKey(_ key: String, shift: Bool) throws -> InputEffect {
        try withEngine { engine in
            key.withCString { yido_engine_input_key(engine, $0, shift) }
        }
    }

    public func backspace() throws -> InputEffect {
        try withEngine { yido_engine_backspace($0) }
    }

    public func flush() throws -> InputEffect {
        try withEngine { yido_engine_flush($0) }
    }

    public func cancel() throws -> InputEffect {
        try withEngine { yido_engine_cancel($0) }
    }

    private func withEngine(
        _ action: (UnsafeMutablePointer<YidoEngine>) -> YidoInputEffect
    ) throws -> InputEffect {
        guard let engine else {
            throw RustEngineError.missingEngine
        }

        let rawEffect = action(engine)
        defer { yido_input_effect_free(rawEffect) }

        if let error = rawEffect.error {
            throw RustEngineError.inputFailed(String(cString: error))
        }

        return InputEffect(
            committed: string(from: rawEffect.committed),
            composing: string(from: rawEffect.composing),
            handled: rawEffect.handled
        )
    }

    private func string(from pointer: UnsafeMutablePointer<CChar>?) -> String {
        guard let pointer else {
            return ""
        }

        return String(cString: pointer)
    }
}
