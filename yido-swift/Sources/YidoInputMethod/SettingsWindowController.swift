import AppKit
import SwiftUI
import YidoSettings

@MainActor
final class SettingsWindowController {
    static let shared = SettingsWindowController()

    private var window: NSWindow?

    func show() {
        if let window {
            window.makeKeyAndOrderFront(nil)
            NSApp.activate()
            return
        }

        let view = LayoutSettingsView(model: LayoutSettingsModel(store: makeLayoutStore()))
        let hostingController = NSHostingController(rootView: view)
        let window = NSWindow(contentViewController: hostingController)
        window.title = "Yido Settings"
        window.styleMask = [.titled, .closable, .miniaturizable]
        window.isReleasedWhenClosed = false
        window.center()
        window.makeKeyAndOrderFront(nil)
        NSApp.activate()
        self.window = window
    }

    private func makeLayoutStore() -> LayoutStore {
        LayoutStore(
            bundledLayoutSource: bundledLayoutSource(),
            userLayoutsDirectory: userLayoutsDirectory()
        )
    }

    private func bundledLayoutSource() -> String {
        guard
            let url = Bundle.module.url(forResource: "ko-dubeolsik", withExtension: "toml"),
            let source = try? String(contentsOf: url, encoding: .utf8)
        else {
            return ""
        }

        return source
    }

    private func userLayoutsDirectory() -> URL {
        FileManager.default
            .urls(for: .applicationSupportDirectory, in: .userDomainMask)
            .first!
            .appendingPathComponent("Yido")
            .appendingPathComponent("Layouts")
    }
}
