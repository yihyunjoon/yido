import AppKit
import IMKSwift
import YidoInputMethod

@MainActor
final class YidoAppDelegate: NSObject, NSApplicationDelegate {
    private var server: IMKServer?

    func applicationDidFinishLaunching(_ notification: Notification) {
        _ = YidoInputController.self

        guard let bundleIdentifier = Bundle.main.bundleIdentifier else {
            fatalError("Yido input method bundle identifier is missing.")
        }

        server = IMKServer(
            name: "\(bundleIdentifier)_Connection",
            bundleIdentifier: bundleIdentifier
        )
    }
}

@main
struct YidoApp {
    @MainActor
    static func main() {
        let application = NSApplication.shared
        let delegate = YidoAppDelegate()
        application.delegate = delegate
        application.setActivationPolicy(.accessory)

        withExtendedLifetime(delegate) {
            application.run()
        }
    }
}
