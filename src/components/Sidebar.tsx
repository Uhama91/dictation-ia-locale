import React from "react";
import { useTranslation } from "react-i18next";
import { Home, Palette, Settings } from "lucide-react";
import DictationLogo from "./icons/DictationLogo";
import { AccueilSettings } from "./settings/accueil/AccueilSettings";
import { StyleSettings } from "./settings/style/StyleSettings";
import { ParametresSettings } from "./settings/parametres/ParametresSettings";

export type SidebarSection = keyof typeof SECTIONS_CONFIG;

interface IconProps {
  width?: number | string;
  height?: number | string;
  size?: number | string;
  className?: string;
  [key: string]: any;
}

interface SectionConfig {
  labelKey: string;
  icon: React.ComponentType<IconProps>;
  component: React.ComponentType;
}

export const SECTIONS_CONFIG = {
  accueil: {
    labelKey: "sidebar.accueil",
    icon: Home,
    component: AccueilSettings,
  },
  style: {
    labelKey: "sidebar.style",
    icon: Palette,
    component: StyleSettings,
  },
  parametres: {
    labelKey: "sidebar.parametres",
    icon: Settings,
    component: ParametresSettings,
  },
} as const satisfies Record<string, SectionConfig>;

interface SidebarProps {
  activeSection: SidebarSection;
  onSectionChange: (section: SidebarSection) => void;
}

export const Sidebar: React.FC<SidebarProps> = ({
  activeSection,
  onSectionChange,
}) => {
  const { t } = useTranslation();

  const sections = Object.entries(SECTIONS_CONFIG).map(([id, config]) => ({
    id: id as SidebarSection,
    ...config,
  }));

  return (
    <div className="flex flex-col w-40 h-full border-e border-mid-gray/20 items-center px-2">
      <DictationLogo width={120} className="m-4" />
      <div className="flex flex-col w-full items-center gap-1 pt-2 border-t border-mid-gray/20">
        {sections.map((section) => {
          const Icon = section.icon;
          const isActive = activeSection === section.id;

          return (
            <button
              key={section.id}
              type="button"
              className={`flex gap-2 items-center p-2 w-full rounded-lg cursor-pointer transition-colors text-left ${
                isActive
                  ? "bg-logo-primary/80"
                  : "hover:bg-mid-gray/20 hover:opacity-100 opacity-85"
              }`}
              onClick={() => onSectionChange(section.id)}
              aria-current={isActive ? "page" : undefined}
            >
              <Icon width={24} height={24} className="shrink-0" />
              <span
                className="text-sm font-medium truncate"
                title={t(section.labelKey)}
              >
                {t(section.labelKey)}
              </span>
            </button>
          );
        })}
      </div>
    </div>
  );
};
