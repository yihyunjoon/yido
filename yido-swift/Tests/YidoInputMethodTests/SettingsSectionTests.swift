import Testing

@testable import YidoInputMethod

@Test
func settingsSectionsExposeLayoutsFirst() {
    #expect(SettingsSection.defaultSelection == .layouts)
    #expect(SettingsSection.allCases == [.layouts, .about])
    #expect(SettingsSection.allCases.map(\.title) == ["Layouts", "About"])
}
