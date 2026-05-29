import CYidoFFI
import Darwin
import Foundation

public enum RustEngineError: Error, Equatable {
    case createFailed(String)
    case inputFailed(String)
    case missingEngine
    case libraryLoadFailed(String)
    case symbolLoadFailed(String)
}

public final class RustEngine: InputEngine {
    private let library: YidoFFILibrary
    private var engine: UnsafeMutablePointer<YidoEngine>?

    public init(layoutToml: String) throws {
        let library = try YidoFFILibrary.shared()
        self.library = library

        let result = layoutToml.withCString { library.engineNew($0) }
        defer { library.engineCreateResultFree(result) }

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
            library.engineFree(engine)
        }
    }

    public func inputKey(_ key: String, shift: Bool) throws -> InputEffect {
        try withEngine { engine in
            key.withCString { library.engineInputKey(engine, $0, shift) }
        }
    }

    public func backspace() throws -> InputEffect {
        try withEngine { library.engineBackspace($0) }
    }

    public func flush() throws -> InputEffect {
        try withEngine { library.engineFlush($0) }
    }

    public func cancel() throws -> InputEffect {
        try withEngine { library.engineCancel($0) }
    }

    private func withEngine(
        _ action: (UnsafeMutablePointer<YidoEngine>) -> YidoInputEffect
    ) throws -> InputEffect {
        guard let engine else {
            throw RustEngineError.missingEngine
        }

        let rawEffect = action(engine)
        defer { library.inputEffectFree(rawEffect) }

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

private final class YidoFFILibrary: @unchecked Sendable {
    typealias EngineNew = @convention(c) (UnsafePointer<CChar>) -> YidoEngineCreateResult
    typealias EngineInputKey = @convention(c) (UnsafeMutablePointer<YidoEngine>, UnsafePointer<CChar>, Bool) -> YidoInputEffect
    typealias EngineBackspace = @convention(c) (UnsafeMutablePointer<YidoEngine>) -> YidoInputEffect
    typealias EngineFlush = @convention(c) (UnsafeMutablePointer<YidoEngine>) -> YidoInputEffect
    typealias EngineCancel = @convention(c) (UnsafeMutablePointer<YidoEngine>) -> YidoInputEffect
    typealias EngineCreateResultFree = @convention(c) (YidoEngineCreateResult) -> Void
    typealias InputEffectFree = @convention(c) (YidoInputEffect) -> Void
    typealias EngineFree = @convention(c) (UnsafeMutablePointer<YidoEngine>) -> Void

    let engineNew: EngineNew
    let engineInputKey: EngineInputKey
    let engineBackspace: EngineBackspace
    let engineFlush: EngineFlush
    let engineCancel: EngineCancel
    let engineCreateResultFree: EngineCreateResultFree
    let inputEffectFree: InputEffectFree
    let engineFree: EngineFree

    private let handle: UnsafeMutableRawPointer
    private static let cached: Result<YidoFFILibrary, RustEngineError> = {
        do {
            return .success(try YidoFFILibrary())
        } catch let error as RustEngineError {
            return .failure(error)
        } catch {
            return .failure(.libraryLoadFailed(String(describing: error)))
        }
    }()

    static func shared() throws -> YidoFFILibrary {
        try cached.get()
    }

    private init() throws {
        let path = try Self.libraryPath()
        guard let handle = dlopen(path, RTLD_NOW | RTLD_LOCAL) else {
            throw RustEngineError.libraryLoadFailed(Self.lastDynamicLoaderError())
        }

        self.handle = handle
        engineNew = try Self.load("yido_engine_new", from: handle)
        engineInputKey = try Self.load("yido_engine_input_key", from: handle)
        engineBackspace = try Self.load("yido_engine_backspace", from: handle)
        engineFlush = try Self.load("yido_engine_flush", from: handle)
        engineCancel = try Self.load("yido_engine_cancel", from: handle)
        engineCreateResultFree = try Self.load("yido_engine_create_result_free", from: handle)
        inputEffectFree = try Self.load("yido_input_effect_free", from: handle)
        engineFree = try Self.load("yido_engine_free", from: handle)
    }

    deinit {
        dlclose(handle)
    }

    private static func libraryPath() throws -> String {
        if let bundled = Bundle.main.privateFrameworksURL?
            .appendingPathComponent("libyido.dylib")
            .path
        {
            return bundled
        }

        throw RustEngineError.libraryLoadFailed("libyido.dylib was not bundled with the app.")
    }

    private static func load<T>(_ symbol: String, from handle: UnsafeMutableRawPointer) throws -> T {
        guard let pointer = dlsym(handle, symbol) else {
            throw RustEngineError.symbolLoadFailed(symbol)
        }

        return unsafeBitCast(pointer, to: T.self)
    }

    private static func lastDynamicLoaderError() -> String {
        guard let error = dlerror() else {
            return "Unknown dynamic loader error"
        }

        return String(cString: error)
    }
}
