import { describe, it, expect, vi, beforeEach } from "vitest";
import { screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { renderWithI18n } from "@/test/helpers";
import ModelCard from "./ModelCard";
import type { ModelInfo } from "@/bindings";

// Minimal ModelInfo fixture for tests
const makeModel = (overrides: Partial<ModelInfo> = {}): ModelInfo => ({
  id: "test-model",
  name: "Test Model",
  description: "A test model",
  size_mb: "500",
  is_recommended: false,
  is_custom: false,
  is_downloaded: false,
  accuracy_score: 0.8,
  speed_score: 0.6,
  supported_languages: ["fr"],
  supports_translation: false,
  ...overrides,
});

describe("ModelCard", () => {
  const onSelect = vi.fn();
  const onDownload = vi.fn();
  const onRetry = vi.fn();
  const onDelete = vi.fn();
  const onCancel = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders model name and description", () => {
    renderWithI18n(
      <ModelCard model={makeModel()} onSelect={onSelect} />,
    );

    expect(screen.getByText("Test Model")).toBeInTheDocument();
    expect(screen.getByText("A test model")).toBeInTheDocument();
  });

  it("shows recommended badge when model is recommended", () => {
    renderWithI18n(
      <ModelCard
        model={makeModel({ is_recommended: true })}
        onSelect={onSelect}
      />,
    );

    expect(screen.getByText("Recommandé")).toBeInTheDocument();
  });

  it("calls onDownload when a downloadable model is clicked", async () => {
    renderWithI18n(
      <ModelCard
        model={makeModel()}
        status="downloadable"
        onSelect={onSelect}
        onDownload={onDownload}
      />,
    );

    await userEvent.click(screen.getByText("Test Model"));
    expect(onDownload).toHaveBeenCalledWith("test-model");
    expect(onSelect).not.toHaveBeenCalled();
  });

  it("shows download progress bar when downloading", () => {
    renderWithI18n(
      <ModelCard
        model={makeModel()}
        status="downloading"
        downloadProgress={45}
        downloadSpeed={2.5}
        onSelect={onSelect}
        onCancel={onCancel}
      />,
    );

    expect(screen.getByText("Téléchargement 45%")).toBeInTheDocument();
    expect(screen.getByText("2.5 Mo/s")).toBeInTheDocument();
    expect(screen.getByText("Annuler")).toBeInTheDocument();
  });

  it("shows error state with inline message and retry button", () => {
    renderWithI18n(
      <ModelCard
        model={makeModel()}
        status="error"
        onSelect={onSelect}
        onRetry={onRetry}
      />,
    );

    expect(
      screen.getByText(
        "Connexion internet nécessaire pour le premier téléchargement.",
      ),
    ).toBeInTheDocument();
    expect(screen.getByText("Réessayer")).toBeInTheDocument();
  });

  it("calls onRetry with model id when retry button is clicked", async () => {
    renderWithI18n(
      <ModelCard
        model={makeModel()}
        status="error"
        onSelect={onSelect}
        onRetry={onRetry}
      />,
    );

    await userEvent.click(screen.getByText("Réessayer"));
    expect(onRetry).toHaveBeenCalledWith("test-model");
  });

  it("does not render retry button when onRetry is not provided", () => {
    renderWithI18n(
      <ModelCard model={makeModel()} status="error" onSelect={onSelect} />,
    );

    expect(
      screen.getByText(
        "Connexion internet nécessaire pour le premier téléchargement.",
      ),
    ).toBeInTheDocument();
    expect(screen.queryByText("Réessayer")).not.toBeInTheDocument();
  });

  it("shows active badge when model is active", () => {
    renderWithI18n(
      <ModelCard
        model={makeModel()}
        status="active"
        onSelect={onSelect}
      />,
    );

    expect(screen.getByText("Actif")).toBeInTheDocument();
  });

  it("shows extracting state", () => {
    renderWithI18n(
      <ModelCard
        model={makeModel()}
        status="extracting"
        onSelect={onSelect}
      />,
    );

    expect(screen.getByText("Extraction...")).toBeInTheDocument();
  });

  it("calls onDelete when delete button is clicked on available model", async () => {
    renderWithI18n(
      <ModelCard
        model={makeModel()}
        status="available"
        onSelect={onSelect}
        onDelete={onDelete}
      />,
    );

    await userEvent.click(screen.getByText("Supprimer"));
    expect(onDelete).toHaveBeenCalledWith("test-model");
  });
});
