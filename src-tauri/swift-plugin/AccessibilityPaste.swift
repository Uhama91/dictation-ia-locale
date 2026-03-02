/// AccessibilityPaste.swift — Collage direct au curseur via AXUIElement
///
/// Task 14 — Implémentation complète
///
/// Stratégie (2 niveaux de fallback) :
/// 1. AXUIElement kAXSelectedTextAttribute — insère directement à la position
///    du curseur sans toucher le presse-papier (~2ms, apps Cocoa natives)
/// 2. Presse-papier + CGEvent Cmd+V — universelle, préserve le contenu
///    du presse-papier en sauvegardant/restaurant (~50ms)
///
/// Compatible : TextEdit, Notes, Mail, Safari, terminaux, Slack, VS Code...
/// Non compatible avec certaines apps games/protection qui bloquent AX.
///
/// Exposé via @_cdecl pour FFI depuis Rust (utils.rs / clipboard.rs)

import Foundation
import ApplicationServices
import AppKit

// MARK: - Target App Tracking

/// App cible mémorisée avant l'affichage de l'overlay DictAI.
/// Permet de réactiver la bonne app au moment du collage, même si
/// `run_on_main_thread` (Tauri) a momentanément rendu DictAI frontmost.
private var savedTargetApp: NSRunningApplication? = nil

/// Sauvegarde l'application frontmost actuelle comme cible du prochain collage.
/// Doit être appelé juste avant d'afficher l'overlay d'enregistrement.
@_cdecl("accessibility_save_target_app")
public func accessibilitySaveTargetApp() {
    savedTargetApp = NSWorkspace.shared.frontmostApplication
}

// MARK: - Public FFI API

/// Colle le texte à la position du curseur dans l'application active.
///
/// Returns:
///   0 = succès via AXUIElement (aucun presse-papier touché)
///   1 = succès via Cmd+V fallback (presse-papier temporairement utilisé)
///  -1 = échec total
@_cdecl("accessibility_paste_text")
public func accessibilityPasteText(_ text: UnsafePointer<CChar>) -> Int32 {
    let textString = String(cString: text)
    guard !textString.isEmpty else { return 0 }

    // Tentative 1 : AXUIElement (direct, sans presse-papier)
    if tryAXPaste(text: textString) {
        return 0
    }

    // Tentative 2 : Cmd+V avec sauvegarde/restauration presse-papier
    pasteViaClipboard(text: textString)
    return 1
}

/// Vérifie si les permissions Accessibility sont accordées.
@_cdecl("accessibility_check_permission")
public func accessibilityCheckPermission() -> Bool {
    return AXIsProcessTrusted()
}

/// Demande les permissions Accessibility (affiche la dialog système macOS).
@_cdecl("accessibility_request_permission")
public func accessibilityRequestPermission() {
    let options = [kAXTrustedCheckOptionPrompt.takeRetainedValue() as String: true] as CFDictionary
    AXIsProcessTrustedWithOptions(options)
}

// MARK: - AXUIElement Strategy

/// Tente de coller via AXUIElement kAXSelectedTextAttribute.
///
/// Fonctionne avec les apps qui exposent cet attribut en écriture :
/// - TextEdit, Notes, Mail, Pages, Numbers
/// - Safari (zones de texte web)
/// - Terminal, iTerm2
///
/// Ne fonctionne généralement PAS avec :
/// - Apps Electron (VS Code, Slack, Discord) — utilisent leur propre renderer
/// - Apps sandboxées sans entitlement com.apple.security.automation.apple-events
private func tryAXPaste(text: String) -> Bool {
    guard AXIsProcessTrusted() else { return false }

    // Récupérer l'élément UI qui a le focus clavier
    let systemWide = AXUIElementCreateSystemWide()
    var focusedRef: CFTypeRef?

    let copyResult = AXUIElementCopyAttributeValue(
        systemWide,
        kAXFocusedUIElementAttribute as CFString,
        &focusedRef
    )

    guard copyResult == .success, let focusedRef = focusedRef else {
        return false
    }

    // swiftlint:disable:next force_cast
    let focused = focusedRef as! AXUIElement

    // Vérifier que l'attribut est accessible en écriture
    var settable: DarwinBoolean = false
    let checkResult = AXUIElementIsAttributeSettable(
        focused,
        kAXSelectedTextAttribute as CFString,
        &settable
    )

    guard checkResult == .success, settable.boolValue else {
        return false
    }

    // Lire la longueur du contenu avant insertion (pour vérification)
    var beforeRef: CFTypeRef?
    AXUIElementCopyAttributeValue(focused, kAXValueAttribute as CFString, &beforeRef)
    let beforeLen = (beforeRef as? String)?.count ?? -1

    // Insérer le texte (remplace la sélection, ou insère au curseur si vide)
    let setResult = AXUIElementSetAttributeValue(
        focused,
        kAXSelectedTextAttribute as CFString,
        text as CFTypeRef
    )

    guard setResult == .success else { return false }

    // Vérification : Chrome/Electron retournent .success mais ignorent l'écriture.
    // On compare la longueur du contenu avant/après pour détecter ce faux positif.
    if beforeLen >= 0 {
        var afterRef: CFTypeRef?
        AXUIElementCopyAttributeValue(focused, kAXValueAttribute as CFString, &afterRef)
        let afterLen = (afterRef as? String)?.count ?? -1

        if afterLen >= 0 && afterLen <= beforeLen {
            NSLog("[DictAI-paste] AXUIElement reported success but text not inserted (before=%d, after=%d) — falling through to Cmd+V", beforeLen, afterLen)
            return false
        }
    }

    return true
}

// MARK: - Clipboard + Cmd+V Fallback

/// Presse-papier + simulation Cmd+V — fallback universel.
///
/// Sauvegarde le contenu du presse-papier avant le collage et le restaure
/// 150ms après pour minimiser l'impact sur le workflow de l'utilisateur.
///
/// Réactive explicitement l'app cible avant le Cmd+V pour les apps comme
/// Chrome/IDX où le Monaco editor peut perdre son focus interne quand
/// l'overlay DictAI apparaît.
private func pasteViaClipboard(text: String) {
    let pasteboard = NSPasteboard.general
    let savedChangeCount = pasteboard.changeCount

    // Sauvegarder le contenu existant (string uniquement pour l'instant)
    let savedString = pasteboard.string(forType: .string)

    // Écrire le texte à coller
    pasteboard.clearContents()
    pasteboard.setString(text, forType: .string)

    // Réactiver l'app cible mémorisée avant l'overlay (ou frontmost en fallback).
    // Critique pour Chrome/Electron : run_on_main_thread (Tauri) peut rendre DictAI
    // frontmost momentanément, ce qui ferait diriger le CGEvent vers DictAI.
    let targetApp = savedTargetApp ?? NSWorkspace.shared.frontmostApplication
    let myPid = ProcessInfo.processInfo.processIdentifier
    if let app = targetApp, app.processIdentifier != myPid {
        NSLog("[DictAI-paste] Activating target app: %@ (pid %d)", app.localizedName ?? "?", app.processIdentifier)
        app.activate(options: .activateIgnoringOtherApps)
        Thread.sleep(forTimeInterval: 0.15) // 150ms — Chrome/Electron a besoin de temps pour refocaliser le renderer
    } else {
        NSLog("[DictAI-paste] WARNING: target app is self or nil — savedTarget=%@, frontmost=%@",
              savedTargetApp?.localizedName ?? "nil",
              NSWorkspace.shared.frontmostApplication?.localizedName ?? "nil")
    }

    // Simuler Cmd+V (keyCode 9 = 'v' sur toutes les dispositions de clavier US/FR)
    let source = CGEventSource(stateID: .hidSystemState)
    if let keyDown = CGEvent(keyboardEventSource: source, virtualKey: 0x09, keyDown: true),
       let keyUp   = CGEvent(keyboardEventSource: source, virtualKey: 0x09, keyDown: false) {
        keyDown.flags = .maskCommand
        keyUp.flags   = .maskCommand
        keyDown.post(tap: .cghidEventTap)
        keyUp.post(tap: .cghidEventTap)
        NSLog("[DictAI-paste] CGEvent Cmd+V posted to HID stream")
    }

    // Restaurer le presse-papier après un délai généreux.
    // Chrome/Electron lit le clipboard de manière asynchrone (IPC renderer → main → clipboard).
    // 150ms était trop court pour des apps web complexes comme IDX.
    let delayNs = DispatchTimeInterval.milliseconds(1000)
    DispatchQueue.global(qos: .userInteractive).asyncAfter(deadline: .now() + delayNs) {
        // Ne restaurer que si le presse-papier n'a pas été modifié par une
        // autre application entre-temps (changeCount +1 = notre propre modification)
        guard pasteboard.changeCount == savedChangeCount + 1 else { return }

        if let saved = savedString {
            pasteboard.clearContents()
            pasteboard.setString(saved, forType: .string)
        } else {
            pasteboard.clearContents()
        }
    }
}
