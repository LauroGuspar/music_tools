import { useMemo } from "react";
import { parseDownloadInput, type DownloadInputType } from "../utils/parseDownloadInput";
import { YoutubeIcon, MusicIcon } from "./Icons";

type SmartOmnibarProps = {
  value: string;
  onChange: (val: string) => void;
  onSubmit: (parsed: DownloadInputType) => void;
  isLoading?: boolean;
  disabled?: boolean;
};

export function SmartOmnibar({
  value,
  onChange,
  onSubmit,
  isLoading = false,
  disabled = false,
}: SmartOmnibarProps) {
  const parsed = useMemo(() => parseDownloadInput(value), [value]);

  const badgeInfo = useMemo(() => {
    if (!value.trim()) return null;
    switch (parsed.type) {
      case "youtube_playlist":
        return { label: "YouTube Playlist", color: "#FF0000", icon: "youtube" };
      case "youtube_video":
        return { label: "Vídeo de YouTube", color: "#FF0000", icon: "youtube" };
      case "spotify_playlist":
        return { label: "Spotify Playlist", color: "#1DB954", icon: "spotify" };
      case "spotify_album":
        return { label: "Spotify Álbum", color: "#1DB954", icon: "spotify" };
      case "spotify_track":
        return { label: "Spotify Canción", color: "#1DB954", icon: "spotify" };
      case "search_query":
        return { label: "Búsqueda libre", color: "#3B82F6", icon: "search" };
    }
  }, [value, parsed]);

  const buttonLabel = useMemo(() => {
    if (isLoading) return "Procesando...";
    switch (parsed.type) {
      case "youtube_playlist":
      case "spotify_playlist":
        return "Cargar Playlist";
      case "spotify_album":
        return "Cargar Álbum";
      case "youtube_video":
        return "Obtener Vídeo";
      case "spotify_track":
        return "Cargar Canción";
      case "search_query":
        return "Buscar en YouTube";
    }
  }, [isLoading, parsed]);

  const handleSubmit = (e?: React.FormEvent) => {
    if (e) e.preventDefault();
    if (!value.trim() || isLoading || disabled) return;
    onSubmit(parsed);
  };

  return (
    <form className="smart-omnibar-form" onSubmit={handleSubmit}>
      <div className="smart-omnibar-container" style={{ position: "relative", display: "flex", flex: 1, alignItems: "center" }}>
        <input
          type="text"
          className="playlist-input smart-omnibar-input"
          placeholder="Pega un enlace de YouTube, Spotify o busca por título/artista..."
          value={value}
          onChange={(e) => onChange(e.target.value)}
          disabled={disabled || isLoading}
          style={{ width: "100%", paddingRight: badgeInfo ? "140px" : "12px" }}
        />

        {badgeInfo && (
          <span
            className="smart-omnibar-badge"
            style={{
              position: "absolute",
              right: "12px",
              display: "inline-flex",
              alignItems: "center",
              gap: "6px",
              fontSize: "0.75rem",
              fontWeight: 600,
              padding: "2px 8px",
              borderRadius: "12px",
              backgroundColor: `${badgeInfo.color}22`,
              color: badgeInfo.color,
              border: `1px solid ${badgeInfo.color}44`,
              pointerEvents: "none",
            }}
          >
            {badgeInfo.icon === "youtube" && <YoutubeIcon style={{ width: 14, height: 14 }} />}
            {badgeInfo.icon === "spotify" && <MusicIcon style={{ width: 14, height: 14 }} />}
            {badgeInfo.label}
          </span>
        )}
      </div>

      <button
        type="submit"
        className="primary-button smart-omnibar-button"
        disabled={disabled || isLoading || !value.trim()}
      >
        {buttonLabel}
      </button>
    </form>
  );
}
