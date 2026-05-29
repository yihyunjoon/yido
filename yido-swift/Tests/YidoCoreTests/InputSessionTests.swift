import Testing

@testable import YidoCore

@Test
func appliesCommittedBeforeMarkedText() throws {
    let engine = MockEngine(effects: [
        InputEffect(committed: "한", composing: "ㄱ", handled: true),
    ])
    let client = MockTextClient()
    let session = InputSession(engine: engine)

    let handled = try session.handlePrintableKey("r", shift: false, client: client)

    #expect(handled)
    #expect(client.operations == [.insert("한"), .mark("ㄱ")])
}

@Test
func appliesCommitBeforeReturningUnhandled() throws {
    let engine = MockEngine(effects: [
        InputEffect(committed: "한", composing: "", handled: false),
    ])
    let client = MockTextClient()
    let session = InputSession(engine: engine)

    let handled = try session.handlePrintableKey("1", shift: false, client: client)

    #expect(!handled)
    #expect(client.operations == [.insert("한"), .clearMark])
}

@Test
func clearsMarkedTextWhenBackspaceRemovesPreedit() throws {
    let engine = MockEngine(effects: [
        InputEffect(committed: "", composing: "", handled: true),
    ])
    let client = MockTextClient()
    let session = InputSession(engine: engine)

    let handled = try session.handleBackspace(client: client)

    #expect(handled)
    #expect(client.operations == [.clearMark])
}

private final class MockEngine: InputEngine {
    private var effects: [InputEffect]

    init(effects: [InputEffect]) {
        self.effects = effects
    }

    func inputKey(_ key: String, shift: Bool) throws -> InputEffect {
        nextEffect()
    }

    func backspace() throws -> InputEffect {
        nextEffect()
    }

    func flush() throws -> InputEffect {
        nextEffect()
    }

    func cancel() throws -> InputEffect {
        nextEffect()
    }

    private func nextEffect() -> InputEffect {
        effects.removeFirst()
    }
}

private final class MockTextClient: TextClient {
    private(set) var operations: [TextOperation] = []

    func insertText(_ text: String) {
        operations.append(.insert(text))
    }

    func setMarkedText(_ text: String) {
        operations.append(.mark(text))
    }

    func clearMarkedText() {
        operations.append(.clearMark)
    }
}

private enum TextOperation: Equatable {
    case insert(String)
    case mark(String)
    case clearMark
}
