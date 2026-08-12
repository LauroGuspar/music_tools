import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import ComparisonPanel from "./components/ComparisonPanel";
import ExportSection from "./components/ExportSection";
import PlaylistInputSection from "./components/PlaylistInputSection";
import SubnavTabs, { type AppTab } from "./components/SubnavTabs";
import LocalUtilitiesPanel from "./components/LocalUtilitiesPanel";
import DownloadsPanel from "./components/DownloadsPanel";
import TopBar from "./components/TopBar";
import SettingsDialog from "./components/SettingsDialog";
import TrackPreviewTable from "./components/TrackPreviewTable";
import { useSpotifyAuth } from "./features/auth/useSpotifyAuth";
import { exportJson } from "./features/export/exportJson";
import { exportTxt } from "./features/export/exportTxt";
import { saveExportedFile, type ExportFormat } from "./features/export/saveExportedFile";
import { fetchPlaylistDetails, type PlaylistDetails } from "./features/playlist/spotifyClient";
import { parsePlaylistInput } from "./utils/parsePlaylistInput";
import type { ExportTrack } from "./types/export-track";
import { AudioPlayer, type AudioTrackInfo } from "./components/AudioPlayer";
import "./App.css";

function App() {
  const [settingsVersion, setSettingsVersion] = useState(0);
  const spotifyAuth = useSpotifyAuth(window.localStorage, settingsVersion);
  const [activeTab, setActiveTab] = useState<AppTab>("export");
  const [isSettingsOpen, setIsSettingsOpen] = useState(false);
  const [playlistInput, setPlaylistInput] = useState("");
  const [playlist, setPlaylist] = useState<PlaylistDetails | null>(null);
  const [playlistError, setPlaylistError] = useState<string | null>(null);
  const [playlistNotice, setPlaylistNotice] = useState<string | null>(null);
  const [playlistStatus, setPlaylistStatus] = useState<"idle" | "loading" | "ready">("idle");
  const [exportFormat, setExportFormat] = useState<ExportFormat>("json");
  const [isExporting, setIsExporting] = useState(false);
  const [preloadedTracks, setPreloadedTracks] = useState<ExportTrack[] | null>(null);
  const [preloadedName, setPreloadedName] = useState<string>("");
  const [activeAudioTrack, setActiveAudioTrack] = useState<AudioTrackInfo | null>(null);

  async function loadPlaylist(): Promise<void> {
    const token = await spotifyAuth.ensureValidToken();
    if (!token) {
      setPlaylistError("Inicia sesión con Spotify antes de cargar una playlist.");
      return;
    }

    setPlaylistStatus("loading");
    setPlaylistError(null);
    setPlaylistNotice(null);

    let playlistId = "";
    try {
      const parsed = parsePlaylistInput(playlistInput);
      playlistId = parsed.playlistId;
    } catch (err) {
      setPlaylist(null);
      setPlaylistStatus("idle");
      setPlaylistError(err instanceof Error ? err.message : "ID de playlist inválido.");
      return;
    }

    try {
      // 1. Intentar cargar usando la API oficial (para playlists propias/colaborativas)
      const nextPlaylist = await fetchPlaylistDetails({
        accessToken: token,
        playlistId,
      });

      setPlaylist(nextPlaylist);
      setPlaylistStatus("ready");
    } catch (apiError) {
      console.warn("Fallo en la API oficial de Spotify. Intentando cargar modo lectura pública:", apiError);
      
      try {
        // 2. Fallback: Cargar usando el embed público (raspado seguro en el backend)
        const embedPlaylist = await invoke<PlaylistDetails>("fetch_public_playlist_embed", {
          playlistId,
        });

        setPlaylist(embedPlaylist);
        setPlaylistStatus("ready");
        
        if (embedPlaylist.tracks.length >= 99) {
          setPlaylistNotice(
            "Nota: Se cargaron las primeras 100 canciones de esta playlist pública. Debido a restricciones de desarrollo de Spotify API, si la playlist tiene más de 100 temas, por favor crea una copia (duplícala) en tu propia cuenta de Spotify y carga el enlace de tu copia."
          );
        } else {
          setPlaylistNotice("Playlist pública cargada correctamente en modo lectura.");
        }
      } catch (embedError) {
        console.error("Fallo también en el fallback público:", embedError);
        setPlaylist(null);
        setPlaylistStatus("idle");
        
        const isForbidden = String(apiError).includes("403") || String(apiError).includes("Forbidden");
        if (isForbidden) {
          setPlaylistError(
            "Error 403 (Forbidden): Debido a las restricciones de desarrollo de Spotify (febrero 2026), no se pueden cargar playlists de otros usuarios mediante la API. Por favor, crea una copia (duplícala) de esta playlist en tu propia cuenta de Spotify y carga el enlace de tu copia aquí."
          );
        } else {
          setPlaylistError(apiError instanceof Error ? apiError.message : String(apiError));
        }
      }
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
      <section className="app-frame">
        <TopBar
          authErrorMessage={spotifyAuth.errorMessage}
          isAuthenticated={spotifyAuth.isAuthenticated}
          isConfigured={spotifyAuth.isConfigured}
          onLogin={spotifyAuth.login}
          onLogout={spotifyAuth.logout}
          status={spotifyAuth.status}
          onOpenSettings={() => setIsSettingsOpen(true)}
        />

        <section className="hero-card">
          <p className="eyebrow">Music Tools</p>
          <h1>Exporta playlists hoy y prepara comparaciones para mañana</h1>
          <p className="lead">
            La app ya puede autenticarse con Spotify, cargar playlists reales y exportarlas.
            Ahora el flujo principal vive en tabs internas para que el workspace no crezca sin control.
          </p>

          <SubnavTabs activeTab={activeTab} onChange={setActiveTab} />

          <div className="workspace-shell">
            <div style={{ display: activeTab === "export" ? "block" : "none" }}>
              {!spotifyAuth.isAuthenticated ? (
                <section className="workspace-panel workspace-empty-state">
                  <p className="section-label">Spotify</p>
                  <h2>Inicia sesión para cargar y exportar playlists</h2>
                  <p className="workspace-copy">
                    Usa el menú de la esquina superior derecha y selecciona Spotify para conectar tu cuenta.
                  </p>
                </section>
              ) : (
                <>
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
                      <TrackPreviewTable
                        playlistName={playlist.name}
                        tracks={playlist.tracks}
                        onSendToDownloads={(tracks, name) => {
                          setPreloadedTracks(tracks);
                          setPreloadedName(name);
                          setActiveTab("downloads");
                        }}
                      />
                      <ExportSection
                        disabled={playlist.tracks.length === 0}
                        format={exportFormat}
                        isExporting={isExporting}
                        onChangeFormat={setExportFormat}
                        onExport={exportPlaylist}
                      />
                    </>
                  ) : (
                    <section className="workspace-panel workspace-empty-state">
                      <p className="section-label">Vista de trabajo</p>
                      <h2>Tu playlist aparecera aqui</h2>
                      <p className="workspace-copy">
                        Cuando cargues una URL o un ID valido, veras la tabla de canciones y el bloque de exportacion.
                      </p>
                    </section>
                  )}
                </>
              )}
            </div>

            <div style={{ display: activeTab === "compare" ? "block" : "none" }}>
              <ComparisonPanel
                currentPlaylist={playlist}
                settingsVersion={settingsVersion}
                onSendToDownloads={(tracks, name) => {
                  setPreloadedTracks(tracks);
                  setPreloadedName(name);
                  setActiveTab("downloads");
                }}
              />
            </div>

            <div style={{ display: activeTab === "utilities" ? "block" : "none" }}>
              <LocalUtilitiesPanel
                settingsVersion={settingsVersion}
                onPlayTrack={(track) => setActiveAudioTrack(track)}
              />
            </div>

            <div style={{ display: activeTab === "downloads" ? "block" : "none" }}>
              <DownloadsPanel
                currentPlaylist={playlist}
                settingsVersion={settingsVersion}
                preloadedTracks={preloadedTracks}
                preloadedName={preloadedName}
                onClearPreloaded={() => {
                  setPreloadedTracks(null);
                  setPreloadedName("");
                }}
                onPlayTrack={(track) => setActiveAudioTrack(track)}
              />
            </div>
          </div>
        </section>
      </section>

      <AudioPlayer
        activeTrack={activeAudioTrack}
        onClose={() => setActiveAudioTrack(null)}
      />

      <SettingsDialog
        isOpen={isSettingsOpen}
        onClose={() => setIsSettingsOpen(false)}
        onSettingsSaved={() => setSettingsVersion((v) => v + 1)}
      />
    </main>
  );
}

export default App;
