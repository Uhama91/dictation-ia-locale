fn main() {
    // Déclarer whisper_native comme cfg connu (évite les warnings unexpected_cfgs)
    println!("cargo:rustc-check-cfg=cfg(whisper_native)");

    // Lier whisper.cpp si compilé (Task 3 — scripts/build-whisper.sh)
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    link_whisper_cpp();

    // Compiler et lier le plugin Swift AccessibilityPaste (macOS uniquement)
    #[cfg(target_os = "macos")]
    compile_swift_plugins();

    generate_tray_translations();
    tauri_build::build()
}

/// Compile les plugins Swift (AccessibilityPaste.swift) en librairie statique
/// et les lie au binaire Rust.
///
/// Requis pour que `dlsym("accessibility_paste_text")` trouve le symbole au runtime.
/// Si swiftc echoue, le build continue — dlsym retournera null et le fallback Enigo prendra le relais.
#[cfg(target_os = "macos")]
fn compile_swift_plugins() {
    use std::path::Path;
    use std::process::Command;

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let swift_dir = Path::new(&manifest_dir).join("swift-plugin");
    let accessibility_swift = swift_dir.join("AccessibilityPaste.swift");
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let lib_path = format!("{out_dir}/libAccessibilityPaste.a");

    println!("cargo:rerun-if-changed=swift-plugin/AccessibilityPaste.swift");

    if !accessibility_swift.exists() {
        println!("cargo:warning=AccessibilityPaste.swift introuvable — collage Accessibility désactivé");
        return;
    }

    let status = Command::new("xcrun")
        .args([
            "swiftc",
            "-emit-library",
            "-static",
            "-o",
            &lib_path,
            accessibility_swift.to_str().unwrap(),
            "-module-name",
            "AccessibilityPaste",
        ])
        .status();

    match status {
        Ok(s) if s.success() => {
            println!("cargo:rustc-link-search=native={out_dir}");
            // Force-load the static library so the linker retains @_cdecl symbols
            // even though Rust never references them statically (we use dlsym at runtime).
            // Without this, the linker strips unreferenced symbols from the archive.
            println!("cargo:rustc-link-arg=-Wl,-force_load,{lib_path}");
            println!("cargo:rustc-link-lib=static=AccessibilityPaste");
            // Frameworks macOS requis par AccessibilityPaste.swift
            println!("cargo:rustc-link-lib=framework=Foundation");
            println!("cargo:rustc-link-lib=framework=ApplicationServices");
            println!("cargo:rustc-link-lib=framework=AppKit");
            // Swift runtime (requis pour les libs Swift statiques)
            // swiftc -print-target-info donne le chemin du runtime
            if let Ok(output) = Command::new("xcrun").args(["swiftc", "-print-target-info"]).output() {
                if let Ok(info) = String::from_utf8(output.stdout) {
                    // Extraire le runtimeLibraryPaths du JSON
                    if let Some(paths_start) = info.find("\"runtimeLibraryPaths\"") {
                        if let Some(bracket_start) = info[paths_start..].find('[') {
                            if let Some(bracket_end) = info[paths_start + bracket_start..].find(']') {
                                let paths_str = &info[paths_start + bracket_start + 1..paths_start + bracket_start + bracket_end];
                                for path in paths_str.split(',') {
                                    let path = path.trim().trim_matches('"').trim();
                                    if !path.is_empty() {
                                        println!("cargo:rustc-link-search=native={path}");
                                    }
                                }
                            }
                        }
                    }
                }
            }
            println!("cargo:warning=AccessibilityPaste.swift compilé — collage Accessibility activé");
        }
        Ok(s) => {
            println!(
                "cargo:warning=swiftc a échoué (exit code {:?}) — collage Accessibility désactivé, fallback Enigo",
                s.code()
            );
        }
        Err(e) => {
            println!("cargo:warning=swiftc introuvable ({e}) — collage Accessibility désactivé, fallback Enigo");
        }
    }
}

/// Lier libwhisper.a si disponible dans vendor/whisper.cpp/build/
///
/// Prérequis : exécuter scripts/build-whisper.sh d'abord.
/// Si la lib est absente, whisper_ffi.rs tombera sur les stubs (Err) au runtime.
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn link_whisper_cpp() {
    use std::path::Path;

    // Chemin attendu après scripts/build-whisper.sh
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let whisper_lib = Path::new(&manifest_dir)
        .parent()
        .unwrap()
        .join("vendor/whisper.cpp/build/src/libwhisper.a");
    let ggml_lib = Path::new(&manifest_dir)
        .parent()
        .unwrap()
        .join("vendor/whisper.cpp/build/ggml/src/libggml.a");

    println!("cargo:rerun-if-changed=../vendor/whisper.cpp/build/src/libwhisper.a");

    if whisper_lib.exists() && ggml_lib.exists() {
        let whisper_include = Path::new(&manifest_dir)
            .parent()
            .unwrap()
            .join("vendor/whisper.cpp/include");

        let ggml_include = Path::new(&manifest_dir)
            .parent()
            .unwrap()
            .join("vendor/whisper.cpp/ggml/include");

        // Compiler le wrapper C (évite de reproduire le layout de whisper_full_params en Rust)
        cc::Build::new()
            .file("src/whisper_wrapper.c")
            .include(&whisper_include)
            .include(&ggml_include)
            .compile("whisper_wrapper");

        // whisper-rs-sys (via transcribe-rs) expose aussi une libwhisper.a plus ancienne.
        // Pour éviter le conflit de noms, on fusionne toutes nos libs en une archive unique.
        let ggml_base_lib = Path::new(&manifest_dir)
            .parent()
            .unwrap()
            .join("vendor/whisper.cpp/build/ggml/src/libggml-base.a");
        let ggml_cpu_lib = Path::new(&manifest_dir)
            .parent()
            .unwrap()
            .join("vendor/whisper.cpp/build/ggml/src/libggml-cpu.a");
        let ggml_metal_lib = Path::new(&manifest_dir)
            .parent()
            .unwrap()
            .join("vendor/whisper.cpp/build/ggml/src/ggml-metal/libggml-metal.a");
        let ggml_blas_lib = Path::new(&manifest_dir)
            .parent()
            .unwrap()
            .join("vendor/whisper.cpp/build/ggml/src/ggml-blas/libggml-blas.a");

        let out_dir = std::env::var("OUT_DIR").unwrap();
        let merged = format!("{out_dir}/libwhisper_full.a");

        // Fusion des archives avec libtool (disponible sur macOS)
        let status = std::process::Command::new("libtool")
            .args(["-static", "-o", &merged])
            .arg(&whisper_lib)
            .arg(&ggml_lib)
            .arg(&ggml_base_lib)
            .arg(&ggml_cpu_lib)
            .arg(&ggml_metal_lib)
            .arg(&ggml_blas_lib)
            .status()
            .expect("libtool non trouvé");

        if !status.success() {
            panic!("libtool failed: impossible de fusionner les libs whisper+ggml");
        }

        println!("cargo:rustc-link-search=native={out_dir}");
        println!("cargo:rustc-link-lib=static=whisper_full");

        // Frameworks macOS requis par whisper.cpp/Metal
        println!("cargo:rustc-link-lib=framework=Foundation");
        println!("cargo:rustc-link-lib=framework=Metal");
        println!("cargo:rustc-link-lib=framework=MetalPerformanceShaders");
        println!("cargo:rustc-link-lib=framework=Accelerate");
        // CoreML (si compilé avec WHISPER_COREML=ON — non actif sans Xcode complet)
        println!("cargo:rustc-link-lib=framework=CoreML");

        println!("cargo:rustc-cfg=whisper_native");
        println!("cargo:warning=whisper.cpp natif activé (Metal — CoreML nécessite Xcode complet)");
    } else {
        println!(
            "cargo:warning=whisper.cpp non compilé — mode stub actif. \
             Exécuter scripts/build-whisper.sh pour activer CoreML+Metal."
        );
    }
}

/// Generate tray menu translations from frontend locale files.
///
/// Source of truth: src/i18n/locales/*/translation.json
/// The English "tray" section defines the struct fields.
fn generate_tray_translations() {
    use std::collections::BTreeMap;
    use std::fs;
    use std::path::Path;

    let out_dir = std::env::var("OUT_DIR").unwrap();
    let locales_dir = Path::new("../src/i18n/locales");

    println!("cargo:rerun-if-changed=../src/i18n/locales");

    // Collect all locale translations
    let mut translations: BTreeMap<String, serde_json::Value> = BTreeMap::new();

    for entry in fs::read_dir(locales_dir).unwrap().flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let lang = path.file_name().unwrap().to_str().unwrap().to_string();
        let json_path = path.join("translation.json");

        println!("cargo:rerun-if-changed={}", json_path.display());

        let content = fs::read_to_string(&json_path).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();

        if let Some(tray) = parsed.get("tray").cloned() {
            translations.insert(lang, tray);
        }
    }

    // English defines the schema
    let english = translations.get("en").unwrap().as_object().unwrap();
    let fields: Vec<_> = english
        .keys()
        .map(|k| (camel_to_snake(k), k.clone()))
        .collect();

    // Generate code
    let mut out = String::from(
        "// Auto-generated from src/i18n/locales/*/translation.json - do not edit\n\n",
    );

    // Struct
    out.push_str("#[derive(Debug, Clone)]\npub struct TrayStrings {\n");
    for (rust_field, _) in &fields {
        out.push_str(&format!("    pub {rust_field}: String,\n"));
    }
    out.push_str("}\n\n");

    // Static map
    out.push_str(
        "pub static TRANSLATIONS: Lazy<HashMap<&'static str, TrayStrings>> = Lazy::new(|| {\n",
    );
    out.push_str("    let mut m = HashMap::new();\n");

    for (lang, tray) in &translations {
        out.push_str(&format!("    m.insert(\"{lang}\", TrayStrings {{\n"));
        for (rust_field, json_key) in &fields {
            let val = tray.get(json_key).and_then(|v| v.as_str()).unwrap_or("");
            out.push_str(&format!(
                "        {rust_field}: \"{}\".to_string(),\n",
                escape_string(val)
            ));
        }
        out.push_str("    });\n");
    }

    out.push_str("    m\n});\n");

    fs::write(Path::new(&out_dir).join("tray_translations.rs"), out).unwrap();

    println!(
        "cargo:warning=Generated tray translations: {} languages, {} fields",
        translations.len(),
        fields.len()
    );
}

fn camel_to_snake(s: &str) -> String {
    s.chars()
        .enumerate()
        .fold(String::new(), |mut acc, (i, c)| {
            if c.is_uppercase() && i > 0 {
                acc.push('_');
            }
            acc.push(c.to_lowercase().next().unwrap());
            acc
        })
}

fn escape_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

