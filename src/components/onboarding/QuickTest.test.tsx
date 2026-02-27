import { describe, it, expect, vi, beforeEach } from "vitest";
import { screen, act, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { renderWithI18n } from "@/test/helpers";
import QuickTest from "./QuickTest";
import { listen } from "@tauri-apps/api/event";

describe("QuickTest", () => {
  const onComplete = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders the idle state with start button and skip", () => {
    renderWithI18n(<QuickTest onComplete={onComplete} />);

    expect(screen.getByText("Test rapide")).toBeInTheDocument();
    expect(
      screen.getByText("Vérifiez que tout fonctionne en dictant une courte phrase."),
    ).toBeInTheDocument();
    expect(screen.getByText("Commencer le test")).toBeInTheDocument();
    expect(screen.getByText("Passer")).toBeInTheDocument();
  });

  it("calls onComplete when skip button is clicked", async () => {
    renderWithI18n(<QuickTest onComplete={onComplete} />);

    await userEvent.click(screen.getByText("Passer"));
    expect(onComplete).toHaveBeenCalledTimes(1);
  });

  it("transitions to recording state when start is clicked", async () => {
    renderWithI18n(<QuickTest onComplete={onComplete} />);

    await userEvent.click(screen.getByText("Commencer le test"));

    expect(screen.getByText("Enregistrement...")).toBeInTheDocument();
    expect(screen.queryByText("Commencer le test")).not.toBeInTheDocument();
  });

  it("registers listeners for transcription-result and transcription-error", async () => {
    renderWithI18n(<QuickTest onComplete={onComplete} />);

    await userEvent.click(screen.getByText("Commencer le test"));

    expect(listen).toHaveBeenCalledWith(
      "transcription-result",
      expect.any(Function),
    );
    expect(listen).toHaveBeenCalledWith(
      "transcription-error",
      expect.any(Function),
    );
  });

  it("shows result and continue button when transcription completes", async () => {
    // Capture the callback for transcription-result
    let resultCallback: ((event: { payload: string }) => void) | null = null;
    vi.mocked(listen).mockImplementation(async (event, handler) => {
      if (event === "transcription-result") {
        resultCallback = handler as (event: { payload: string }) => void;
      }
      return () => {};
    });

    renderWithI18n(<QuickTest onComplete={onComplete} />);
    await userEvent.click(screen.getByText("Commencer le test"));

    // Wait for listener to be registered
    await waitFor(() => {
      expect(resultCallback).not.toBeNull();
    });

    // Simulate transcription result inside act
    act(() => {
      resultCallback?.({ payload: "Bonjour le monde" });
    });

    expect(screen.getByText("Bonjour le monde")).toBeInTheDocument();
    expect(screen.getByText("Tout fonctionne parfaitement !")).toBeInTheDocument();
    expect(screen.getByText("Continuer")).toBeInTheDocument();
    expect(screen.queryByText("Passer")).not.toBeInTheDocument();
  });

  it("calls onComplete when continue button is clicked after success", async () => {
    let resultCallback: ((event: { payload: string }) => void) | null = null;
    vi.mocked(listen).mockImplementation(async (event, handler) => {
      if (event === "transcription-result") {
        resultCallback = handler as (event: { payload: string }) => void;
      }
      return () => {};
    });

    renderWithI18n(<QuickTest onComplete={onComplete} />);
    await userEvent.click(screen.getByText("Commencer le test"));

    await waitFor(() => {
      expect(resultCallback).not.toBeNull();
    });

    act(() => {
      resultCallback?.({ payload: "Test réussi" });
    });

    await userEvent.click(screen.getByText("Continuer"));
    expect(onComplete).toHaveBeenCalledTimes(1);
  });
});
