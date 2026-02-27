import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { Info, X, ExternalLink } from "lucide-react";
import { openUrl } from "@tauri-apps/plugin-opener";
import { commands } from "@/bindings";
import { useSettingsStore } from "@/stores/settingsStore";

const OLLAMA_DISMISSED_KEY = "ollama-banner-dismissed";

const OllamaBanner: React.FC = () => {
  const { t } = useTranslation();
  const [visible, setVisible] = useState(false);
  const postProcessEnabled = useSettingsStore(
    (state) => state.settings?.post_process_enabled ?? false,
  );

  useEffect(() => {
    // Only show banner if post-processing is enabled
    if (!postProcessEnabled) {
      setVisible(false);
      return;
    }

    // Check if user previously dismissed
    const dismissed = localStorage.getItem(OLLAMA_DISMISSED_KEY);
    if (dismissed === "true") return;

    // Check Ollama status
    commands.checkOllamaStatus().then((available) => {
      if (!available) {
        setVisible(true);
      }
    });
  }, [postProcessEnabled]);

  const handleDismiss = () => {
    setVisible(false);
    localStorage.setItem(OLLAMA_DISMISSED_KEY, "true");
  };

  if (!visible) return null;

  return (
    <div className="w-full px-4 py-2.5 bg-amber-500/10 border-b border-amber-500/20 flex items-center gap-3">
      <Info className="w-4 h-4 text-amber-400 shrink-0" />
      <p className="text-xs text-amber-300 flex-1">
        {t("ollama.banner")}
      </p>
      <button
        onClick={() => openUrl("https://ollama.com")}
        className="flex items-center gap-1 text-xs text-amber-400 hover:text-amber-300 transition-colors shrink-0"
      >
        <ExternalLink className="w-3 h-3" />
        {t("ollama.link")}
      </button>
      <button
        onClick={handleDismiss}
        className="text-amber-400/50 hover:text-amber-400 transition-colors shrink-0"
        title={t("ollama.dismiss")}
      >
        <X className="w-3.5 h-3.5" />
      </button>
    </div>
  );
};

export default OllamaBanner;
