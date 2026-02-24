/// WhisperANE.swift — Whisper CoreML encoder sur Apple Neural Engine
///
/// Task 6 — Implémentation complète à venir
///
/// Architecture prévue (ADR-007) :
/// - Utilise CoreML pour exécuter l'encoder Whisper sur ANE (~150ms vs ~500ms GPU)
/// - Le décodeur reste côté whisper.cpp/Metal
/// - Exposé via @_cdecl pour FFI depuis Rust (whisper_ffi.rs)
///
/// Alternative V2 : utiliser WhisperKit (argmaxinc/WhisperKit) qui gère
/// le pipeline complet ANE encoder + decoder
///
/// Prérequis :
/// - Fichiers CoreML compilés depuis whisper.cpp/models/generate_coreml_model.py
/// - Xcode 15+ avec CoreML framework
/// - macOS 13.0+ (minimum spécifié dans tauri.conf.json)
///
/// Usage depuis Rust (Task 4 — whisper_ffi.rs) :
/// ```rust
/// extern "C" {
///     fn whisper_coreml_encode(mel_data: *const f32, mel_len: usize,
///                               output: *mut f32, output_len: usize) -> i32;
/// }
/// ```

import Foundation
import CoreML

// TODO Task 6 : Implémenter WhisperCoreMLEncoder
// @_cdecl("whisper_coreml_encode")
// public func whisperCoreMLEncode(melData: UnsafePointer<Float>, melLen: Int,
//                                   output: UnsafeMutablePointer<Float>, outputLen: Int) -> Int32 {
//     // Charger le modèle CoreML
//     // Préparer les tenseurs d'entrée
//     // Exécuter l'inférence ANE
//     // Copier les résultats dans output
//     return 0 // 0 = succès
// }

// STUB : placeholder pour la compilation
@_cdecl("whisper_ane_available")
public func whisperANEAvailable() -> Bool {
    // Vérifier si ANE est disponible (Apple Silicon M1+)
    return ProcessInfo.processInfo.processorCount >= 1 // Placeholder
}
