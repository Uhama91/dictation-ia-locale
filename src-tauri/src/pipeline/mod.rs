/// Pipeline de post-traitement hybride : règles locales + LLM conditionnel
///
/// Architecture (ADR-009) :
///   Transcription Whisper
///     ↓
///   [rules::apply] — toujours, < 1ms
///     ↓
///   confidence >= 0.85 ET words <= 30 ET mode Chat ? → retourner direct
///   Sinon → [cleanup::run] — Qwen2.5-0.5B Q4 via llama.cpp

pub mod modes;
pub mod orchestrator;
pub mod rules;
