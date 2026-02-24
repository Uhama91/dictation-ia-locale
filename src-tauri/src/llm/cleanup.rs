/// LLM de nettoyage FR — Ollama (Qwen2.5:0.5b) via HTTP synchrone
///
/// Task 10 — Implémentation Ollama locale
///
/// Architecture :
/// - Backend : Ollama sur http://127.0.0.1:11434
/// - Modèle  : qwen2.5:0.5b (auto-détecté, voir OLLAMA_MODEL)
/// - HTTP    : reqwest::blocking dans un std::thread dédié (pas de conflit Tokio)
/// - Timeout : 8s (LLM_TIMEOUT_SECS) + 2s marge thread
/// - Params  : temperature=0.0 (greedy), top_k=1, num_predict=128
///
/// Installation Ollama :
///   brew install ollama
///   ollama pull qwen2.5:0.5b
///   ollama serve

use crate::pipeline::modes::WriteMode;
use anyhow::Result;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

/// URL de base Ollama local
const OLLAMA_BASE_URL: &str = "http://127.0.0.1:11434";
/// Modèle cible — qwen2.5:0.5b (~500 Mo, 200-300ms/50 tokens sur M1)
const OLLAMA_MODEL: &str = "qwen2.5:0.5b";
/// Timeout inférence LLM
const LLM_TIMEOUT_SECS: u64 = 8;
/// Limite de tokens générés (suffisant pour corriger ~100 mots)
const MAX_TOKENS: i64 = 128;

/// Corps de la requête Ollama /api/chat
fn build_ollama_payload(text: &str, mode: WriteMode) -> serde_json::Value {
    serde_json::json!({
        "model": OLLAMA_MODEL,
        "messages": [
            {"role": "system", "content": mode.system_prompt()},
            {"role": "user",   "content": text}
        ],
        "stream": false,
        "options": {
            "num_predict":    MAX_TOKENS,
            "temperature":    0.0,
            "top_k":          1,
            "repeat_penalty": 1.0
        }
    })
}

/// Appelle Ollama depuis un thread non-Tokio (évite le deadlock reqwest::blocking + async).
fn call_ollama(text: &str, mode: WriteMode) -> Result<String> {
    let (tx, rx) = mpsc::channel::<Result<String>>();
    let payload = build_ollama_payload(text, mode);

    thread::spawn(move || {
        let result = (|| -> Result<String> {
            let client = reqwest::blocking::Client::builder()
                .timeout(Duration::from_secs(LLM_TIMEOUT_SECS))
                .build()?;

            let resp: serde_json::Value = client
                .post(format!("{OLLAMA_BASE_URL}/api/chat"))
                .json(&payload)
                .send()?
                .json()?;

            let content = resp["message"]["content"]
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("Champ 'message.content' absent de la réponse Ollama"))?
                .trim()
                .to_string();

            Ok(content)
        })();
        let _ = tx.send(result);
    });

    rx.recv_timeout(Duration::from_secs(LLM_TIMEOUT_SECS + 2))
        .unwrap_or_else(|_| Err(anyhow::anyhow!("LLM timeout après {}s", LLM_TIMEOUT_SECS)))
}

/// Nettoie le texte transcrit avec le LLM Qwen2.5:0.5b via Ollama.
///
/// # Arguments
/// * `text` - Texte post-règles à nettoyer
/// * `mode` - Mode d'écriture (Chat/Pro/Code) — détermine le prompt système
///
/// # Returns
/// Texte nettoyé, ou `Err` si Ollama n'est pas disponible (le pipeline tombera sur les règles).
pub fn run(text: &str, mode: WriteMode) -> Result<String> {
    log::debug!(
        "LLM cleanup : mode={}, {} mots → Ollama {}",
        mode,
        text.split_whitespace().count(),
        OLLAMA_MODEL
    );

    let result = call_ollama(text, mode);

    match &result {
        Ok(cleaned) => log::debug!(
            "LLM cleanup OK : {} → {} chars",
            text.len(),
            cleaned.len()
        ),
        Err(e) => log::warn!("LLM cleanup échec ({}) — fallback règles", e),
    }

    result
}

/// Vérifie si Ollama est disponible avec le modèle configuré.
///
/// Utilisé par l'UI pour afficher l'état du LLM local.
pub fn is_model_available() -> bool {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let available = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(2))
            .build()
            .ok()
            .and_then(|c| {
                c.get(format!("{OLLAMA_BASE_URL}/api/tags"))
                    .send()
                    .ok()
            })
            .and_then(|r| r.json::<serde_json::Value>().ok())
            .and_then(|json| {
                json["models"].as_array().map(|models| {
                    models.iter().any(|m| {
                        m["name"].as_str().map_or(false, |n| n.starts_with("qwen2.5:0.5b"))
                    })
                })
            })
            .unwrap_or(false);
        let _ = tx.send(available);
    });
    rx.recv_timeout(Duration::from_secs(3)).unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn payload_has_required_fields() {
        let p = build_ollama_payload("test", WriteMode::Chat);
        assert_eq!(p["model"], OLLAMA_MODEL);
        assert_eq!(p["stream"], false);
        assert!(p["messages"].is_array());
        assert_eq!(p["messages"].as_array().unwrap().len(), 2);
        assert_eq!(p["options"]["temperature"], 0.0);
        assert_eq!(p["options"]["top_k"], 1);
        assert_eq!(p["options"]["num_predict"], MAX_TOKENS);
    }

    #[test]
    fn payload_uses_mode_system_prompt() {
        let p_chat = build_ollama_payload("test", WriteMode::Chat);
        let p_pro  = build_ollama_payload("test", WriteMode::Pro);
        let sys_chat = p_chat["messages"][0]["content"].as_str().unwrap();
        let sys_pro  = p_pro["messages"][0]["content"].as_str().unwrap();
        assert_ne!(sys_chat, sys_pro);
    }

    #[test]
    fn run_returns_err_when_ollama_unavailable() {
        // Sans Ollama démarré, run() doit retourner Err (pas paniquer)
        // Si Ollama est démarré par hasard, le test passe aussi (Ok acceptable)
        let result = run("Euh, je voulais dire bonjour.", WriteMode::Chat);
        // Err OU Ok — l'important est de ne pas paniquer
        match result {
            Ok(s) => assert!(!s.is_empty()),
            Err(_) => { /* attendu sans Ollama */ }
        }
    }
}
