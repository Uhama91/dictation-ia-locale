import React, { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { getVersion } from "@tauri-apps/api/app";
import { openUrl } from "@tauri-apps/plugin-opener";
import { SettingsGroup } from "../../ui/SettingsGroup";
import { SettingContainer } from "../../ui/SettingContainer";
import { Button } from "../../ui/Button";
import { AppDataDirectory } from "../AppDataDirectory";
import { AppLanguageSelector } from "../AppLanguageSelector";
import { LogDirectory } from "../debug";
import UpdateChecker from "../../update-checker";

export const AboutSettings: React.FC = () => {
  const { t } = useTranslation();
  const [version, setVersion] = useState("");

  useEffect(() => {
    const fetchVersion = async () => {
      try {
        const appVersion = await getVersion();
        setVersion(appVersion);
      } catch (error) {
        console.error("Failed to get app version:", error);
        setVersion("0.1.0");
      }
    };

    fetchVersion();
  }, []);

  return (
    <div className="max-w-3xl w-full mx-auto space-y-6">
      {/* Header identité DictAI */}
      <div className="text-center py-4">
        {/* eslint-disable-next-line i18next/no-literal-string */}
        <h1 className="text-3xl font-brand text-ink-green">DictAI</h1>
        {/* eslint-disable-next-line i18next/no-literal-string */}
        <span className="text-sm font-mono text-mid-gray">v{version}</span>
        <p className="text-sm text-mid-gray mt-1">
          {t("settings.about.tagline")}
        </p>
      </div>

      <SettingsGroup title={t("settings.about.title")}>
        <AppLanguageSelector descriptionMode="tooltip" grouped={true} />
        <SettingContainer
          title={t("settings.about.version.title")}
          description={t("settings.about.version.description")}
          grouped={true}
        >
          {/* eslint-disable-next-line i18next/no-literal-string */}
          <span className="text-sm font-mono">v{version}</span>
        </SettingContainer>

        <SettingContainer
          title={t("settings.about.license.title")}
          description={t("settings.about.license.description")}
          grouped={true}
        >
          <Button
            variant="secondary"
            size="md"
            onClick={() =>
              openUrl(
                "https://github.com/Uhama91/dictation-ia-locale/blob/main/LICENSE",
              )
            }
          >
            {/* eslint-disable-next-line i18next/no-literal-string */}
            MIT
          </Button>
        </SettingContainer>

        <SettingContainer
          title={t("settings.about.sourceCode.title")}
          description={t("settings.about.sourceCode.description")}
          grouped={true}
        >
          <Button
            variant="secondary"
            size="md"
            onClick={() =>
              openUrl("https://github.com/Uhama91/dictation-ia-locale")
            }
          >
            {t("settings.about.sourceCode.button")}
          </Button>
        </SettingContainer>

        <SettingContainer
          title={t("settings.about.updates.title")}
          description={t("settings.about.updates.description")}
          grouped={true}
        >
          <UpdateChecker />
        </SettingContainer>

        <AppDataDirectory descriptionMode="tooltip" grouped={true} />
        <LogDirectory grouped={true} />
      </SettingsGroup>

      <SettingsGroup title={t("settings.about.acknowledgments.title")}>
        <SettingContainer
          title={t("settings.about.acknowledgments.handy.title")}
          description={t(
            "settings.about.acknowledgments.handy.description",
          )}
          grouped={true}
          layout="stacked"
        />
        <SettingContainer
          title={t("settings.about.acknowledgments.whisper.title")}
          description={t(
            "settings.about.acknowledgments.whisper.description",
          )}
          grouped={true}
          layout="stacked"
        >
          <div className="text-sm text-mid-gray">
            {t("settings.about.acknowledgments.whisper.details")}
          </div>
        </SettingContainer>
        <SettingContainer
          title={t("settings.about.acknowledgments.ollama.title")}
          description={t(
            "settings.about.acknowledgments.ollama.description",
          )}
          grouped={true}
          layout="stacked"
        />
      </SettingsGroup>
    </div>
  );
};
