/// MenuBar.swift — NSStatusBar natif macOS
///
/// Task 13 — Implémentation complète
///
/// NSStatusItem avec icônes SF Symbols + animation d'enregistrement.
/// Piloté depuis Rust via FFI @_cdecl.
///
/// États supportés :
///   0 = idle       — "waveform" (prêt)
///   1 = recording  — "waveform.circle.fill" + pulse alpha
///   2 = processing — "ellipsis.circle" (transcription)
///   3 = error      — "exclamationmark.circle.fill" (rouge)
///
/// Intégration Rust (tray.rs) :
///   extern "C" {
///       fn menubar_init() -> i32;
///       fn menubar_set_state(state: i32) -> i32;
///       fn menubar_is_available() -> bool;
///   }

import Foundation
import AppKit

// MARK: - State

@objc public enum MenuBarState: Int {
    case idle       = 0
    case recording  = 1
    case processing = 2
    case error      = 3
}

// MARK: - Global state (main thread uniquement)

private var _statusItem: NSStatusItem?
private var _pulseTimer: Timer?

// MARK: - FFI Exports

/// Initialise le NSStatusItem. Doit être appelé depuis le main thread au démarrage.
///
/// Returns: 0 = succès, -1 = non main thread, -2 = erreur
@_cdecl("menubar_init")
public func menubarInit() -> Int32 {
    guard Thread.isMainThread else {
        // Forcer l'exécution sur le main thread si nécessaire
        DispatchQueue.main.sync { _ = menubarInit() }
        return 0
    }

    guard _statusItem == nil else { return 0 } // Déjà initialisé

    let item = NSStatusBar.system.statusItem(withLength: NSStatusItem.squareLength)
    item.button?.image = symbolImage(name: "waveform", size: 16)
    item.button?.image?.isTemplate = true
    item.button?.toolTip = "Dictation IA"
    _statusItem = item
    return 0
}

/// Met à jour l'état visuel de l'icône (thread-safe).
@_cdecl("menubar_set_state")
public func menubarSetState(_ state: Int32) -> Int32 {
    let barState = MenuBarState(rawValue: Int(state)) ?? .idle
    DispatchQueue.main.async { applyState(barState) }
    return 0
}

/// Indique si la menu bar native est disponible (toujours true sur macOS).
@_cdecl("menubar_is_available")
public func menubarIsAvailable() -> Bool { true }

// MARK: - State Application (main thread)

private func applyState(_ state: MenuBarState) {
    _pulseTimer?.invalidate()
    _pulseTimer = nil

    guard let button = _statusItem?.button else { return }

    switch state {
    case .idle:
        button.alphaValue = 1.0
        button.contentTintColor = nil
        button.image = symbolImage(name: "waveform", size: 16)
        button.image?.isTemplate = true

    case .recording:
        button.contentTintColor = nil
        button.image = symbolImage(name: "waveform.circle.fill", size: 16)
        button.image?.isTemplate = true
        // Pulse alpha pour indiquer l'enregistrement en cours
        _pulseTimer = Timer.scheduledTimer(withTimeInterval: 0.55, repeats: true) { _ in
            NSAnimationContext.runAnimationGroup { ctx in
                ctx.duration = 0.25
                button.animator().alphaValue = button.alphaValue > 0.5 ? 0.3 : 1.0
            }
        }
        RunLoop.current.add(_pulseTimer!, forMode: .common)

    case .processing:
        button.alphaValue = 0.6
        button.contentTintColor = nil
        button.image = symbolImage(name: "ellipsis.circle", size: 16)
        button.image?.isTemplate = true

    case .error:
        button.alphaValue = 1.0
        // Icône rouge — pas template (les images template ignorent la teinte)
        button.image = symbolImage(name: "exclamationmark.circle.fill", size: 16, color: .systemRed)
        button.image?.isTemplate = false
    }
}

// MARK: - Icon Helpers

/// Génère une icône SF Symbol au taille donnée.
/// Fallback sur un cercle plein si SF Symbols non disponible.
private func symbolImage(name: String, size: CGFloat, color: NSColor? = nil) -> NSImage? {
    let config = NSImage.SymbolConfiguration(pointSize: size, weight: .medium)

    if let image = NSImage(systemSymbolName: name, accessibilityDescription: nil)?
        .withSymbolConfiguration(config) {
        if let color = color {
            // Teinter l'image pour la couleur d'erreur
            let tinted = image.copy() as! NSImage
            tinted.lockFocus()
            color.set()
            NSRect(origin: .zero, size: tinted.size).fill(using: .sourceAtop)
            tinted.unlockFocus()
            return tinted
        }
        return image
    }

    // Fallback : cercle simple
    let dimension = size + 2
    let fallback = NSImage(size: NSSize(width: dimension, height: dimension), flipped: false) { rect in
        (color ?? NSColor.labelColor).setFill()
        NSBezierPath(ovalIn: rect.insetBy(dx: 2, dy: 2)).fill()
        return true
    }
    return fallback
}
