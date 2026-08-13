import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import ComparisonPanel from "./components/ComparisonPanel";
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
import { StagingWorkspace, type StagingBlockData, type StagingTrack } from "./components/StagingWorkspace";
import type { DownloadInputType } from "./utils/parseDownloadInput";
import { deduplicateStagingBlocks } from "./utils/duplicateDetector";
import { loadSettings, applyTheme } from "./utils/settings";
import { ListIcon, InfoIcon, CloseIcon } from "./components/Icons";
import "./App.css";

function App() {
  const [settingsVersion, setSettingsVersion] = useState(0);

  useEffect(() => {
    applyTheme(loadSettings());
  }, [settingsVersion]);

  const spotifyAuth = useSpotifyAuth(window.localStorage, settingsVersion);
  const [activeTab, setActiveTab] = useState<AppTab>("export");
  const [isSettingsOpen, setIsSettingsOpen] = useState(false);
  const [playlist, setPlaylist] = useState<PlaylistDetails | null>(null);
  const [playlistError, setPlaylistError] = useState<string | null>(null);
  const [playlistNotice, setPlaylistNotice] = useState<string | null>(null);
  const [preloadedTracks, setPreloadedTracks] = useState<ExportTrack[] | null>(null);
  const [preloadedName, setPreloadedName] = useState<string>("");
  const [activeAudioTrack, setActiveAudioTrack] = useState<AudioTrackInfo | null>(null);
  const [omnibarInput, setOmnibarInput] = useState("");
  const [universalPlaylist, setUniversalPlaylist] = useState<UniversalPlaylistData | null>(null);
  const [mediaSearchResults, setMediaSearchResults] = useState<any[]>([]);
  const [mediaTitle, setMediaTitle] = useState("Resultados de búsqueda");
  const [isOmnibarLoading, setIsOmnibarLoading] = useState(false);
  const [stagingBlocks, setStagingBlocks] = useState<StagingBlockData[]>([]);
  const [spotifyFallbackWarning, setSpotifyFallbackWarning] = useState<string | null>(null);
  const [isStagingDrawerOpen, setIsStagingDrawerOpen] = useState(false);

  const handleOmnibarSubmitInApp = async (parsed: DownloadInputType) => {
    setIsOmnibarLoading(true);
    setPlaylistError(null);
    setPlaylistNotice(null);
    setSpotifyFallbackWarning(null);
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
        let isPublicFallbackUsed = false;
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
          isPublicFallbackUsed = true;
        }

        if (isPublicFallbackUsed) {
          setSpotifyFallbackWarning(
            "Esta playlist de Spotify no pertenece a tu cuenta o requiere permisos restringidos (error 403 en API de Spotify). " +
            "Se ha cargado usando el reproductor público de Spotify, el cual está limitado por Spotify a las primeras ~99 canciones y sólo incluye información básica (Título y Artista)."
          );
        }

        const unifiedTracks: UnifiedTrackInfo[] = playlistData.tracks.map((t, idx) => ({
          id: `${t.title}-${idx}`,
          title: t.title,
          artistOrChannel: t.artists.join(", "),
          thumbnailUrl: t.albumImageUrl,
          albumName: t.albumName,
          releaseDate: t.releaseDate,
          duration: t.duration,
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

  const handleAddUniversalTracksToStaging = (selectedTracks: UnifiedTrackInfo[]) => {
    if (!universalPlaylist) return;
    const newBlock: StagingBlockData = {
      id: `block-${Date.now()}`,
      title: universalPlaylist.title,
      source: universalPlaylist.source,
      tracks: selectedTracks.map((t) => ({
        id: t.id,
        title: t.title,
        artistOrChannel: t.artistOrChannel,
        thumbnailUrl: t.thumbnailUrl,
        duration: t.duration,
        videoUrl: t.videoUrl,
        albumName: t.albumName,
        releaseDate: t.releaseDate,
      })),
    };
    setStagingBlocks((prev) => [...prev, newBlock]);
    setUniversalPlaylist(null);
    setIsStagingDrawerOpen(true);
    setPlaylistNotice(`Añadido el bloque "${universalPlaylist.title}" a la Mesa de Trabajo (${selectedTracks.length} canciones).`);
  };

  const handleAddMediaTrackToStaging = (track: any) => {
    const trackObj: StagingTrack = {
      id: track.id || track.videoUrl || track.title,
      title: track.title,
      artistOrChannel: track.channel || "YouTube",
      thumbnailUrl: track.thumbnailUrl,
      duration: track.duration,
      videoUrl: track.videoUrl,
    };

    setStagingBlocks((prev) => {
      const existingIdx = prev.findIndex((b) => b.id === "youtube-single-searches");
      if (existingIdx >= 0) {
        const updated = [...prev];
        const targetBlock = updated[existingIdx];
        if (!targetBlock.tracks.some((t) => t.id === trackObj.id)) {
          updated[existingIdx] = {
            ...targetBlock,
            tracks: [...targetBlock.tracks, trackObj],
          };
        }
        return updated;
      } else {
        const newBlock: StagingBlockData = {
          id: "youtube-single-searches",
          title: "Búsquedas y vídeos sueltos",
          source: "youtube",
          tracks: [trackObj],
        };
        return [...prev, newBlock];
      }
    });

    setIsStagingDrawerOpen(true);
    setPlaylistNotice(`Añadida "${track.title}" al bloque de Búsquedas de YouTube.`);
  };

  const handleSendStagingToDownloads = (selectedTracks: StagingTrack[], sourceName: string) => {
    const exportTracks: ExportTrack[] = selectedTracks.map((t) => ({
      title: t.title,
      artists: t.artistOrChannel ? t.artistOrChannel.split(", ") : ["Desconocido"],
      albumName: t.albumName,
      albumImageUrl: t.thumbnailUrl,
      releaseDate: t.releaseDate,
    }));

    setPreloadedTracks(exportTracks);
    setPreloadedName(sourceName);
    setActiveTab("downloads");
  };

  const handleExportStagingTracks = async (selectedTracks: StagingTrack[], format: ExportFormat, name: string) => {
    const exportTracks: ExportTrack[] = selectedTracks.map((t) => ({
      title: t.title,
      artists: t.artistOrChannel ? t.artistOrChannel.split(", ") : ["Desconocido"],
      albumName: t.albumName,
      albumImageUrl: t.thumbnailUrl,
      releaseDate: t.releaseDate,
    }));

    const content = format === "json" ? exportJson(exportTracks) : exportTxt(exportTracks);
    const didSave = await saveExportedFile({
      content,
      format,
      playlistName: name,
    });

    if (didSave) {
      setPlaylistNotice(`Exportadas ${exportTracks.length} canciones en formato ${format.toUpperCase()}.`);
    }
  };

  const handleDeduplicateStagingBlocks = () => {
    const { newBlocks, removedCount } = deduplicateStagingBlocks(stagingBlocks);
    setStagingBlocks(newBlocks);
    if (removedCount > 0) {
      setPlaylistNotice(`Se eliminaron ${removedCount} canciones duplicadas de la Playlist Unificada.`);
    } else {
      setPlaylistNotice("No se encontraron canciones duplicadas en la Playlist Unificada.");
    }
  };

  const handleAddExportTracksToStaging = (tracks: ExportTrack[], sourceTitle: string) => {
    if (tracks.length === 0) return;
    const newBlock: StagingBlockData = {
      id: `block-${Date.now()}`,
      title: sourceTitle,
      source: "search",
      tracks: tracks.map((t, idx) => ({
        id: `${t.title}-${idx}-${Date.now()}`,
        title: t.title,
        artistOrChannel: t.artists.join(", "),
        albumName: t.albumName,
        releaseDate: t.releaseDate,
      })),
    };
    setStagingBlocks((prev) => [...prev, newBlock]);
    setPlaylistNotice(`Añadido el bloque "${sourceTitle}" a la Mesa de Trabajo (${tracks.length} canciones).`);
  };

  const totalStagingTracks = stagingBlocks.reduce((acc, b) => acc + b.tracks.length, 0);

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

        <button
          type="button"
          className="drawer-trigger-btn"
          onClick={() => setIsStagingDrawerOpen((prev) => !prev)}
          title="Abrir panel lateral de la Mesa de Trabajo"
        >
          <ListIcon style={{ width: 18, height: 18 }} />
          <span>Mesa de Trabajo</span>
          <span className="badge-count">{totalStagingTracks}</span>
        </button>

        <div
          className={isStagingDrawerOpen ? "drawer-overlay is-open" : "drawer-overlay"}
          onClick={() => setIsStagingDrawerOpen(false)}
          aria-label="Cerrar panel lateral de la Mesa de Trabajo"
        />

        <section className="hero-card">
          <div>
            <p className="eyebrow">Music Tools</p>
            <h1>Explora, compara y organiza tus playlists</h1>
            <p className="lead">
              Conéctate a Spotify, busca vídeos en YouTube y gestiona tu biblioteca local en un solo lugar.
            </p>
          </div>

          <div style={{ marginTop: "1.5rem" }}>
            <SubnavTabs activeTab={activeTab} onChange={setActiveTab} />
          </div>

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

              {spotifyFallbackWarning && (
                <div
                  style={{
                    backgroundColor: "rgba(33, 150, 243, 0.12)",
                    border: "1px solid rgba(33, 150, 243, 0.3)",
                    borderRadius: "8px",
                    padding: "0.75rem 1rem",
                    marginTop: "1rem",
                    display: "flex",
                    alignItems: "center",
                    justifyContent: "space-between",
                    gap: "0.75rem",
                    color: "#64b5f6",
                  }}
                >
                  <div style={{ display: "flex", alignItems: "center", gap: "0.6rem" }}>
                    <InfoIcon style={{ width: 20, height: 20, flexShrink: 0 }} />
                    <span style={{ fontSize: "0.88rem", lineHeight: 1.4 }}>
                      <strong>Modo de Carga Pública Activo:</strong> {spotifyFallbackWarning}
                    </span>
                  </div>
                  <button
                    type="button"
                    onClick={() => setSpotifyFallbackWarning(null)}
                    title="Cerrar aviso"
                    style={{
                      background: "transparent",
                      border: "none",
                      color: "inherit",
                      cursor: "pointer",
                      display: "inline-flex",
                      alignItems: "center",
                      padding: "4px",
                    }}
                  >
                    <CloseIcon style={{ width: 16, height: 16 }} />
                  </button>
                </div>
              )}

              {playlistError ? <p className="auth-error workspace-error">{playlistError}</p> : null}
              {playlistNotice ? <p className="workspace-notice">{playlistNotice}</p> : null}

              {universalPlaylist && (
                <UniversalPlaylistPreview
                  playlist={universalPlaylist}
                  onAddTracksToQueue={handleAddUniversalTracksToStaging}
                  onExportPlaylist={async (selectedTracks, format, name) => {
                    const exportTracks: ExportTrack[] = selectedTracks.map((t) => ({
                      title: t.title,
                      artists: [t.artistOrChannel],
                      albumImageUrl: t.thumbnailUrl,
                      duration: t.duration,
                    }));
                    const content = format === "json" ? exportJson(exportTracks) : exportTxt(exportTracks);
                    const didSave = await saveExportedFile({ content, format, playlistName: name });
                    if (didSave) {
                      setPlaylistNotice(`Exportadas ${exportTracks.length} canciones de "${name}" en formato ${format.toUpperCase()}.`);
                    }
                  }}
                  onClose={() => setUniversalPlaylist(null)}
                />
              )}

              {mediaSearchResults.length > 0 && (
                <MediaPreviewCard
                  results={mediaSearchResults}
                  title={mediaTitle}
                  onDownloadTrack={handleAddMediaTrackToStaging}
                  onPlayPreview={(track) => {
                    setActiveAudioTrack({ filePath: track.videoUrl, title: track.title, artists: [track.channel] });
                  }}
                  onClose={() => setMediaSearchResults([])}
                />
              )}

              {!universalPlaylist && mediaSearchResults.length === 0 && (
                <section className="workspace-panel workspace-empty-state">
                  <p className="section-label">Buscador y Carga Inteligente</p>
                  <h2>Explora, carga o busca canciones</h2>
                  <p className="workspace-copy">
                    Pega un enlace de Spotify, YouTube o realiza una búsqueda en la barra superior para previsualizar playlists y añadir canciones a tu Mesa de Trabajo.
                  </p>
                </section>
              )}
            </div>

            <div style={{ display: activeTab === "compare" ? "block" : "none" }}>
              <ComparisonPanel
                currentPlaylist={playlist}
                stagingBlocks={stagingBlocks}
                settingsVersion={settingsVersion}
                onSendToDownloads={(tracks, name) => {
                  setPreloadedTracks(tracks);
                  setPreloadedName(name);
                  setActiveTab("downloads");
                }}
                onAddTracksToStaging={handleAddExportTracksToStaging}
              />
            </div>

            <div style={{ display: activeTab === "utilities" ? "block" : "none" }}>
              <LocalUtilitiesPanel
                settingsVersion={settingsVersion}
                stagingBlocks={stagingBlocks}
                onPlayTrack={(track) => setActiveAudioTrack(track)}
                onAddTracksToStaging={handleAddExportTracksToStaging}
              />
            </div>

            <div style={{ display: activeTab === "downloads" ? "block" : "none" }}>
              <DownloadsPanel
                currentPlaylist={playlist}
                stagingBlocks={stagingBlocks}
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

      <StagingWorkspace
        blocks={stagingBlocks}
        onRemoveBlock={(blockId) => setStagingBlocks((prev) => prev.filter((b) => b.id !== blockId))}
        onClearAllBlocks={() => setStagingBlocks([])}
        onSendToDownloads={handleSendStagingToDownloads}
        onExportTracks={handleExportStagingTracks}
        onDeduplicateBlocks={handleDeduplicateStagingBlocks}
        onRemoveTrack={(blockId, trackId) => {
          setStagingBlocks((prev) =>
            prev
              .map((b) => (b.id === blockId ? { ...b, tracks: b.tracks.filter((t) => t.id !== trackId) } : b))
              .filter((b) => b.tracks.length > 0)
          );
        }}
        isOpen={isStagingDrawerOpen}
        onClose={() => setIsStagingDrawerOpen(false)}
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
