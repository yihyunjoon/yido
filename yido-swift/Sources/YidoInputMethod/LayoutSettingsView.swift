import SwiftUI
import UniformTypeIdentifiers
import YidoCore

struct LayoutSettingsView: View {
    @StateObject private var model: LayoutSettingsModel
    @State private var isImporting = false

    init(model: LayoutSettingsModel) {
        _model = StateObject(wrappedValue: model)
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 16) {
            HStack {
                Text("Layouts")
                    .font(.title2)
                    .fontWeight(.semibold)

                Spacer()

                Button {
                    isImporting = true
                } label: {
                    Label("Import", systemImage: "square.and.arrow.down")
                }

                Button {
                    model.removeSelectedLayout()
                } label: {
                    Label("Remove", systemImage: "trash")
                }
                .disabled(!model.canRemoveSelectedLayout)
            }

            List(selection: $model.selectedLayoutID) {
                ForEach(model.layouts, id: \.id) { layout in
                    HStack {
                        VStack(alignment: .leading) {
                            Text(layout.name)
                            Text(layout.id)
                                .font(.caption)
                                .foregroundStyle(.secondary)
                        }
                        Spacer()
                        if layout.isBundled {
                            Text("Built-in")
                                .font(.caption)
                                .foregroundStyle(.secondary)
                        }
                    }
                    .tag(layout.id)
                }
            }
            .frame(minHeight: 260)

            if let message = model.errorMessage {
                Text(message)
                    .font(.callout)
                    .foregroundStyle(.red)
            }
        }
        .padding(24)
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
        .onAppear {
            model.reload()
        }
        .fileImporter(
            isPresented: $isImporting,
            allowedContentTypes: [.item],
            allowsMultipleSelection: false
        ) { result in
            model.importLayout(result)
        }
    }
}

@MainActor
final class LayoutSettingsModel: ObservableObject {
    @Published private(set) var layouts: [LayoutDescriptor] = []
    @Published var selectedLayoutID: String?
    @Published private(set) var errorMessage: String?

    private let store: LayoutStore

    init(store: LayoutStore) {
        self.store = store
    }

    var canRemoveSelectedLayout: Bool {
        guard let selectedLayoutID else {
            return false
        }

        return layouts.contains { $0.id == selectedLayoutID && !$0.isBundled }
    }

    func reload() {
        do {
            layouts = try store.layouts()
            selectedLayoutID = selectedLayoutID ?? layouts.first?.id
            errorMessage = nil
        } catch {
            errorMessage = Self.message(for: error)
        }
    }

    func importLayout(_ result: Result<[URL], Error>) {
        do {
            guard let url = try result.get().first else {
                return
            }

            let descriptor = try store.importLayout(from: url)
            selectedLayoutID = descriptor.id
            reload()
        } catch {
            errorMessage = Self.message(for: error)
        }
    }

    func removeSelectedLayout() {
        guard let selectedLayoutID else {
            return
        }

        do {
            try store.removeLayout(id: selectedLayoutID)
            self.selectedLayoutID = nil
            reload()
        } catch {
            errorMessage = Self.message(for: error)
        }
    }

    private static func message(for error: Error) -> String {
        switch error {
        case LayoutStoreError.duplicateLayoutID(let id):
            "Layout id already exists: \(id)"
        case LayoutStoreError.cannotRemoveBundledLayout:
            "The built-in Dubeolsik layout cannot be removed."
        case LayoutStoreError.missingLayoutMetadata:
            "The selected file does not contain valid layout metadata."
        case LayoutStoreError.unsupportedEngine(let engine):
            "Unsupported layout engine: \(engine)"
        default:
            error.localizedDescription
        }
    }
}
