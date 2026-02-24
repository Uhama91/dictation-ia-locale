fn main() {
    // Déclarer whisper_native comme cfg connu (évite les warnings unexpected_cfgs)
    println!("cargo:rustc-check-cfg=cfg(whisper_native)");

    // Lier whisper.cpp si compilé (Task 3 — scripts/build-whisper.sh)
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    link_whisper_cpp();

    generate_tray_translations();
    tauri_build::build()
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
        let whisper_search = whisper_lib.parent().unwrap().display();
        let ggml_search = ggml_lib.parent().unwrap().display();

        println!("cargo:rustc-link-search=native={whisper_search}");
        println!("cargo:rustc-link-lib=static=whisper");
        println!("cargo:rustc-link-search=native={ggml_search}");
        println!("cargo:rustc-link-lib=static=ggml");

        // Frameworks macOS requis par whisper.cpp/Metal
        println!("cargo:rustc-link-lib=framework=Foundation");
        println!("cargo:rustc-link-lib=framework=Metal");
        println!("cargo:rustc-link-lib=framework=MetalPerformanceShaders");
        println!("cargo:rustc-link-lib=framework=Accelerate");
        // CoreML (si compilé avec WHISPER_COREML=ON)
        println!("cargo:rustc-link-lib=framework=CoreML");

        println!("cargo:rustc-cfg=whisper_native");
        println!("cargo:warning=whisper.cpp natif activé (CoreML + Metal)");
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

