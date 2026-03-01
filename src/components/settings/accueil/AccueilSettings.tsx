import React, { useState, useEffect, useCallback } from "react";
import { useTranslation } from "react-i18next";
import { Mic, MicOff } from "lucide-react";
import { listen } from "@tauri-apps/api/event";
import { WriteModeSelector } from "../WriteModeSelector";
import { ShortcutInput } from "../ShortcutInput";
import { SettingsGroup } from "../../ui/SettingsGroup";
import { commands, type HistoryEntry } from "@/bindings";
import { formatRelativeTime } from "@/utils/dateFormat";
import { WRITE_MODE_CONFIG, type WriteMode } from "@/config/writeModes";

const LastDictationPreview: React.FC = () => {
  const { t, i18n } = useTranslation();
  const [lastEntry, setLastEntry] = useState<HistoryEntry | null>(null);

  const loadLastEntry = useCallback(async () => {
    try {
      const result = await commands.getHistoryEntries();
      if (result.status === "ok" && result.data.length > 0) {
        setLastEntry(result.data[0]);
      }
    } catch (error) {
      console.error("Failed to load last history entry:", error);
    }
  }, []);

  useEffect(() => {
    let mounted = true;
    loadLastEntry();

    let unlisten: (() => void) | undefined;
    listen("history-updated", () => {
      if (mounted) loadLastEntry();
    }).then((fn) => {
      if (mounted) {
        unlisten = fn;
      } else {
        fn();
      }
    });

    return () => {
      mounted = false;
      unlisten?.();
    };
  }, [loadLastEntry]);

  if (!lastEntry) {
    return (
      <div className="px-4 py-3 text-center text-text/50 text-sm italic">
        {t("settings.history.empty")}
      </div>
    );
  }

  const modeConfig =
    lastEntry.write_mode && lastEntry.write_mode in WRITE_MODE_CONFIG
      ? WRITE_MODE_CONFIG[lastEntry.write_mode as WriteMode]
      : null;
  const displayText =
    lastEntry.post_processed_text ?? lastEntry.transcription_text;
  const relativeDate = formatRelativeTime(
    String(lastEntry.timestamp),
    i18n.language,
  );

  return (
    <div className="px-4 py-3 space-y-2">
      <div className="flex items-center gap-2">
        <span className="text-xs text-text/50">{relativeDate}</span>
        {modeConfig && lastEntry.write_mode && (
          <span
            className={`inline-flex items-center gap-1 px-1.5 py-0.5 rounded text-[10px] font-medium border ${modeConfig.badgeClassName}`}
          >
            <span>{modeConfig.emoji}</span>
            {t(
              `settings.general.writeMode.modes.${lastEntry.write_mode}.label`,
            )}
          </span>
        )}
      </div>
      <p className="text-sm text-text/80 italic line-clamp-3 select-text cursor-text">
        {displayText}
      </p>
    </div>
  );
};

const DictationStateIndicator: React.FC = () => {
  const { t } = useTranslation();
  const [isRecording, setIsRecording] = useState(false);
  const [isTranscribing, setIsTranscribing] = useState(false);

  useEffect(() => {
    let mounted = true;
    const unlisteners: (() => void)[] = [];

    const subscribe = (event: string, handler: () => void) => {
      listen(event, () => {
        if (mounted) handler();
      }).then((fn) => {
        if (mounted) {
          unlisteners.push(fn);
        } else {
          fn();
        }
      });
    };

    subscribe("recording-started", () => setIsRecording(true));
    subscribe("recording-stopped", () => setIsRecording(false));
    subscribe("transcription-started", () => setIsTranscribing(true));
    subscribe("transcription-complete", () => setIsTranscribing(false));

    return () => {
      mounted = false;
      unlisteners.forEach((fn) => fn());
    };
  }, []);

  const isActive = isRecording || isTranscribing;
  const statusText = isRecording
    ? t("overlay.transcribing")
    : isTranscribing
      ? t("overlay.processing")
      : t("sidebar.accueil");

  return (
    <div
      className={`flex items-center gap-3 px-4 py-3 rounded-lg border transition-colors ${
        isActive
          ? "border-logo-primary/40 bg-logo-primary/5"
          : "border-mid-gray/20 bg-background"
      }`}
    >
      <div
        className={`p-2 rounded-full ${isActive ? "bg-logo-primary/15 text-logo-primary" : "bg-mid-gray/10 text-text/50"}`}
      >
        {isActive ? <Mic size={20} /> : <MicOff size={20} />}
      </div>
      <span
        className={`text-sm font-medium ${isActive ? "text-logo-primary" : "text-text/70"}`}
      >
        {statusText}
      </span>
      {isActive && (
        <span className="ml-auto w-2 h-2 rounded-full bg-logo-primary animate-pulse" />
      )}
    </div>
  );
};

export const AccueilSettings: React.FC = () => {
  const { t } = useTranslation();

  return (
    <div className="max-w-3xl w-full mx-auto space-y-6">
      <DictationStateIndicator />

      <SettingsGroup title={t("settings.general.title")}>
        <ShortcutInput shortcutId="transcribe" grouped={true} />
        <WriteModeSelector descriptionMode="tooltip" grouped={true} />
      </SettingsGroup>

      <SettingsGroup title={t("sidebar.accueilLastDictation")}>
        <LastDictationPreview />
      </SettingsGroup>
    </div>
  );
};
