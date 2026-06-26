export type AppSettings = {
  defaultLocalMusicFolder: string;
  defaultPlaylistExportFolder: string;
  defaultSongDownloadFolder: string;
  fuzzyThreshold: number;
};

const SETTINGS_KEY = "music_tools_settings_v1";

export const DEFAULT_SETTINGS: AppSettings = {
  defaultLocalMusicFolder: "",
  defaultPlaylistExportFolder: "",
  defaultSongDownloadFolder: "",
  fuzzyThreshold: 0.8,
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
  } catch (err) {
    console.error("Error saving settings:", err);
  }
}
