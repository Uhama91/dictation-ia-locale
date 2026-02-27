import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { Shield, Download } from "lucide-react";
import { listen } from "@tauri-apps/api/event";

type PrivacyState = "local" | "downloading";

const PrivacyBadge: React.FC = () => {
  const { t } = useTranslation();
  const [state, setState] = useState<PrivacyState>("local");

  useEffect(() => {
    const unlisteners: (() => void)[] = [];

    listen("model-download-progress", () => {
      setState("downloading");
    }).then((fn) => unlisteners.push(fn));

    listen("model-download-complete", () => {
      setState("local");
    }).then((fn) => unlisteners.push(fn));

    listen("model-download-cancelled", () => {
      setState("local");
    }).then((fn) => unlisteners.push(fn));

    return () => {
      unlisteners.forEach((fn) => fn());
    };
  }, []);

  const isDownloading = state === "downloading";

  return (
    <div
      className={`flex items-center gap-1.5 px-2 py-1 rounded-full text-xs font-medium transition-colors ${
        isDownloading
          ? "bg-amber-500/15 text-amber-400"
          : "bg-emerald-500/15 text-emerald-400"
      }`}
      title={
        isDownloading
          ? t("privacy.downloading")
          : t("privacy.local")
      }
    >
      {isDownloading ? (
        <Download className="w-3 h-3" />
      ) : (
        <Shield className="w-3 h-3" />
      )}
      <span>
        {isDownloading ? t("privacy.downloading") : t("privacy.local")}
      </span>
    </div>
  );
};

export default PrivacyBadge;
