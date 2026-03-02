import React, { useState } from "react";
import { useTranslation } from "react-i18next";
import { ChevronDown } from "lucide-react";
import { MicrophoneSelector } from "../MicrophoneSelector";
import { OutputDeviceSelector } from "../OutputDeviceSelector";
import { AudioFeedback } from "../AudioFeedback";
import { VolumeSlider } from "../VolumeSlider";
import { MuteWhileRecording } from "../MuteWhileRecording";
import { PushToTalk } from "../PushToTalk";
import { TriggerKeySelector } from "../TriggerKeySelector";
import { ModelSettingsCard } from "../general/ModelSettingsCard";
import { ModelsSettings } from "../models/ModelsSettings";
import { ShowOverlay } from "../ShowOverlay";
import { ModelUnloadTimeoutSetting } from "../ModelUnloadTimeout";
import { CustomWords } from "../CustomWords";
import { StartHidden } from "../StartHidden";
import { AutostartToggle } from "../AutostartToggle";
import { ShowTrayIcon } from "../ShowTrayIcon";
import { PasteMethodSetting } from "../PasteMethod";
import { TypingToolSetting } from "../TypingTool";
import { ClipboardHandlingSetting } from "../ClipboardHandling";
import { AutoSubmit } from "../AutoSubmit";
import { PostProcessingToggle } from "../PostProcessingToggle";
import { AppendTrailingSpace } from "../AppendTrailingSpace";
import { HistoryLimit } from "../HistoryLimit";
import { RecordingRetentionPeriodSelector } from "../RecordingRetentionPeriod";
import { ExperimentalToggle } from "../ExperimentalToggle";
import { KeyboardImplementationSelector } from "../debug/KeyboardImplementationSelector";
import { HistorySettings } from "../history/HistorySettings";
import { AboutSettings } from "../about/AboutSettings";
import { DebugSettings } from "../debug/DebugSettings";
import { PostProcessingSettings } from "../post-processing/PostProcessingSettings";
import { useSettings } from "../../../hooks/useSettings";

interface AccordionSectionProps {
  id: string;
  title: string;
  isOpen: boolean;
  onToggle: () => void;
  children: React.ReactNode;
}

const AccordionSection: React.FC<AccordionSectionProps> = ({
  id,
  title,
  isOpen,
  onToggle,
  children,
}) => (
  <div className="border border-mid-gray/20 rounded-lg overflow-hidden">
    <button
      onClick={onToggle}
      className="w-full flex items-center justify-between px-4 py-3 bg-background hover:bg-mid-gray/5 transition-colors cursor-pointer"
      aria-expanded={isOpen}
      aria-controls={`accordion-${id}`}
    >
      <span className="text-sm font-medium text-text">{title}</span>
      <ChevronDown
        size={16}
        className={`text-text/50 transition-transform duration-200 ${isOpen ? "rotate-180" : ""}`}
      />
    </button>
    {isOpen && (
      <div id={`accordion-${id}`} className="border-t border-mid-gray/20 p-4">
        {children}
      </div>
    )}
  </div>
);

export const ParametresSettings: React.FC = () => {
  const { t } = useTranslation();
  const { settings, audioFeedbackEnabled, getSetting } = useSettings();
  const experimentalEnabled = getSetting("experimental_enabled") || false;
  const postProcessEnabled = settings?.post_process_enabled ?? false;
  const debugMode = settings?.debug_mode ?? false;

  const [openSections, setOpenSections] = useState<Record<string, boolean>>({
    audio: false,
    shortcuts: false,
    models: false,
    history: false,
    advanced: false,
    postprocessing: false,
    about: false,
    debug: false,
  });

  const toggleSection = (id: string) => {
    setOpenSections((prev) => ({ ...prev, [id]: !prev[id] }));
  };

  return (
    <div className="max-w-3xl w-full mx-auto space-y-3">
      <div className="px-4">
        <h2 className="text-xs font-medium text-mid-gray uppercase tracking-wide">
          {t("sidebar.parametres")}
        </h2>
      </div>

      <AccordionSection
        id="audio"
        title={t("settings.sound.title")}
        isOpen={openSections.audio}
        onToggle={() => toggleSection("audio")}
      >
        <div className="space-y-1">
          <MicrophoneSelector descriptionMode="tooltip" grouped={true} />
          <MuteWhileRecording descriptionMode="tooltip" grouped={true} />
          <AudioFeedback descriptionMode="tooltip" grouped={true} />
          <OutputDeviceSelector
            descriptionMode="tooltip"
            grouped={true}
            disabled={!audioFeedbackEnabled}
          />
          <VolumeSlider disabled={!audioFeedbackEnabled} />
          <PushToTalk descriptionMode="tooltip" grouped={true} />
        </div>
      </AccordionSection>

      <AccordionSection
        id="shortcuts"
        title={t("settings.general.shortcut.title")}
        isOpen={openSections.shortcuts}
        onToggle={() => toggleSection("shortcuts")}
      >
        <div className="space-y-1">
          <TriggerKeySelector descriptionMode="tooltip" grouped={true} />
        </div>
      </AccordionSection>

      <AccordionSection
        id="models"
        title={t("settings.models.title")}
        isOpen={openSections.models}
        onToggle={() => toggleSection("models")}
      >
        <ModelSettingsCard />
        <div className="mt-4">
          <ModelsSettings />
        </div>
      </AccordionSection>

      <AccordionSection
        id="history"
        title={t("settings.history.title")}
        isOpen={openSections.history}
        onToggle={() => toggleSection("history")}
      >
        <HistorySettings />
      </AccordionSection>

      <AccordionSection
        id="advanced"
        title={t("settings.advanced.title")}
        isOpen={openSections.advanced}
        onToggle={() => toggleSection("advanced")}
      >
        <div className="space-y-4">
          <div className="space-y-1">
            <StartHidden descriptionMode="tooltip" grouped={true} />
            <AutostartToggle descriptionMode="tooltip" grouped={true} />
            <ShowTrayIcon descriptionMode="tooltip" grouped={true} />
            <ShowOverlay descriptionMode="tooltip" grouped={true} />
            <ModelUnloadTimeoutSetting
              descriptionMode="tooltip"
              grouped={true}
            />
            <ExperimentalToggle descriptionMode="tooltip" grouped={true} />
          </div>
          <div className="space-y-1">
            <PasteMethodSetting descriptionMode="tooltip" grouped={true} />
            <TypingToolSetting descriptionMode="tooltip" grouped={true} />
            <ClipboardHandlingSetting
              descriptionMode="tooltip"
              grouped={true}
            />
            <AutoSubmit descriptionMode="tooltip" grouped={true} />
          </div>
          <div className="space-y-1">
            <CustomWords descriptionMode="tooltip" grouped />
            <AppendTrailingSpace descriptionMode="tooltip" grouped={true} />
          </div>
          <div className="space-y-1">
            <HistoryLimit descriptionMode="tooltip" grouped={true} />
            <RecordingRetentionPeriodSelector
              descriptionMode="tooltip"
              grouped={true}
            />
          </div>
          {experimentalEnabled && (
            <div className="space-y-1">
              <PostProcessingToggle descriptionMode="tooltip" grouped={true} />
              <KeyboardImplementationSelector
                descriptionMode="tooltip"
                grouped={true}
              />
            </div>
          )}
        </div>
      </AccordionSection>

      {postProcessEnabled && (
        <AccordionSection
          id="postprocessing"
          title={t("settings.postProcessing.title")}
          isOpen={openSections.postprocessing}
          onToggle={() => toggleSection("postprocessing")}
        >
          <PostProcessingSettings />
        </AccordionSection>
      )}

      <AccordionSection
        id="about"
        title={t("settings.about.title")}
        isOpen={openSections.about}
        onToggle={() => toggleSection("about")}
      >
        <AboutSettings />
      </AccordionSection>

      {debugMode && (
        <AccordionSection
          id="debug"
          title={t("settings.debug.title")}
          isOpen={openSections.debug}
          onToggle={() => toggleSection("debug")}
        >
          <DebugSettings />
        </AccordionSection>
      )}
    </div>
  );
};
