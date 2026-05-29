import SwiftUI

struct SettingsView: View {
    @State private var selectedSection: SettingsSection? = SettingsSection.defaultSelection

    private let layoutSettingsModel: LayoutSettingsModel

    init(layoutSettingsModel: LayoutSettingsModel) {
        self.layoutSettingsModel = layoutSettingsModel
    }

    var body: some View {
        NavigationSplitView {
            List(SettingsSection.allCases, selection: $selectedSection) { section in
                NavigationLink(value: section) {
                    Label(section.title, systemImage: section.systemImage)
                }
            }
            .navigationSplitViewColumnWidth(170)
        } detail: {
            detailView
                .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
        }
        .frame(minWidth: 640, minHeight: 420)
        .onAppear {
            selectedSection = selectedSection ?? SettingsSection.defaultSelection
        }
    }

    @ViewBuilder
    private var detailView: some View {
        switch selectedSection ?? SettingsSection.defaultSelection {
        case .layouts:
            LayoutSettingsView(model: layoutSettingsModel)
        case .about:
            AboutSettingsView()
        }
    }
}

private struct AboutSettingsView: View {
    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("Yido")
                .font(.title2)
                .fontWeight(.semibold)

            Text("Version \(version)")
                .foregroundStyle(.secondary)

            Text(bundleIdentifier)
                .font(.caption)
                .foregroundStyle(.secondary)
        }
        .padding(24)
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
    }

    private var version: String {
        Bundle.main.object(forInfoDictionaryKey: "CFBundleShortVersionString") as? String ?? "0.1.0"
    }

    private var bundleIdentifier: String {
        Bundle.main.bundleIdentifier ?? "com.yido.inputmethod.Yido"
    }
}
