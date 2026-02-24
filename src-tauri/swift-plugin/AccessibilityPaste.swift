/// AccessibilityPaste.swift — Collage direct au curseur via AXUIElement
///
/// Task 14 — Implémentation complète à venir
///
/// Remplace Enigo (simulation Cmd+V) par l'API Accessibility macOS native.
/// Avantage : collage direct sans interférence avec le presse-papier système.
///
/// Nécessite l'autorisation Accessibility dans System Preferences.
///
/// Exposé via @_cdecl pour FFI depuis Rust (input.rs Task 14)

import Foundation
import ApplicationServices
import AppKit

/// Colle le texte à la position du curseur dans l'application active.
///
/// Stratégie :
/// 1. Essayer AXUIElement (paste natif via Accessibility API)
/// 2. Fallback : presse-papier + simulation Cmd+V via CGEvent
@_cdecl("accessibility_paste_text")
public func accessibilityPasteText(_ text: UnsafePointer<CChar>) -> Int32 {
    let textString = String(cString: text)

    // Sauvegarder le contenu actuel du presse-papier
    let pasteboard = NSPasteboard.general
    let savedContents = pasteboard.string(forType: .string)

    // Copier le texte dans le presse-papier
    pasteboard.clearContents()
    pasteboard.setString(textString, forType: .string)

    // TODO Task 14 : Essayer d'abord AXUIElement
    // let focusedElement = getFocusedAXElement()
    // if let element = focusedElement {
    //     AXUIElementSetAttributeValue(element, kAXSelectedTextAttribute as CFString, textString as CFTypeRef)
    //     return 0
    // }

    // Fallback : simulation Cmd+V
    let cmdVDown = CGEvent(keyboardEventSource: nil, virtualKey: 0x09, keyDown: true)
    cmdVDown?.flags = .maskCommand
    cmdVDown?.post(tap: .cghidEventTap)

    let cmdVUp = CGEvent(keyboardEventSource: nil, virtualKey: 0x09, keyDown: false)
    cmdVUp?.flags = .maskCommand
    cmdVUp?.post(tap: .cghidEventTap)

    // Attendre que le coller soit effectué avant de restaurer le presse-papier
    DispatchQueue.main.asyncAfter(deadline: .now() + 0.1) {
        if let saved = savedContents {
            pasteboard.clearContents()
            pasteboard.setString(saved, forType: .string)
        }
    }

    return 0 // succès
}

/// Vérifie si les permissions Accessibility sont accordées
@_cdecl("accessibility_check_permission")
public func accessibilityCheckPermission() -> Bool {
    return AXIsProcessTrusted()
}
