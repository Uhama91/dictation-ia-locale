import { describe, it, expect, vi, beforeEach } from "vitest";
import { screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { renderWithI18n } from "@/test/helpers";
import { HistorySettings } from "./HistorySettings";
import { commands, type HistoryEntry } from "@/bindings";

// Fixture: a single history entry
const makeEntry = (overrides: Partial<HistoryEntry> = {}): HistoryEntry => ({
  id: 1,
  file_name: "recording-001.wav",
  timestamp: Date.now() / 1000, // Unix seconds
  saved: false,
  title: "Test dictation",
  transcription_text: "Bonjour, ceci est un test de transcription.",
  post_processed_text: null,
  post_process_prompt: null,
  write_mode: "chat",
  ...overrides,
});

describe("HistorySettings", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    // Default: empty history, loaded
    vi.mocked(commands.getHistoryEntries).mockResolvedValue({
      status: "ok",
      data: [],
    });
  });

  it("shows loading state initially", () => {
    // Make getHistoryEntries hang (never resolve)
    vi.mocked(commands.getHistoryEntries).mockReturnValue(
      new Promise(() => {}),
    );

    renderWithI18n(<HistorySettings />);

    expect(screen.getByText("Chargement de l'historique...")).toBeInTheDocument();
  });

  it("shows empty state when no entries", async () => {
    renderWithI18n(<HistorySettings />);

    await waitFor(() => {
      expect(
        screen.getByText(
          "Pas encore de transcriptions. Commencez à enregistrer pour créer votre historique !",
        ),
      ).toBeInTheDocument();
    });
  });

  it("renders history entries with transcription text", async () => {
    vi.mocked(commands.getHistoryEntries).mockResolvedValue({
      status: "ok",
      data: [makeEntry()],
    });

    renderWithI18n(<HistorySettings />);

    await waitFor(() => {
      expect(
        screen.getByText("Bonjour, ceci est un test de transcription."),
      ).toBeInTheDocument();
    });
  });

  it("shows write mode badge for entries with write_mode", async () => {
    vi.mocked(commands.getHistoryEntries).mockResolvedValue({
      status: "ok",
      data: [makeEntry({ write_mode: "pro" })],
    });

    renderWithI18n(<HistorySettings />);

    await waitFor(() => {
      // The "pro" mode should show its emoji and label
      expect(screen.getByText("✉️")).toBeInTheDocument();
    });
  });

  it("shows post-processed text by default when available", async () => {
    vi.mocked(commands.getHistoryEntries).mockResolvedValue({
      status: "ok",
      data: [
        makeEntry({
          transcription_text: "texte brut original",
          post_processed_text: "Texte corrigé et amélioré.",
        }),
      ],
    });

    renderWithI18n(<HistorySettings />);

    await waitFor(() => {
      expect(
        screen.getByText("Texte corrigé et amélioré."),
      ).toBeInTheDocument();
      // Raw text should not be visible
      expect(
        screen.queryByText("texte brut original"),
      ).not.toBeInTheDocument();
    });
  });

  it("toggles between raw and processed text", async () => {
    vi.mocked(commands.getHistoryEntries).mockResolvedValue({
      status: "ok",
      data: [
        makeEntry({
          transcription_text: "texte brut original",
          post_processed_text: "Texte corrigé et amélioré.",
        }),
      ],
    });

    renderWithI18n(<HistorySettings />);

    await waitFor(() => {
      expect(screen.getByText("Texte corrigé et amélioré.")).toBeInTheDocument();
    });

    // Click to show raw text
    await userEvent.click(
      screen.getByText("Voir la transcription originale"),
    );

    expect(screen.getByText("texte brut original")).toBeInTheDocument();
    expect(
      screen.queryByText("Texte corrigé et amélioré."),
    ).not.toBeInTheDocument();

    // Click to show processed again
    await userEvent.click(screen.getByText("Voir le texte traité"));

    expect(screen.getByText("Texte corrigé et amélioré.")).toBeInTheDocument();
  });

  it("copies transcription text to clipboard when copy button is clicked", async () => {
    vi.mocked(commands.getHistoryEntries).mockResolvedValue({
      status: "ok",
      data: [makeEntry({ transcription_text: "Texte à copier" })],
    });

    renderWithI18n(<HistorySettings />);

    await waitFor(() => {
      expect(
        screen.getByTitle("Copier la transcription dans le presse-papiers"),
      ).toBeInTheDocument();
    });

    await userEvent.click(
      screen.getByTitle("Copier la transcription dans le presse-papiers"),
    );

    expect(navigator.clipboard.writeText).toHaveBeenCalledWith("Texte à copier");
  });

  it("copies post-processed text when available", async () => {
    vi.mocked(commands.getHistoryEntries).mockResolvedValue({
      status: "ok",
      data: [
        makeEntry({
          transcription_text: "texte brut",
          post_processed_text: "Texte amélioré",
        }),
      ],
    });

    renderWithI18n(<HistorySettings />);

    await waitFor(() => {
      expect(
        screen.getByTitle("Copier la transcription dans le presse-papiers"),
      ).toBeInTheDocument();
    });

    await userEvent.click(
      screen.getByTitle("Copier la transcription dans le presse-papiers"),
    );

    // Should copy the post-processed text, not the raw transcription
    expect(navigator.clipboard.writeText).toHaveBeenCalledWith("Texte amélioré");
  });

  it("calls toggleHistoryEntrySaved when star is clicked", async () => {
    vi.mocked(commands.getHistoryEntries).mockResolvedValue({
      status: "ok",
      data: [makeEntry({ id: 42 })],
    });
    vi.mocked(commands.toggleHistoryEntrySaved).mockResolvedValue({
      status: "ok",
      data: null,
    });

    renderWithI18n(<HistorySettings />);

    await waitFor(() => {
      expect(
        screen.getByTitle("Enregistrer la transcription"),
      ).toBeInTheDocument();
    });

    await userEvent.click(
      screen.getByTitle("Enregistrer la transcription"),
    );

    expect(commands.toggleHistoryEntrySaved).toHaveBeenCalledWith(42);
  });

  it("shows unsave title for saved entries", async () => {
    vi.mocked(commands.getHistoryEntries).mockResolvedValue({
      status: "ok",
      data: [makeEntry({ saved: true })],
    });

    renderWithI18n(<HistorySettings />);

    await waitFor(() => {
      expect(
        screen.getByTitle("Retirer des favoris"),
      ).toBeInTheDocument();
    });
  });

  it("calls deleteHistoryEntry when delete button is clicked", async () => {
    vi.mocked(commands.getHistoryEntries).mockResolvedValue({
      status: "ok",
      data: [makeEntry({ id: 99 })],
    });
    vi.mocked(commands.deleteHistoryEntry).mockResolvedValue({
      status: "ok",
      data: null,
    });

    renderWithI18n(<HistorySettings />);

    await waitFor(() => {
      expect(screen.getByTitle("Supprimer l'entrée")).toBeInTheDocument();
    });

    await userEvent.click(screen.getByTitle("Supprimer l'entrée"));

    expect(commands.deleteHistoryEntry).toHaveBeenCalledWith(99);
  });

  it("renders the title and open folder button", async () => {
    renderWithI18n(<HistorySettings />);

    await waitFor(() => {
      expect(screen.getByText("Historique")).toBeInTheDocument();
      expect(
        screen.getByText("Ouvrir le dossier des enregistrements"),
      ).toBeInTheDocument();
    });
  });

  it("calls openRecordingsFolder when button is clicked", async () => {
    vi.mocked(commands.openRecordingsFolder).mockResolvedValue({
      status: "ok",
      data: null,
    });

    renderWithI18n(<HistorySettings />);

    await waitFor(() => {
      expect(
        screen.getByText("Ouvrir le dossier des enregistrements"),
      ).toBeInTheDocument();
    });

    await userEvent.click(
      screen.getByText("Ouvrir le dossier des enregistrements"),
    );

    expect(commands.openRecordingsFolder).toHaveBeenCalled();
  });

  it("renders multiple entries", async () => {
    vi.mocked(commands.getHistoryEntries).mockResolvedValue({
      status: "ok",
      data: [
        makeEntry({ id: 1, transcription_text: "Première dictée" }),
        makeEntry({ id: 2, transcription_text: "Deuxième dictée" }),
        makeEntry({ id: 3, transcription_text: "Troisième dictée" }),
      ],
    });

    renderWithI18n(<HistorySettings />);

    await waitFor(() => {
      expect(screen.getByText("Première dictée")).toBeInTheDocument();
      expect(screen.getByText("Deuxième dictée")).toBeInTheDocument();
      expect(screen.getByText("Troisième dictée")).toBeInTheDocument();
    });
  });
});
