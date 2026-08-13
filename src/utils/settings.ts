export type ThemeMode = "dark" | "light" | "system";

export type ThemePreset =
  | "default-dark"
  | "default-light"
  | "catppuccin-mocha"
  | "catppuccin-latte"
  | "dracula"
  | "one-dark"
  | "nord";

export type AppSettings = {
  defaultLocalMusicFolder: string;
  defaultPlaylistExportFolder: string;
  defaultSongDownloadFolder: string;
  fuzzyThreshold: number;
  spotifyClientId: string;
  manualYoutubeSelection: boolean;
  themeMode: ThemeMode;
  themePreset: ThemePreset;
};

const SETTINGS_KEY = "music_tools_settings_v1";

export const DEFAULT_SETTINGS: AppSettings = {
  defaultLocalMusicFolder: "",
  defaultPlaylistExportFolder: "",
  defaultSongDownloadFolder: "",
  fuzzyThreshold: 0.8,
  spotifyClientId: "",
  manualYoutubeSelection: false,
  themeMode: "dark",
  themePreset: "default-dark",
};

export function loadSettings(): AppSettings {
  try {
    if (typeof localStorage === "undefined") return DEFAULT_SETTINGS;
    const raw = localStorage.getItem(SETTINGS_KEY);
    if (!raw) return DEFAULT_SETTINGS;
    const parsed = JSON.parse(raw);
    return { ...DEFAULT_SETTINGS, ...parsed };
  } catch (err) {
    console.error("Error loading settings:", err);
    return DEFAULT_SETTINGS;
  }
}

export function saveSettings(settings: AppSettings): void {
  try {
    if (typeof localStorage === "undefined") return;
    localStorage.setItem(SETTINGS_KEY, JSON.stringify(settings));
    applyTheme(settings);
  } catch (err) {
    console.error("Error saving settings:", err);
  }
}

export function applyTheme(settings: AppSettings): void {
  if (typeof document === "undefined") return;
  const root = document.documentElement;
  const theme = settings.themePreset || "default-dark";
  let mode = settings.themeMode || "dark";

  if (mode === "system") {
    const prefersDark = window.matchMedia && window.matchMedia("(prefers-color-scheme: dark)").matches;
    mode = prefersDark ? "dark" : "light";
  }

  root.setAttribute("data-theme", theme);
  root.setAttribute("data-mode", mode);
}

