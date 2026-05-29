enum SettingsSection: String, CaseIterable, Identifiable, Hashable {
    case layouts
    case about

    static let defaultSelection = SettingsSection.layouts

    var id: Self {
        self
    }

    var title: String {
        switch self {
        case .layouts:
            "Layouts"
        case .about:
            "About"
        }
    }

    var systemImage: String {
        switch self {
        case .layouts:
            "keyboard"
        case .about:
            "info.circle"
        }
    }
}
