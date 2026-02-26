/// accessibility.rs — Pont FFI vers AccessibilityPaste.swift
///
/// Utilise dlsym pour charger les symboles Swift a l'execution.
/// Cela evite les erreurs de linkage pendant `cargo test` ou `cargo bench`,
/// car le Swift plugin n'est compile et linke que par `tauri build` / `tauri dev`.
///
/// Codes retour de accessibility_paste_text :
///   0 = succes via AXUIElement (aucun presse-papier touche)
///   1 = succes via Cmd+V fallback (presse-papier temporairement utilise)
///  -1 = echec total

#[cfg(target_os = "macos")]
use std::ffi::CString;
#[cfg(target_os = "macos")]
use std::os::raw::c_char;

/// Colle le texte a la position du curseur via Accessibility API (macOS).
///
/// Returns:
///   Ok(0) = AXUIElement direct (~2ms, apps Cocoa)
///   Ok(1) = Cmd+V fallback (~50ms, apps Electron)
///   Err   = echec total, symbole absent, ou plateforme non macOS
#[cfg(target_os = "macos")]
pub fn paste_via_accessibility(text: &str) -> Result<i32, String> {
    let c_text = CString::new(text).map_err(|e| format!("Invalid text for FFI: {}", e))?;

    // Resolve the symbol dynamically to avoid link-time errors in test builds
    let sym = unsafe { libc::dlsym(libc::RTLD_DEFAULT, b"accessibility_paste_text\0".as_ptr() as *const _) };
    if sym.is_null() {
        return Err("accessibility_paste_text symbol not found (Swift plugin not linked)".into());
    }

    let func: unsafe extern "C" fn(*const c_char) -> i32 = unsafe { std::mem::transmute(sym) };
    let result = unsafe { func(c_text.as_ptr()) };

    if result >= 0 {
        Ok(result)
    } else {
        Err("Accessibility paste failed (both AX and Cmd+V)".into())
    }
}

/// Verifie si les permissions Accessibility sont accordees.
#[cfg(target_os = "macos")]
pub fn check_permission() -> bool {
    let sym = unsafe { libc::dlsym(libc::RTLD_DEFAULT, b"accessibility_check_permission\0".as_ptr() as *const _) };
    if sym.is_null() {
        return false;
    }
    let func: unsafe extern "C" fn() -> bool = unsafe { std::mem::transmute(sym) };
    unsafe { func() }
}

/// Demande les permissions Accessibility (affiche la dialog systeme macOS).
#[cfg(target_os = "macos")]
pub fn request_permission() {
    let sym = unsafe { libc::dlsym(libc::RTLD_DEFAULT, b"accessibility_request_permission\0".as_ptr() as *const _) };
    if sym.is_null() {
        return;
    }
    let func: unsafe extern "C" fn() = unsafe { std::mem::transmute(sym) };
    unsafe { func() }
}

#[cfg(not(target_os = "macos"))]
pub fn paste_via_accessibility(_text: &str) -> Result<i32, String> {
    Err("Accessibility paste is only available on macOS".into())
}

#[cfg(not(target_os = "macos"))]
pub fn check_permission() -> bool {
    false
}

#[cfg(not(target_os = "macos"))]
pub fn request_permission() {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn cstring_rejects_null_bytes() {
        let text_with_null = "hello\0world";
        assert!(CString::new(text_with_null).is_err());
    }

    #[test]
    fn cstring_accepts_empty_string() {
        assert!(CString::new("").is_ok());
    }

    #[test]
    fn cstring_accepts_french_unicode() {
        let text = "J'ai ecrit un texte avec des accents : ecoute, ca marche !";
        assert!(CString::new(text).is_ok());
    }

    #[test]
    fn cstring_accepts_long_text() {
        let text = "Je voudrais vous dire que cette application de dictee vocale fonctionne vraiment tres bien merci.";
        assert!(CString::new(text).is_ok());
    }

    /// Simulates the return code logic from paste_via_accessibility
    fn interpret_ffi_result(code: i32) -> Result<i32, String> {
        if code >= 0 {
            Ok(code)
        } else {
            Err("Accessibility paste failed (both AX and Cmd+V)".into())
        }
    }

    #[test]
    fn ffi_result_code_0_is_ax_success() {
        assert_eq!(interpret_ffi_result(0).unwrap(), 0);
    }

    #[test]
    fn ffi_result_code_1_is_cmdv_fallback() {
        assert_eq!(interpret_ffi_result(1).unwrap(), 1);
    }

    #[test]
    fn ffi_result_code_negative_is_error() {
        assert!(interpret_ffi_result(-1).is_err());
    }

    #[test]
    fn paste_gracefully_handles_missing_symbol() {
        // In test builds, the Swift library isn't linked.
        // paste_via_accessibility should return Err, not panic.
        let result = paste_via_accessibility("test");
        // On macOS without Swift: symbol not found. On other platforms: not available.
        assert!(result.is_err());
    }

    #[test]
    fn check_permission_returns_false_without_swift() {
        // Without Swift library linked, should return false (safe default)
        let result = check_permission();
        assert!(!result);
    }
}
