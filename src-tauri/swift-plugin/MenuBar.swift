/// MenuBar.swift — NSStatusBar natif macOS
///
/// Task 13 — Implémentation complète à venir
///
/// Remplace la menu bar Tauri (artificielle) par NSStatusBar natif.
/// Avantage : comportement macOS correct, icône template, animation fluide.
///
/// Exposé via @_cdecl pour FFI depuis Rust (tray.rs adaptation Task 13)

import Foundation
import AppKit

// États de l'icône menu bar
@objc public enum MenuBarState: Int {
    case idle = 0
    case recording = 1
    case processing = 2
    case error = 3
}

// TODO Task 13 : Implémenter NSStatusItem
// Cette implémentation sera activée quand la FFI Rust↔Swift sera en place

/// Placeholder — la menu bar native sera implémentée en Task 13
/// En attendant, la menu bar Tauri (tray.rs) reste active
@_cdecl("menubar_set_state")
public func menubarSetState(_ state: Int32) -> Int32 {
    // TODO Task 13 : Mettre à jour l'icône NSStatusItem selon l'état
    // let menuBarState = MenuBarState(rawValue: Int(state)) ?? .idle
    // statusItem?.button?.image = iconForState(menuBarState)
    return 0 // succès
}

@_cdecl("menubar_is_available")
public func menubarIsAvailable() -> Bool {
    return true // macOS toujours disponible
}
