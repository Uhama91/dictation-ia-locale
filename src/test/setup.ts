import "@testing-library/jest-dom/vitest";
import { vi } from "vitest";

// =============================================================================
// Mock: @tauri-apps/api/event
// =============================================================================
vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(() => Promise.resolve(() => {})),
  emit: vi.fn(() => Promise.resolve()),
  once: vi.fn(() => Promise.resolve(() => {})),
}));

// =============================================================================
// Mock: @tauri-apps/api/core
// =============================================================================
vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
  convertFileSrc: vi.fn((path: string) => `asset://localhost/${path}`),
}));

// =============================================================================
// Mock: @tauri-apps/api/app
// =============================================================================
vi.mock("@tauri-apps/api/app", () => ({
  getVersion: vi.fn(() => Promise.resolve("0.7.8")),
  getName: vi.fn(() => Promise.resolve("DictAI")),
}));

// =============================================================================
// Mock: @tauri-apps/plugin-os
// =============================================================================
vi.mock("@tauri-apps/plugin-os", () => ({
  platform: vi.fn(() => "macos"),
  type: vi.fn(() => Promise.resolve("macos")),
  locale: vi.fn(() => Promise.resolve("fr-FR")),
}));

// =============================================================================
// Mock: @tauri-apps/plugin-opener
// =============================================================================
vi.mock("@tauri-apps/plugin-opener", () => ({
  openUrl: vi.fn(() => Promise.resolve()),
}));

// =============================================================================
// Mock: @tauri-apps/plugin-dialog
// =============================================================================
vi.mock("@tauri-apps/plugin-dialog", () => ({
  ask: vi.fn(() => Promise.resolve(true)),
  confirm: vi.fn(() => Promise.resolve(true)),
  message: vi.fn(() => Promise.resolve()),
}));

// =============================================================================
// Mock: @tauri-apps/plugin-fs
// =============================================================================
vi.mock("@tauri-apps/plugin-fs", () => ({
  readFile: vi.fn(() => Promise.resolve(new Uint8Array())),
  readTextFile: vi.fn(() => Promise.resolve("")),
  writeFile: vi.fn(() => Promise.resolve()),
  writeTextFile: vi.fn(() => Promise.resolve()),
  exists: vi.fn(() => Promise.resolve(false)),
}));

// =============================================================================
// Mock: @tauri-apps/plugin-updater
// =============================================================================
vi.mock("@tauri-apps/plugin-updater", () => ({
  check: vi.fn(() => Promise.resolve(null)),
}));

// =============================================================================
// Mock: @tauri-apps/plugin-process
// =============================================================================
vi.mock("@tauri-apps/plugin-process", () => ({
  relaunch: vi.fn(() => Promise.resolve()),
  exit: vi.fn(() => Promise.resolve()),
}));

// =============================================================================
// Mock: @tauri-apps/plugin-store
// =============================================================================
vi.mock("@tauri-apps/plugin-store", () => ({
  Store: vi.fn().mockImplementation(() => ({
    get: vi.fn(() => Promise.resolve(null)),
    set: vi.fn(() => Promise.resolve()),
    save: vi.fn(() => Promise.resolve()),
    delete: vi.fn(() => Promise.resolve()),
  })),
  load: vi.fn(() =>
    Promise.resolve({
      get: vi.fn(() => Promise.resolve(null)),
      set: vi.fn(() => Promise.resolve()),
      save: vi.fn(() => Promise.resolve()),
      delete: vi.fn(() => Promise.resolve()),
    }),
  ),
}));

// =============================================================================
// Mock: @tauri-apps/plugin-clipboard-manager
// =============================================================================
vi.mock("@tauri-apps/plugin-clipboard-manager", () => ({
  writeText: vi.fn(() => Promise.resolve()),
  readText: vi.fn(() => Promise.resolve("")),
}));

// =============================================================================
// Mock: @tauri-apps/plugin-global-shortcut
// =============================================================================
vi.mock("@tauri-apps/plugin-global-shortcut", () => ({
  register: vi.fn(() => Promise.resolve()),
  unregister: vi.fn(() => Promise.resolve()),
  isRegistered: vi.fn(() => Promise.resolve(false)),
}));

// =============================================================================
// Mock: @tauri-apps/plugin-autostart
// =============================================================================
vi.mock("@tauri-apps/plugin-autostart", () => ({
  enable: vi.fn(() => Promise.resolve()),
  disable: vi.fn(() => Promise.resolve()),
  isEnabled: vi.fn(() => Promise.resolve(false)),
}));

// =============================================================================
// Mock: @tauri-apps/plugin-sql
// =============================================================================
vi.mock("@tauri-apps/plugin-sql", () => ({
  default: {
    load: vi.fn(() =>
      Promise.resolve({
        execute: vi.fn(() => Promise.resolve()),
        select: vi.fn(() => Promise.resolve([])),
      }),
    ),
  },
}));

// =============================================================================
// Mock: tauri-plugin-macos-permissions-api
// =============================================================================
vi.mock("tauri-plugin-macos-permissions-api", () => ({
  checkAccessibilityPermission: vi.fn(() => Promise.resolve(true)),
  checkMicrophonePermission: vi.fn(() => Promise.resolve(true)),
  requestAccessibilityPermission: vi.fn(() => Promise.resolve(true)),
  requestMicrophonePermission: vi.fn(() => Promise.resolve(true)),
}));

// =============================================================================
// Mock: @/bindings (auto-generated Tauri specta bindings)
// =============================================================================
vi.mock("@/bindings", () => ({
  commands: {
    // Settings
    getSettings: vi.fn(() =>
      Promise.resolve({
        status: "ok",
        data: {
          post_process_enabled: false,
          debug_mode: false,
          language: "fr",
          write_mode: "chat",
        },
      }),
    ),
    updateSettings: vi.fn(() => Promise.resolve({ status: "ok", data: null })),

    // Models
    hasAnyModelsAvailable: vi.fn(() =>
      Promise.resolve({ status: "ok", data: true }),
    ),
    hasAnyModelsOrDownloads: vi.fn(() =>
      Promise.resolve({ status: "ok", data: true }),
    ),
    getAvailableModels: vi.fn(() =>
      Promise.resolve({ status: "ok", data: [] }),
    ),
    getModelInfo: vi.fn(() =>
      Promise.resolve({ status: "ok", data: null }),
    ),
    downloadModel: vi.fn(() =>
      Promise.resolve({ status: "ok", data: null }),
    ),
    cancelDownload: vi.fn(() =>
      Promise.resolve({ status: "ok", data: null }),
    ),
    deleteModel: vi.fn(() =>
      Promise.resolve({ status: "ok", data: null }),
    ),
    setActiveModel: vi.fn(() =>
      Promise.resolve({ status: "ok", data: null }),
    ),
    getCurrentModel: vi.fn(() =>
      Promise.resolve({ status: "ok", data: "" }),
    ),
    getTranscriptionModelStatus: vi.fn(() =>
      Promise.resolve({ status: "ok", data: null }),
    ),
    isModelLoading: vi.fn(() =>
      Promise.resolve({ status: "ok", data: false }),
    ),
    getModelLoadStatus: vi.fn(() =>
      Promise.resolve({ status: "ok", data: { loaded: false, model_id: null } }),
    ),

    // Ollama
    checkOllamaStatus: vi.fn(() => Promise.resolve(true)),

    // Audio
    getAudioDevices: vi.fn(() =>
      Promise.resolve({ status: "ok", data: [] }),
    ),
    getOutputDevices: vi.fn(() =>
      Promise.resolve({ status: "ok", data: [] }),
    ),
    getAvailableMicrophones: vi.fn(() =>
      Promise.resolve({ status: "ok", data: [] }),
    ),
    getAvailableOutputDevices: vi.fn(() =>
      Promise.resolve({ status: "ok", data: [] }),
    ),
    updateMicrophoneMode: vi.fn(() =>
      Promise.resolve({ status: "ok", data: null }),
    ),
    getMicrophoneMode: vi.fn(() =>
      Promise.resolve({ status: "ok", data: false }),
    ),

    // Pipeline
    initializeEnigo: vi.fn(() =>
      Promise.resolve({ status: "ok", data: null }),
    ),
    initializeShortcuts: vi.fn(() =>
      Promise.resolve({ status: "ok", data: null }),
    ),

    // History
    getHistoryEntries: vi.fn(() =>
      Promise.resolve({ status: "ok", data: [] }),
    ),
    toggleHistoryEntrySaved: vi.fn(() =>
      Promise.resolve({ status: "ok", data: null }),
    ),
    deleteHistoryEntry: vi.fn(() =>
      Promise.resolve({ status: "ok", data: null }),
    ),
    getAudioFilePath: vi.fn(() =>
      Promise.resolve({ status: "ok", data: "" }),
    ),
    updateHistoryLimit: vi.fn(() =>
      Promise.resolve({ status: "ok", data: null }),
    ),
    updateRecordingRetentionPeriod: vi.fn(() =>
      Promise.resolve({ status: "ok", data: null }),
    ),

    // App settings
    getAppSettings: vi.fn(() =>
      Promise.resolve({
        status: "ok",
        data: {
          post_process_enabled: false,
          debug_mode: false,
          app_language: "fr",
          write_mode: "chat",
        },
      }),
    ),
    getDefaultSettings: vi.fn(() =>
      Promise.resolve({ status: "ok", data: {} }),
    ),

    // Misc
    checkAccessibilityPermission: vi.fn(() => Promise.resolve(true)),
    checkAppleIntelligenceAvailable: vi.fn(() => Promise.resolve(false)),
    isRecording: vi.fn(() => Promise.resolve(false)),
    cancelOperation: vi.fn(() => Promise.resolve()),
    getAppDirPath: vi.fn(() =>
      Promise.resolve({ status: "ok", data: "/tmp/dictai" }),
    ),
    getLogDirPath: vi.fn(() =>
      Promise.resolve({ status: "ok", data: "/tmp/dictai/logs" }),
    ),
    playTestSound: vi.fn(() => Promise.resolve()),
    isLaptop: vi.fn(() =>
      Promise.resolve({ status: "ok", data: false }),
    ),
  },
  events: {},
}));

// =============================================================================
// Mock: matchMedia (required by some UI components)
// =============================================================================
Object.defineProperty(window, "matchMedia", {
  writable: true,
  value: vi.fn().mockImplementation((query: string) => ({
    matches: false,
    media: query,
    onchange: null,
    addListener: vi.fn(),
    removeListener: vi.fn(),
    addEventListener: vi.fn(),
    removeEventListener: vi.fn(),
    dispatchEvent: vi.fn(),
  })),
});
