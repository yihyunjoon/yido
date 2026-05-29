import AppKit
import IMKSwift
import YidoInputCore

@MainActor
@objc(YidoInputController)
public final class YidoInputController: IMKInputSessionController {
    private weak var session: InputSession?

    override public init(server: IMKServer, delegate: Any?, client inputClient: any IMKTextInput) {
        super.init(server: server, delegate: delegate, client: inputClient)
    }

    override public func handle(_ event: NSEvent?, client sender: any IMKTextInput) -> Bool {
        guard event != nil else {
            return false
        }

        return false
    }

    override public func inputText(_ string: String, client sender: any IMKTextInput) -> Bool {
        guard !string.isEmpty else {
            return false
        }

        return false
    }

    public func reassign(session: InputSession) {
        self.session = session
    }
}
