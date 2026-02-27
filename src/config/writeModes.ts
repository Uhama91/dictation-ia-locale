export const WRITE_MODES = ["chat", "pro", "code"] as const;
export type WriteMode = (typeof WRITE_MODES)[number];

export interface WriteModeConfig {
  emoji: string;
  badgeClassName: string;
}

/**
 * Shared write mode config used by WriteModeSelector and HistorySettings.
 * Keys match the values stored in DB and settings ("chat", "pro", "code").
 */
export const WRITE_MODE_CONFIG: Record<WriteMode, WriteModeConfig> = {
  chat: {
    emoji: "💬",
    badgeClassName: "bg-blue-500/15 text-blue-400 border-blue-500/30",
  },
  pro: {
    emoji: "✉️",
    badgeClassName: "bg-purple-500/15 text-purple-400 border-purple-500/30",
  },
  code: {
    emoji: "💻",
    badgeClassName: "bg-green-500/15 text-green-400 border-green-500/30",
  },
};
