import SwiftUI
import UIKit
import UniformTypeIdentifiers

/// A stack navigation container: NavigationStack on iOS 16, and on iOS 15 a NavigationView that behaves
/// the same (one column, pages pushed on top).
struct CompatNavigationStack<Content: View>: View {
    @ViewBuilder var content: () -> Content

    var body: some View {
        if #available(iOS 16.0, *) {
            NavigationStack(root: content)
        } else {
            NavigationView(content: content).navigationViewStyle(.stack)
        }
    }
}

/// The heights a sheet may take. A name of this module's own so the type exists on iOS 15, where
/// SwiftUI's PresentationDetent is not part of the SDK.
enum HorizonSheetSize { case medium, large, mediumLarge }

extension View {
    /// `presentationDetents`, iOS 16 only: on iOS 15 sheets are already full height.
    @ViewBuilder func horizonDetents(_ size: HorizonSheetSize) -> some View {
        if #available(iOS 16.0, *) {
            switch size {
            case .medium: self.presentationDetents([.medium])
            case .large: self.presentationDetents([.large])
            case .mediumLarge: self.presentationDetents([.medium, .large])
            }
        } else {
            self
        }
    }

    @ViewBuilder func horizonDragIndicator() -> some View {
        if #available(iOS 16.0, *) { self.presentationDragIndicator(.visible) } else { self }
    }

    /// `toolbarBackground(.hidden, for: .navigationBar)`, iOS 16 only.
    @ViewBuilder func horizonHiddenNavigationBarBackground() -> some View {
        if #available(iOS 16.0, *) { self.toolbarBackground(.hidden, for: .navigationBar) } else { self }
    }
}

/// ShareLink on iOS 16, and on iOS 15 a button that presents the system share sheet itself.
struct CompatShareLink<Content: View>: View {
    let item: URL
    @ViewBuilder var label: () -> Content

    var body: some View {
        if #available(iOS 16.0, *) {
            ShareLink(item: item, label: label)
        } else {
            Button { presentShareSheet(item) } label: { label() }
        }
    }
}

private func presentShareSheet(_ url: URL) {
    let windows = UIApplication.shared.connectedScenes
        .compactMap { $0 as? UIWindowScene }
        .flatMap(\.windows)
    guard let window = windows.first(where: \.isKeyWindow) ?? windows.first,
          var top = window.rootViewController else { return }
    while let presented = top.presentedViewController { top = presented }
    let sheet = UIActivityViewController(activityItems: [url], applicationActivities: nil)
    if let popover = sheet.popoverPresentationController {
        popover.sourceView = top.view
        popover.sourceRect = CGRect(x: top.view.bounds.midX, y: top.view.bounds.midY, width: 0, height: 0)
        popover.permittedArrowDirections = []
    }
    top.present(sheet, animated: true)
}

/// The Files picker, presented from the topmost view controller. SwiftUI's `fileImporter` draws the Files
/// screen on iOS 15 but a tapped document goes nowhere, so imports use UIKit's own picker.
enum DocumentPicker {
    private static let delegate = DocumentPickerDelegate()

    /// `asCopy` hands over a copy in the sandbox: no security scope to forget to open, and a cloud file is
    /// downloaded before the callback runs. `then` runs on the main thread with the chosen files, or an error.
    static func choose(types: [UTType], many: Bool = false,
                       _ then: @escaping (Result<[URL], Error>) -> Void) {
        DispatchQueue.main.async { present(types, many: many, then: then) }
    }

    private static func present(_ types: [UTType], many: Bool,
                                then: @escaping (Result<[URL], Error>) -> Void) {
        guard !delegate.waiting else { return }
        guard let host = gameViewController(), host.view.window != nil else {
            then(.failure(PickerError("The launcher is not on screen. Try again in a moment.")))
            return
        }
        let picker = UIDocumentPickerViewController(forOpeningContentTypes: types, asCopy: true)
        picker.allowsMultipleSelection = many
        picker.delegate = delegate
        delegate.waiting = true
        delegate.then = then
        LauncherLog.write("files picker: presented for \(types.map(\.identifier).joined(separator: ", "))")
        host.present(picker, animated: true)
    }
}

private final class DocumentPickerDelegate: NSObject, UIDocumentPickerDelegate {
    var waiting = false
    var then: ((Result<[URL], Error>) -> Void)?

    func documentPicker(_ controller: UIDocumentPickerViewController, didPickDocumentsAt urls: [URL]) {
        let then = finish()
        LauncherLog.write("files picker: chose \(urls.map(\.lastPathComponent).joined(separator: ", "))")
        controller.dismiss(animated: true)
        then?(.success(urls))
    }

    func documentPickerWasCancelled(_ controller: UIDocumentPickerViewController) {
        _ = finish()
        LauncherLog.write("files picker: cancelled")
    }

    private func finish() -> ((Result<[URL], Error>) -> Void)? {
        let then = then
        self.then = nil
        waiting = false
        return then
    }
}

private struct PickerError: LocalizedError {
    let errorDescription: String?
    init(_ reason: String) { errorDescription = reason }
}
