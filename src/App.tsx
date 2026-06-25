import { useState } from "react";
import AuthSection from "./components/AuthSection";
import ExportSection from "./components/ExportSection";
import PlaylistInputSection from "./components/PlaylistInputSection";
import TrackPreviewTable from "./components/TrackPreviewTable";
import { useSpotifyAuth } from "./features/auth/useSpotifyAuth";
import { exportJson } from "./features/export/exportJson";
import { exportTxt } from "./features/export/exportTxt";
import { saveExportedFile, type ExportFormat } from "./features/export/saveExportedFile";
import { fetchPlaylistDetails, type PlaylistDetails } from "./features/playlist/spotifyClient";
import { parsePlaylistInput } from "./utils/parsePlaylistInput";
import "./App.css";

function App() {
  const spotifyAuth = useSpotifyAuth();
  const [playlistInput, setPlaylistInput] = useState("");
  const [playlist, setPlaylist] = useState<PlaylistDetails | null>(null);
  const [playlistError, setPlaylistError] = useState<string | null>(null);
  const [playlistNotice, setPlaylistNotice] = useState<string | null>(null);
  const [playlistStatus, setPlaylistStatus] = useState<"idle" | "loading" | "ready">("idle");
  const [exportFormat, setExportFormat] = useState<ExportFormat>("json");
  const [isExporting, setIsExporting] = useState(false);

  async function loadPlaylist(): Promise<void> {
    if (!spotifyAuth.accessToken) {
      setPlaylistError("Inicia sesion con Spotify antes de cargar una playlist.");
      return;
    }

    setPlaylistStatus("loading");
    setPlaylistError(null);
    setPlaylistNotice(null);

    try {
      const { playlistId } = parsePlaylistInput(playlistInput);
      const nextPlaylist = await fetchPlaylistDetails({
        accessToken: spotifyAuth.accessToken,
        playlistId,
      });

      setPlaylist(nextPlaylist);
      setPlaylistStatus("ready");
    } catch (error) {
      setPlaylist(null);
      setPlaylistStatus("idle");
      setPlaylistError(error instanceof Error ? error.message : "No se pudo cargar la playlist.");
    }
  }

  async function exportPlaylist(): Promise<void> {
    if (!playlist) {
      return;
    }

    setIsExporting(true);
    setPlaylistError(null);
    setPlaylistNotice("Preparando el archivo para exportar...");

    try {
      const content = exportFormat === "json" ? exportJson(playlist.tracks) : exportTxt(playlist.tracks);

      const didSave = await saveExportedFile({
        content,
        format: exportFormat,
        playlistName: playlist.name,
      });

      setPlaylistNotice(
        didSave
          ? `Archivo ${exportFormat.toUpperCase()} guardado correctamente.`
          : "Exportacion cancelada por el usuario.",
      );
    } catch (error) {
      setPlaylistError(error instanceof Error ? error.message : "No se pudo exportar la playlist.");
      setPlaylistNotice(null);
    } finally {
      setIsExporting(false);
    }
  }

  return (
    <main className="app-shell">
      <section className="hero-card">
        <p className="eyebrow">Spotify Tools</p>
        <h1>Exporta playlists de Spotify a JSON o TXT</h1>
        <p className="lead">
          La base ya soporta el inicio de sesion con Spotify en escritorio usando
          PKCE y callback local. El siguiente bloque conectara lectura de playlists
          y exportacion.
        </p>

        <div className="status-grid" aria-label="Estado inicial del proyecto">
          <article>
            <span className="status-label">Shell</span>
            <strong>Tauri v2</strong>
          </article>
          <article>
            <span className="status-label">Frontend</span>
            <strong>React + TypeScript</strong>
          </article>
          <article>
            <span className="status-label">Proximo modulo</span>
            <strong>Exportacion</strong>
          </article>
        </div>

        <AuthSection
          errorMessage={spotifyAuth.errorMessage}
          isAuthenticated={spotifyAuth.isAuthenticated}
          isConfigured={spotifyAuth.isConfigured}
          onLogin={spotifyAuth.login}
          onLogout={spotifyAuth.logout}
          status={spotifyAuth.status}
        />

        <PlaylistInputSection
          disabled={!spotifyAuth.isAuthenticated}
          inputValue={playlistInput}
          isLoading={playlistStatus === "loading"}
          onChange={setPlaylistInput}
          onSubmit={loadPlaylist}
        />

        {playlistError ? <p className="auth-error workspace-error">{playlistError}</p> : null}
        {playlistNotice ? <p className="workspace-notice">{playlistNotice}</p> : null}

        {playlist ? (
          <>
            <TrackPreviewTable playlistName={playlist.name} tracks={playlist.tracks} />
            <ExportSection
              disabled={playlist.tracks.length === 0}
              format={exportFormat}
              isExporting={isExporting}
              onChangeFormat={setExportFormat}
              onExport={exportPlaylist}
            />
          </>
        ) : null}
      </section>
    </main>
  );
}

export default App;
