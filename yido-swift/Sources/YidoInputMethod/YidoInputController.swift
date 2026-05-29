import AppKit
import IMKSwift
import YidoInputCore
import YidoRustFFI

@MainActor
@objc(YidoInputController)
public final class YidoInputController: IMKInputSessionController {
    private static let sessions = NSMapTable<AnyObject, InputSession>.weakToStrongObjects()

    private weak var session: InputSession?

    override public init(server: IMKServer, delegate: Any?, client inputClient: any IMKTextInput) {
        super.init(server: server, delegate: delegate, client: inputClient)
        session = Self.session(for: inputClient)
    }

    override public func handle(_ event: NSEvent?, client sender: any IMKTextInput) -> Bool {
        guard
            let event,
            let session,
            !event.modifierFlags.contains(.command),
            !event.modifierFlags.contains(.control),
            !event.modifierFlags.contains(.option)
        else {
            return false
        }

        let client = IMKTextClientAdapter(sender)

        do {
            switch event.keyCode {
            case 36, 76:
                _ = try session.flush(client: client)
                return false
            case 51:
                return try session.handleBackspace(client: client)
            case 53:
                return try session.cancel(client: client)
            default:
                guard let key = Self.keyForEngine(from: event) else {
                    return false
                }

                return try session.handlePrintableKey(
                    key,
                    shift: event.modifierFlags.contains(.shift),
                    client: client
                )
            }
        } catch {
            return false
        }
    }

    override public func inputText(_ string: String, client sender: any IMKTextInput) -> Bool {
        guard let session, string.count == 1 else {
            return false
        }

        do {
            return try session.handlePrintableKey(
                string.lowercased(),
                shift: false,
                client: IMKTextClientAdapter(sender)
            )
        } catch {
            return false
        }
    }

    override public func showPreferences(_ sender: (any IMKTextInput)?) {
        SettingsWindowController.shared.show()
    }

    public func reassign(session: InputSession) {
        self.session = session
    }

    private static func session(for client: any IMKTextInput) -> InputSession? {
        let key = client as AnyObject
        if let existing = sessions.object(forKey: key) {
            return existing
        }

        guard let session = makeDefaultSession() else {
            return nil
        }

        sessions.setObject(session, forKey: key)
        return session
    }

    private static func makeDefaultSession() -> InputSession? {
        guard
            let url = Bundle.module.url(forResource: "ko-dubeolsik", withExtension: "toml"),
            let source = try? String(contentsOf: url, encoding: .utf8),
            let engine = try? RustEngine(layoutToml: source)
        else {
            return nil
        }

        return InputSession(engine: engine)
    }

    private static func keyForEngine(from event: NSEvent) -> String? {
        guard
            let rawKey = event.charactersIgnoringModifiers,
            rawKey.count == 1
        else {
            return nil
        }

        if rawKey.range(of: #"^[A-Za-z]$"#, options: .regularExpression) != nil {
            return rawKey.lowercased()
        }

        return rawKey
    }
}

private final class IMKTextClientAdapter: TextClient {
    private let client: any IMKTextInput

    init(_ client: any IMKTextInput) {
        self.client = client
    }

    func insertText(_ text: String) {
        client.insertText(text, replacementRange: NSRange(location: NSNotFound, length: 0))
    }

    func setMarkedText(_ text: String) {
        client.setMarkedText(
            text,
            selectionRange: NSRange(location: text.utf16.count, length: 0),
            replacementRange: NSRange(location: NSNotFound, length: 0)
        )
    }

    func clearMarkedText() {
        client.setMarkedText(
            "",
            selectionRange: NSRange(location: 0, length: 0),
            replacementRange: NSRange(location: NSNotFound, length: 0)
        )
    }
}
