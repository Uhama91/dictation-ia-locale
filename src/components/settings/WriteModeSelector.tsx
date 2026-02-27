import React from "react";
import { useTranslation } from "react-i18next";
import { SettingContainer } from "../ui/SettingContainer";
import { useSettings } from "../../hooks/useSettings";
import { WRITE_MODES, WRITE_MODE_CONFIG, type WriteMode } from "@/config/writeModes";

interface WriteModeButtonProps {
  mode: WriteMode;
  active: boolean;
  label: string;
  description: string;
  onClick: () => void;
}

const WriteModeButton: React.FC<WriteModeButtonProps> = ({
  mode,
  active,
  label,
  description,
  onClick,
}) => (
  <button
    onClick={onClick}
    title={description}
    className={[
      "flex flex-col items-center px-3 py-2 rounded-lg border text-xs font-medium transition-all duration-150 select-none",
      active
        ? "border-logo-primary bg-logo-primary/10 text-logo-primary"
        : "border-mid-gray/30 text-mid-gray hover:border-mid-gray/60 hover:text-white/80",
    ].join(" ")}
  >
    <span className="text-base leading-none mb-0.5">
      {WRITE_MODE_CONFIG[mode].emoji}
    </span>
    <span>{label}</span>
  </button>
);

interface WriteModeProps {
  descriptionMode?: "inline" | "tooltip";
  grouped?: boolean;
}

export const WriteModeSelector: React.FC<WriteModeProps> = React.memo(
  ({ descriptionMode = "tooltip", grouped = false }) => {
    const { t } = useTranslation();
    const { getSetting, updateSetting, isUpdating } = useSettings();

    const currentMode = (getSetting("write_mode") as WriteMode) ?? "chat";
    const updating = isUpdating("write_mode");

    return (
      <SettingContainer
        title={t("settings.general.writeMode.title")}
        description={t("settings.general.writeMode.description")}
        descriptionMode={descriptionMode}
        grouped={grouped}
        layout="stacked"
        disabled={updating}
      >
        <div className="flex gap-2 mt-1" aria-label="Mode d'écriture">
          {WRITE_MODES.map((mode) => (
            <WriteModeButton
              key={mode}
              mode={mode}
              active={currentMode === mode}
              label={t(`settings.general.writeMode.modes.${mode}.label`)}
              description={t(
                `settings.general.writeMode.modes.${mode}.description`,
              )}
              onClick={() => !updating && updateSetting("write_mode", mode)}
            />
          ))}
        </div>
      </SettingContainer>
    );
  },
);

WriteModeSelector.displayName = "WriteModeSelector";
