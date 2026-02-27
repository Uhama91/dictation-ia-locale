import { describe, it, expect, vi, beforeEach } from "vitest";
import { screen, act, waitFor } from "@testing-library/react";
import { renderWithI18n } from "@/test/helpers";
import PrivacyBadge from "./PrivacyBadge";
import { listen } from "@tauri-apps/api/event";

// Helper: capture listen callbacks and expose them
function mockListenWithCapture() {
  const callbacks: Record<string, ((...args: unknown[]) => void)> = {};

  vi.mocked(listen).mockImplementation(async (event, handler) => {
    callbacks[event as string] = handler as (...args: unknown[]) => void;
    return () => {};
  });

  return callbacks;
}

describe("PrivacyBadge", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders in local state by default", () => {
    renderWithI18n(<PrivacyBadge />);

    expect(screen.getByText("Local")).toBeInTheDocument();
    expect(screen.queryByText("Téléchargement en cours")).not.toBeInTheDocument();
  });

  it("has green styling in local state", () => {
    renderWithI18n(<PrivacyBadge />);

    const badge = screen.getByText("Local").closest("div");
    expect(badge?.className).toContain("emerald");
  });

  it("registers listeners for download events", () => {
    renderWithI18n(<PrivacyBadge />);

    expect(listen).toHaveBeenCalledWith(
      "model-download-progress",
      expect.any(Function),
    );
    expect(listen).toHaveBeenCalledWith(
      "model-download-complete",
      expect.any(Function),
    );
    expect(listen).toHaveBeenCalledWith(
      "model-download-cancelled",
      expect.any(Function),
    );
  });

  it("switches to downloading state on model-download-progress event", async () => {
    const callbacks = mockListenWithCapture();

    renderWithI18n(<PrivacyBadge />);

    // Wait for listeners to be registered
    await waitFor(() => {
      expect(callbacks["model-download-progress"]).toBeDefined();
    });

    // Trigger download progress inside act
    act(() => {
      callbacks["model-download-progress"]();
    });

    expect(screen.getByText("Téléchargement en cours")).toBeInTheDocument();
    expect(screen.queryByText("Local")).not.toBeInTheDocument();

    const badge = screen.getByText("Téléchargement en cours").closest("div");
    expect(badge?.className).toContain("amber");
  });

  it("switches back to local state on model-download-complete event", async () => {
    const callbacks = mockListenWithCapture();

    renderWithI18n(<PrivacyBadge />);

    await waitFor(() => {
      expect(callbacks["model-download-progress"]).toBeDefined();
      expect(callbacks["model-download-complete"]).toBeDefined();
    });

    // Start downloading
    act(() => {
      callbacks["model-download-progress"]();
    });
    expect(screen.getByText("Téléchargement en cours")).toBeInTheDocument();

    // Complete download
    act(() => {
      callbacks["model-download-complete"]();
    });
    expect(screen.getByText("Local")).toBeInTheDocument();
  });

  it("switches back to local state on model-download-cancelled event", async () => {
    const callbacks = mockListenWithCapture();

    renderWithI18n(<PrivacyBadge />);

    await waitFor(() => {
      expect(callbacks["model-download-progress"]).toBeDefined();
      expect(callbacks["model-download-cancelled"]).toBeDefined();
    });

    // Start downloading
    act(() => {
      callbacks["model-download-progress"]();
    });
    expect(screen.getByText("Téléchargement en cours")).toBeInTheDocument();

    // Cancel download
    act(() => {
      callbacks["model-download-cancelled"]();
    });
    expect(screen.getByText("Local")).toBeInTheDocument();
  });
});
