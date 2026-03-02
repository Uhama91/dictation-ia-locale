import React from "react";
import { useTranslation } from "react-i18next";
import { SettingContainer } from "../ui/SettingContainer";
import { Dropdown, type DropdownOption } from "../ui/Dropdown";
import { useSettings } from "../../hooks/useSettings";
import { commands } from "@/bindings";
import { toast } from "sonner";

const TRIGGER_KEY_OPTIONS: DropdownOption[] = [
  { value: "option", label: "Option (⌥)" },
  { value: "command", label: "Commande (⌘)" },
];

interface TriggerKeySelectorProps {
  descriptionMode?: "tooltip" | "inline";
  grouped?: boolean;
}

export const TriggerKeySelector: React.FC<TriggerKeySelectorProps> = ({
  descriptionMode = "tooltip",
  grouped = false,
}) => {
  const { t } = useTranslation();
  const { getSetting, isUpdating, refreshSettings } = useSettings();
  const currentKey = (getSetting("trigger_key") as string) ?? "option";

  const handleSelect = async (value: string) => {
    if (value === currentKey) return;

    try {
      const result = await commands.changeTriggerKeySetting(value);
      if (result.status === "error") {
        console.error("Failed to update trigger key:", result.error);
        toast.error(String(result.error));
        return;
      }
      await refreshSettings();
    } catch (error) {
      console.error("Failed to update trigger key:", error);
      toast.error(String(error));
    }
  };

  const optionLabel =
    currentKey === "option"
      ? t("settings.triggerKey.option")
      : t("settings.triggerKey.command");

  return (
    <SettingContainer
      title={t("settings.triggerKey.title")}
      description={t("settings.triggerKey.description")}
      descriptionMode={descriptionMode}
      grouped={grouped}
      layout="horizontal"
    >
      <div className="flex flex-col items-end gap-1">
        <Dropdown
          options={TRIGGER_KEY_OPTIONS}
          selectedValue={currentKey}
          onSelect={handleSelect}
          disabled={isUpdating("trigger_key")}
        />
        <div className="flex flex-col items-end gap-0.5 text-[10px] text-text/40 leading-tight">
          <span>{t("settings.triggerKey.holdMode")}</span>
          <span>{t("settings.triggerKey.doubleTapMode")}</span>
        </div>
      </div>
    </SettingContainer>
  );
};
