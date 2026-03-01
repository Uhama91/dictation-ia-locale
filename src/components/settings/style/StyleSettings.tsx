import React from "react";
import { useTranslation } from "react-i18next";
import { useSettings } from "../../../hooks/useSettings";
import { WRITE_MODES, WRITE_MODE_CONFIG, type WriteMode } from "@/config/writeModes";

interface WriteModeCardProps {
  mode: WriteMode;
  active: boolean;
  disabled: boolean;
  onClick: () => void;
}

const WriteModeCard: React.FC<WriteModeCardProps> = ({
  mode,
  active,
  disabled,
  onClick,
}) => {
  const { t } = useTranslation();
  const config = WRITE_MODE_CONFIG[mode];

  const examples: Record<WriteMode, { before: string; after: string }> = {
    chat: {
      before: t("settings.style.examples.chat.before"),
      after: t("settings.style.examples.chat.after"),
    },
    pro: {
      before: t("settings.style.examples.pro.before"),
      after: t("settings.style.examples.pro.after"),
    },
    code: {
      before: t("settings.style.examples.code.before"),
      after: t("settings.style.examples.code.after"),
    },
  };

  return (
    <button
      onClick={onClick}
      disabled={disabled}
      className={[
        "w-full text-left p-4 rounded-lg border transition-all duration-150",
        disabled
          ? "opacity-50 cursor-not-allowed"
          : "cursor-pointer",
        active
          ? "border-logo-primary bg-logo-primary/5 ring-1 ring-logo-primary/30"
          : "border-mid-gray/20 bg-background hover:border-mid-gray/40",
      ].join(" ")}
    >
      <div className="flex items-center gap-3 mb-3">
        <span className="text-2xl">{config.emoji}</span>
        <div>
          <h3 className="text-sm font-semibold text-text">
            {t(`settings.general.writeMode.modes.${mode}.label`)}
          </h3>
          <p className="text-xs text-text/60">
            {t(`settings.general.writeMode.modes.${mode}.description`)}
          </p>
        </div>
        {active && (
          <span className="ml-auto text-[10px] font-medium uppercase tracking-wider text-logo-primary bg-logo-primary/10 px-2 py-0.5 rounded-full">
            {t("modelSelector.active")}
          </span>
        )}
      </div>

      <div className="space-y-2 pt-2 border-t border-mid-gray/10">
        <div>
          <span className="text-[10px] uppercase tracking-wider text-text/40 font-medium">
            {t("settings.style.before")}
          </span>
          <p className="text-xs text-text/50 italic mt-0.5">
            {examples[mode].before}
          </p>
        </div>
        <div>
          <span className="text-[10px] uppercase tracking-wider text-logo-primary/70 font-medium">
            {t("settings.style.after")}
          </span>
          <p className="text-xs text-text/80 mt-0.5">{examples[mode].after}</p>
        </div>
      </div>
    </button>
  );
};

export const StyleSettings: React.FC = () => {
  const { t } = useTranslation();
  const { getSetting, updateSetting, isUpdating } = useSettings();

  const currentMode = (getSetting("write_mode") as WriteMode) ?? "chat";
  const updating = isUpdating("write_mode");

  return (
    <div className="max-w-3xl w-full mx-auto space-y-4">
      <div className="px-4">
        <h2 className="text-xs font-medium text-mid-gray uppercase tracking-wide">
          {t("settings.general.writeMode.title")}
        </h2>
        <p className="text-xs text-mid-gray mt-1">
          {t("settings.general.writeMode.description")}
        </p>
      </div>
      <div className="space-y-3">
        {WRITE_MODES.map((mode) => (
          <WriteModeCard
            key={mode}
            mode={mode}
            active={currentMode === mode}
            disabled={updating}
            onClick={() => updateSetting("write_mode", mode)}
          />
        ))}
      </div>
    </div>
  );
};
