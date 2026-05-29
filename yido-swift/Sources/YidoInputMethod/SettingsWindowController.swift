import AppKit
import SwiftUI
import YidoCore

@MainActor
@objc(YidoPreferencesWindowController)
public final class YidoPreferencesWindowController: NSWindowController {
    public static let shared = YidoPreferencesWindowController()

    public convenience init() {
        self.init(window: Self.makeWindow())
    }

    override public init(window: NSWindow?) {
        super.init(window: window ?? Self.makeWindow())
        self.window?.isReleasedWhenClosed = false
    }

    @available(*, unavailable)
    required init?(coder: NSCoder) {
        fatalError("init(coder:) is not supported")
    }

    override public func showWindow(_ sender: Any?) {
        window?.center()
        window?.makeKeyAndOrderFront(sender)
        NSApp.activate(ignoringOtherApps: true)
    }

    private static func makeWindow() -> NSWindow {
        let view = SettingsView(layoutSettingsModel: LayoutSettingsModel(store: makeLayoutStore()))
        let hostingController = NSHostingController(rootView: view)
        let window = NSWindow(contentViewController: hostingController)
        window.title = "Yido Settings"
        window.styleMask = [.titled, .closable, .miniaturizable, .resizable]
        window.setContentSize(NSSize(width: 720, height: 460))
        return window
    }

    private static func makeLayoutStore() -> LayoutStore {
        LayoutStore(
            bundledLayoutSource: bundledLayoutSource(),
            userLayoutsDirectory: userLayoutsDirectory()
        )
    }

    private static func bundledLayoutSource() -> String {
        guard
            let url = Bundle.module.url(forResource: "ko-dubeolsik", withExtension: "toml"),
            let source = try? String(contentsOf: url, encoding: .utf8)
        else {
            return ""
        }

        return source
    }

    private static func userLayoutsDirectory() -> URL {
        FileManager.default
            .urls(for: .applicationSupportDirectory, in: .userDomainMask)
            .first!
            .appendingPathComponent("Yido")
            .appendingPathComponent("Layouts")
    }
}
