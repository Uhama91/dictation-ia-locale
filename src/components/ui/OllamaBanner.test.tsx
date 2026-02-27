import { describe, it, expect, vi, beforeEach } from "vitest";
import { screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { renderWithI18n } from "@/test/helpers";
import OllamaBanner from "./OllamaBanner";
import { commands } from "@/bindings";
import { useSettingsStore } from "@/stores/settingsStore";

// Helper to set post_process_enabled in the Zustand store
function setPostProcessEnabled(enabled: boolean) {
  useSettingsStore.setState({
    settings: {
      ...useSettingsStore.getState().settings,
      post_process_enabled: enabled,
    } as ReturnType<typeof useSettingsStore.getState>["settings"],
  });
}

describe("OllamaBanner", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    localStorage.clear();
    // Reset store to default (post_process disabled)
    useSettingsStore.setState({ settings: null });
  });

  it("does not render when post_process_enabled is false", () => {
    renderWithI18n(<OllamaBanner />);
    // Banner should not be visible — no ollama text in the DOM
    expect(screen.queryByRole("button")).toBeNull();
  });

  it("renders when post_process_enabled is true and Ollama is unavailable", async () => {
    vi.mocked(commands.checkOllamaStatus).mockResolvedValue(false);
    setPostProcessEnabled(true);

    renderWithI18n(<OllamaBanner />);

    await waitFor(() => {
      // The banner should now be visible — look for the dismiss button (X)
      const buttons = screen.getAllByRole("button");
      expect(buttons.length).toBeGreaterThanOrEqual(1);
    });
  });

  it("does not render when Ollama is available", async () => {
    vi.mocked(commands.checkOllamaStatus).mockResolvedValue(true);
    setPostProcessEnabled(true);

    renderWithI18n(<OllamaBanner />);

    // Wait a tick for the async check to complete
    await waitFor(() => {
      expect(commands.checkOllamaStatus).toHaveBeenCalled();
    });

    // Banner should still not be visible
    expect(screen.queryByRole("button")).toBeNull();
  });

  it("dismisses and persists to localStorage", async () => {
    vi.mocked(commands.checkOllamaStatus).mockResolvedValue(false);
    setPostProcessEnabled(true);

    renderWithI18n(<OllamaBanner />);

    await waitFor(() => {
      expect(screen.getAllByRole("button").length).toBeGreaterThanOrEqual(1);
    });

    // Find the dismiss button (last button with X icon)
    const buttons = screen.getAllByRole("button");
    const dismissButton = buttons[buttons.length - 1];
    await userEvent.click(dismissButton);

    // Banner should disappear
    await waitFor(() => {
      expect(screen.queryByRole("button")).toBeNull();
    });

    // localStorage should be set
    expect(localStorage.getItem("ollama-banner-dismissed")).toBe("true");
  });

  it("does not render when previously dismissed", async () => {
    localStorage.setItem("ollama-banner-dismissed", "true");
    vi.mocked(commands.checkOllamaStatus).mockResolvedValue(false);
    setPostProcessEnabled(true);

    renderWithI18n(<OllamaBanner />);

    // Wait for effect
    await waitFor(() => {
      expect(screen.queryByRole("button")).toBeNull();
    });
  });
});
