import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import ComparisonPanel from "./components/ComparisonPanel";
import ExportSection from "./components/ExportSection";
import SubnavTabs, { type AppTab } from "./components/SubnavTabs";
import LocalUtilitiesPanel from "./components/LocalUtilitiesPanel";
import DownloadsPanel from "./components/DownloadsPanel";
import TopBar from "./components/TopBar";
import SettingsDialog from "./components/SettingsDialog";
import { useSpotifyAuth } from "./features/auth/useSpotifyAuth";
import { exportJson } from "./features/export/exportJson";
import { exportTxt } from "./features/export/exportTxt";
import { saveExportedFile, type ExportFormat } from "./features/export/saveExportedFile";
import { fetchPlaylistDetails, type PlaylistDetails } from "./features/playlist/spotifyClient";
import type { ExportTrack } from "./types/export-track";
import { AudioPlayer, type AudioTrackInfo } from "./components/AudioPlayer";
import { SmartOmnibar } from "./components/SmartOmnibar";
import { UniversalPlaylistPreview, type UniversalPlaylistData, type UnifiedTrackInfo } from "./components/UniversalPlaylistPreview";
import { MediaPreviewCard } from "./components/MediaPreviewCard";
import type { DownloadInputType } from "./utils/parseDownloadInput";
import "./App.css";

function App() {
  const [settingsVersion, setSettingsVersion] = useState(0);
  const spotifyAuth = useSpotifyAuth(window.localStorage, settingsVersion);
  const [activeTab, setActiveTab] = useState<AppTab>("export");
  const [isSettingsOpen, setIsSettingsOpen] = useState(false);
  const [playlist, setPlaylist] = useState<PlaylistDetails | null>(null);
  const [playlistError, setPlaylistError] = useState<string | null>(null);
  const [playlistNotice, setPlaylistNotice] = useState<string | null>(null);
  const [exportFormat, setExportFormat] = useState<ExportFormat>("json");
  const [isExporting, setIsExporting] = useState(false);
  const [preloadedTracks, setPreloadedTracks] = useState<ExportTrack[] | null>(null);
  const [preloadedName, setPreloadedName] = useState<string>("");
  const [activeAudioTrack, setActiveAudioTrack] = useState<AudioTrackInfo | null>(null);
  const [omnibarInput, setOmnibarInput] = useState("");
  const [universalPlaylist, setUniversalPlaylist] = useState<UniversalPlaylistData | null>(null);
  const [mediaSearchResults, setMediaSearchResults] = useState<any[]>([]);
  const [mediaTitle, setMediaTitle] = useState("Resultados de búsqueda");
  const [isOmnibarLoading, setIsOmnibarLoading] = useState(false);

  const handleOmnibarSubmitInApp = async (parsed: DownloadInputType) => {
    setIsOmnibarLoading(true);
    setPlaylistError(null);
    setPlaylistNotice(null);
    try {
      if (parsed.type === "youtube_playlist") {
        const details = await invoke<{
          id: string;
          title: string;
          tracks: {
            id: string;
            title: string;
            channel: string;
            duration: string;
            thumbnailUrl: string;
            videoUrl: string;
          }[];
        }>("fetch_youtube_playlist", {
          urlOrId: parsed.playlistId || parsed.rawUrl,
        });

        const unifiedTracks: UnifiedTrackInfo[] = details.tracks.map((t) => ({
          id: t.id || t.videoUrl || t.title,
          title: t.title,
          artistOrChannel: t.channel,
          thumbnailUrl: t.thumbnailUrl,
          duration: t.duration,
          videoUrl: t.videoUrl,
        }));

        const playlistObj: UniversalPlaylistData = {
          id: details.id,
          title: details.title,
          source: "youtube",
          tracks: unifiedTracks,
        };

        setUniversalPlaylist(playlistObj);
        setPlaylist({
          id: details.id,
          name: details.title,
          tracks: details.tracks.map((t) => ({ title: t.title, artists: [t.channel] })),
        });
        setMediaSearchResults([]);
        setPlaylistNotice(`Playlist de YouTube "${details.title}" cargada con ${unifiedTracks.length} canciones.`);
      } else if (parsed.type === "youtube_video") {
        const info = await invoke<{
          id: string;
          title: string;
          channel: string;
          duration: string;
          thumbnailUrl: string;
          videoUrl: string;
        }>("fetch_youtube_video_info", {
          urlOrId: parsed.videoId || parsed.rawUrl,
        });

        setMediaSearchResults([info]);
        setMediaTitle("Vídeo de YouTube");
        setUniversalPlaylist(null);
      } else if (parsed.type === "search_query") {
        if (!parsed.query.trim()) {
          setIsOmnibarLoading(false);
          return;
        }
        const searchResults = await invoke<any[]>("search_youtube", {
          query: parsed.query,
        });
        setMediaSearchResults(searchResults);
        setMediaTitle(`Resultados para "${parsed.query}"`);
        setUniversalPlaylist(null);
      } else if (parsed.type === "spotify_playlist") {
        let playlistData: PlaylistDetails | null = null;
        const token = await spotifyAuth.ensureValidToken();
        if (token) {
          try {
            playlistData = await fetchPlaylistDetails({
              accessToken: token,
              playlistId: parsed.playlistId,
            });
          } catch (err) {
            console.warn("Fallo en API de Spotify, probando fallback público:", err);
          }
        }

        if (!playlistData) {
          playlistData = await invoke<PlaylistDetails>("fetch_public_playlist_embed", {
            playlistId: parsed.playlistId,
          });
        }

        const unifiedTracks: UnifiedTrackInfo[] = playlistData.tracks.map((t, idx) => ({
          id: `${t.title}-${idx}`,
          title: t.title,
          artistOrChannel: t.artists.join(", "),
          thumbnailUrl: t.albumImageUrl,
          albumName: t.albumName,
          releaseDate: t.releaseDate,
        }));

        const playlistObj: UniversalPlaylistData = {
          id: playlistData.id,
          title: playlistData.name,
          source: "spotify",
          tracks: unifiedTracks,
        };

        setUniversalPlaylist(playlistObj);
        setPlaylist(playlistData);
        setMediaSearchResults([]);
        setPlaylistNotice(`Playlist de Spotify "${playlistData.name}" cargada con ${unifiedTracks.length} canciones.`);
      } else if (parsed.type === "spotify_track") {
        setPlaylistError("Para buscar una canción individual, escribe su título/artista en la barra para buscarla en YouTube.");
      }
    } catch (err) {
      console.error(err);
      setPlaylistError(err instanceof Error ? err.message : String(err));
    } finally {
      setIsOmnibarLoading(false);
    }
  };

  const handleAddUniversalTracksToQueue = (selectedTracks: UnifiedTrackInfo[]) => {
    const exportTracks: ExportTrack[] = selectedTracks.map((t) => ({
      title: t.title,
      artists: t.artistOrChannel ? t.artistOrChannel.split(", ") : ["Desconocido"],
      albumName: t.albumName,
      albumImageUrl: t.thumbnailUrl,
      releaseDate: t.releaseDate,
    }));

    setPreloadedTracks(exportTracks);
    setPreloadedName(universalPlaylist?.title || "Playlist");
    setActiveTab("downloads");
  };

  const handleExportUniversalTracks = async (selectedTracks: UnifiedTrackInfo[]) => {
    const exportTracks: ExportTrack[] = selectedTracks.map((t) => ({
      title: t.title,
      artists: t.artistOrChannel ? t.artistOrChannel.split(", ") : ["Desconocido"],
      albumName: t.albumName,
      albumImageUrl: t.thumbnailUrl,
      releaseDate: t.releaseDate,
    }));

    const content = exportFormat === "json" ? exportJson(exportTracks) : exportTxt(exportTracks);
    const didSave = await saveExportedFile({
      content,
      format: exportFormat,
      playlistName: universalPlaylist?.title || "Playlist",
    });

    if (didSave) {
      setPlaylistNotice(`Exportadas ${exportTracks.length} canciones en formato ${exportFormat.toUpperCase()}.`);
    }
  };



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
              <div className="workspace-panel">
                <p className="section-label">Buscador y Carga Inteligente</p>
                <h2>Explorar, exportar y descargar por URL (YouTube / Spotify)</h2>
                <p className="workspace-copy" style={{ marginBottom: "1rem" }}>
                  Pega un enlace de vídeo o playlist de YouTube, una playlist de Spotify o busca por nombre de canción o artista.
                </p>

                <SmartOmnibar
                  value={omnibarInput}
                  onChange={setOmnibarInput}
                  onSubmit={handleOmnibarSubmitInApp}
                  isLoading={isOmnibarLoading}
                />
              </div>

              {playlistError ? <p className="auth-error workspace-error">{playlistError}</p> : null}
              {playlistNotice ? <p className="workspace-notice">{playlistNotice}</p> : null}

              {universalPlaylist ? (
                <>
                  <UniversalPlaylistPreview
                    playlist={universalPlaylist}
                    onAddTracksToQueue={handleAddUniversalTracksToQueue}
                    onExportSelectedTracks={handleExportUniversalTracks}
                    onClose={() => setUniversalPlaylist(null)}
                  />

                  <ExportSection
                    disabled={universalPlaylist.tracks.length === 0}
                    format={exportFormat}
                    isExporting={isExporting}
                    onChangeFormat={setExportFormat}
                    onExport={exportPlaylist}
                  />
                </>
              ) : mediaSearchResults.length > 0 ? (
                <MediaPreviewCard
                  results={mediaSearchResults}
                  title={mediaTitle}
                  onDownloadTrack={(track) => {
                    setPreloadedTracks([{ title: track.title, artists: [track.channel] }]);
                    setPreloadedName(track.title);
                    setActiveTab("downloads");
                  }}
                  onPlayPreview={(track) => {
                    setActiveAudioTrack({ filePath: track.videoUrl, title: track.title, artists: [track.channel] });
                  }}
                  onClose={() => setMediaSearchResults([])}
                />
              ) : (
                <section className="workspace-panel workspace-empty-state">
                  <p className="section-label">Vista de trabajo</p>
                  <h2>Tu playlist aparecera aqui</h2>
                  <p className="workspace-copy">
                    Cuando cargues un enlace o un ID válido (Spotify o YouTube), verás la tabla interactiva con portadas, canal/artista, duración, casillas de selección, exportación a JSON/TXT y el botón para descargar.
                  </p>
                </section>
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
