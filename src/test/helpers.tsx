import { render, type RenderOptions } from "@testing-library/react";
import { I18nextProvider } from "react-i18next";
import i18n from "i18next";
import { initReactI18next } from "react-i18next";
import type { ReactElement } from "react";

// Translations for tests — only FR (primary) and EN (fallback)
import fr from "../i18n/locales/fr/translation.json";
import en from "../i18n/locales/en/translation.json";

// Lightweight i18n instance for tests (no Tauri deps, no side-effects)
const testI18n = i18n.createInstance();
testI18n.use(initReactI18next).init({
  resources: {
    fr: { translation: fr },
    en: { translation: en },
  },
  lng: "fr",
  fallbackLng: "en",
  interpolation: { escapeValue: false },
  react: { useSuspense: false },
});

/**
 * Render a component wrapped with i18next provider (FR locale).
 * Usage: const { getByText } = renderWithI18n(<MyComponent />);
 */
export function renderWithI18n(
  ui: ReactElement,
  options?: Omit<RenderOptions, "wrapper">,
) {
  return render(ui, {
    wrapper: ({ children }) => (
      <I18nextProvider i18n={testI18n}>{children}</I18nextProvider>
    ),
    ...options,
  });
}

export { testI18n };
