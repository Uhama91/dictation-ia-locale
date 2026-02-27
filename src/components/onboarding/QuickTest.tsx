import { useState } from "react";
import { useTranslation } from "react-i18next";
import { Mic, Loader2, Check, SkipForward } from "lucide-react";
import { listen } from "@tauri-apps/api/event";
import { commands } from "@/bindings";

interface QuickTestProps {
  onComplete: () => void;
}

type TestState = "idle" | "recording" | "processing" | "done";

const QuickTest: React.FC<QuickTestProps> = ({ onComplete }) => {
  const { t } = useTranslation();
  const [state, setState] = useState<TestState>("idle");
  const [result, setResult] = useState<string>("");

  const handleStartTest = async () => {
    setState("recording");

    try {
      // Listen for transcription result
      const unlisten = await listen<string>("transcription-result", (event) => {
        setResult(event.payload);
        setState("done");
        unlisten();
      });

      // Also listen for errors
      const unlistenError = await listen<string>(
        "transcription-error",
        () => {
          setState("idle");
          unlistenError();
        },
      );

      // Trigger recording via the shortcut system — simulate pressing the transcribe shortcut
      // The recording will be handled by the existing TranscribeAction
      await commands.initializeEnigo();
      await commands.initializeShortcuts();
    } catch (error) {
      console.error("Quick test failed:", error);
      setState("idle");
    }
  };

  return (
    <div className="h-screen w-screen flex flex-col items-center justify-center gap-6 p-6">
      <div className="text-center">
        <h2 className="text-xl font-semibold text-text mb-2">
          {t("onboarding.quickTest.title")}
        </h2>
        <p className="text-text/70">
          {t("onboarding.quickTest.description")}
        </p>
      </div>

      <div className="flex flex-col items-center gap-4 max-w-md w-full">
        {state === "idle" && (
          <button
            onClick={handleStartTest}
            className="flex items-center gap-3 px-6 py-4 rounded-xl bg-logo-primary hover:bg-logo-primary/90 text-white font-medium transition-all hover:scale-[1.02] active:scale-[0.98]"
          >
            <Mic className="w-5 h-5" />
            {t("onboarding.quickTest.startButton")}
          </button>
        )}

        {state === "recording" && (
          <div className="flex items-center gap-3 text-logo-primary">
            <div className="relative">
              <Mic className="w-8 h-8" />
              <span className="absolute -top-1 -right-1 w-3 h-3 bg-red-500 rounded-full animate-pulse" />
            </div>
            <span className="text-lg font-medium">
              {t("onboarding.quickTest.recording")}
            </span>
          </div>
        )}

        {state === "processing" && (
          <div className="flex items-center gap-3 text-text/70">
            <Loader2 className="w-6 h-6 animate-spin" />
            <span>{t("onboarding.quickTest.processing")}</span>
          </div>
        )}

        {state === "done" && result && (
          <div className="w-full flex flex-col items-center gap-4">
            <div className="w-full p-4 rounded-lg bg-white/5 border border-emerald-500/20">
              <p className="text-xs text-text/50 mb-1">
                {t("onboarding.quickTest.result")}
              </p>
              <p className="text-text">{result}</p>
            </div>
            <div className="flex items-center gap-2 text-emerald-400">
              <Check className="w-5 h-5" />
              <span className="font-medium">
                {t("onboarding.quickTest.success")}
              </span>
            </div>
          </div>
        )}
      </div>

      <div className="flex items-center gap-3 mt-4">
        {state === "done" ? (
          <button
            onClick={onComplete}
            className="px-6 py-3 rounded-lg bg-logo-primary hover:bg-logo-primary/90 text-white font-medium transition-colors"
          >
            {t("onboarding.quickTest.continue")}
          </button>
        ) : (
          <button
            onClick={onComplete}
            className="flex items-center gap-2 px-4 py-2 rounded-lg text-text/50 hover:text-text/70 hover:bg-white/5 transition-colors"
          >
            <SkipForward className="w-4 h-4" />
            {t("onboarding.quickTest.skip")}
          </button>
        )}
      </div>
    </div>
  );
};

export default QuickTest;
