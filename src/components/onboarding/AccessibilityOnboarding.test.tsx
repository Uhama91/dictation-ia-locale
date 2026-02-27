import { describe, it, expect, vi, beforeEach } from "vitest";
import { screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { renderWithI18n } from "@/test/helpers";
import AccessibilityOnboarding from "./AccessibilityOnboarding";
import { platform } from "@tauri-apps/plugin-os";
import {
  checkAccessibilityPermission,
  checkMicrophonePermission,
  requestAccessibilityPermission,
  requestMicrophonePermission,
} from "tauri-plugin-macos-permissions-api";
import { openUrl } from "@tauri-apps/plugin-opener";

describe("AccessibilityOnboarding", () => {
  const onComplete = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
    vi.useFakeTimers({ shouldAdvanceTime: true });
    // Default: macOS, both permissions needed
    vi.mocked(platform).mockReturnValue("macos");
    vi.mocked(checkAccessibilityPermission).mockResolvedValue(false);
    vi.mocked(checkMicrophonePermission).mockResolvedValue(false);
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("skips immediately on non-macOS platforms", async () => {
    vi.mocked(platform).mockReturnValue("linux");

    renderWithI18n(<AccessibilityOnboarding onComplete={onComplete} />);

    await waitFor(() => {
      expect(onComplete).toHaveBeenCalledTimes(1);
    });
  });

  it("shows permission cards when permissions are needed on macOS", async () => {
    renderWithI18n(<AccessibilityOnboarding onComplete={onComplete} />);

    await waitFor(() => {
      expect(screen.getByText("Autoriser le microphone")).toBeInTheDocument();
      expect(screen.getByText("Autoriser l'accessibilité")).toBeInTheDocument();
    });
  });

  it("auto-completes when both permissions are already granted", async () => {
    vi.mocked(checkAccessibilityPermission).mockResolvedValue(true);
    vi.mocked(checkMicrophonePermission).mockResolvedValue(true);

    renderWithI18n(<AccessibilityOnboarding onComplete={onComplete} />);

    // Should show success briefly then call onComplete after timeout
    await waitFor(() => {
      expect(screen.getByText("Tout est prêt !")).toBeInTheDocument();
    });

    vi.advanceTimersByTime(500);

    await waitFor(() => {
      expect(onComplete).toHaveBeenCalled();
    });
  });

  it("shows denied state with 'Open Settings' button when microphone permission is denied", async () => {
    vi.mocked(requestMicrophonePermission).mockRejectedValue(
      new Error("denied"),
    );

    renderWithI18n(<AccessibilityOnboarding onComplete={onComplete} />);

    await waitFor(() => {
      expect(screen.getByText("Autoriser le microphone")).toBeInTheDocument();
    });

    await userEvent.click(screen.getByText("Autoriser le microphone"));

    await waitFor(() => {
      expect(
        screen.getByText(
          "Sans accès au microphone, DictAI ne pourra pas transcrire votre voix.",
        ),
      ).toBeInTheDocument();
      expect(
        screen.getByText("Ouvrir les Préférences Système"),
      ).toBeInTheDocument();
    });
  });

  it("shows denied state with 'Open Settings' button when accessibility permission is denied", async () => {
    vi.mocked(requestAccessibilityPermission).mockRejectedValue(
      new Error("denied"),
    );

    renderWithI18n(<AccessibilityOnboarding onComplete={onComplete} />);

    await waitFor(() => {
      expect(
        screen.getByText("Autoriser l'accessibilité"),
      ).toBeInTheDocument();
    });

    await userEvent.click(screen.getByText("Autoriser l'accessibilité"));

    await waitFor(() => {
      expect(
        screen.getByText(
          "Sans accès à l'accessibilité, DictAI ne pourra pas coller le texte dans vos applications.",
        ),
      ).toBeInTheDocument();
      expect(
        screen.getByText("Ouvrir les Préférences Système"),
      ).toBeInTheDocument();
    });
  });

  it("opens system preferences when 'Open Settings' is clicked (denied state)", async () => {
    vi.mocked(requestMicrophonePermission).mockRejectedValue(
      new Error("denied"),
    );

    renderWithI18n(<AccessibilityOnboarding onComplete={onComplete} />);

    await waitFor(() => {
      expect(screen.getByText("Autoriser le microphone")).toBeInTheDocument();
    });

    await userEvent.click(screen.getByText("Autoriser le microphone"));

    await waitFor(() => {
      expect(
        screen.getByText("Ouvrir les Préférences Système"),
      ).toBeInTheDocument();
    });

    await userEvent.click(screen.getByText("Ouvrir les Préférences Système"));

    expect(openUrl).toHaveBeenCalledWith(
      "x-apple.systempreferences:com.apple.preference.security?Privacy_Microphone",
    );
  });

  it("does NOT show 'Continue without permissions' for new users", async () => {
    renderWithI18n(
      <AccessibilityOnboarding
        onComplete={onComplete}
        isReturningUser={false}
      />,
    );

    await waitFor(() => {
      expect(screen.getByText("Autoriser le microphone")).toBeInTheDocument();
    });

    expect(
      screen.queryByText("Continuer sans ces permissions"),
    ).not.toBeInTheDocument();
  });

  it("shows 'Continue without permissions' for returning users", async () => {
    renderWithI18n(
      <AccessibilityOnboarding onComplete={onComplete} isReturningUser />,
    );

    await waitFor(() => {
      expect(
        screen.getByText("Continuer sans ces permissions"),
      ).toBeInTheDocument();
    });
  });

  it("calls onComplete when returning user clicks 'Continue without permissions'", async () => {
    renderWithI18n(
      <AccessibilityOnboarding onComplete={onComplete} isReturningUser />,
    );

    await waitFor(() => {
      expect(
        screen.getByText("Continuer sans ces permissions"),
      ).toBeInTheDocument();
    });

    await userEvent.click(screen.getByText("Continuer sans ces permissions"));
    expect(onComplete).toHaveBeenCalledTimes(1);
  });
});
