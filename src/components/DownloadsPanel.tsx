import { useState, useEffect, useRef, useCallback } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { loadSettings } from "../utils/settings";
import { parsePlaylistFile } from "../features/playlist/playlistFileLoader";
import type { ExportTrack } from "../types/export-track";
import { isMatch } from "../utils/songMatcher";
import { WarningIcon, RefreshIcon, PauseIcon, PlayIcon, TrashIcon, CloseIcon, YoutubeIcon, MusicIcon, CheckIcon, ZapIcon, FolderIcon, SearchIcon } from "./Icons";


import { stagingToExportTracks } from "../utils/duplicateDetector";
import type { StagingBlockData } from "./StagingWorkspace";

interface YoutubeSearchResult {
  id: string;
  title: string;
  channel: string;
  duration: string;
  thumbnailUrl: string;
  videoUrl: string;
}

interface QueueTrack {
  id: string; // "Artist - Title"
  title: string;
  artists: string[];
  albumName?: string;
  albumImageUrl?: string;
  releaseDate?: string;
  customUrl?: string;
  status: "waiting" | "searching" | "downloading" | "converting" | "completed" | "in_disk" | "failed";
  progress: number;
  speed?: string;
  eta?: string;
  error?: string;
  attempts: number;
}

type PlaylistSource = "unified" | "current" | "file";
type QueueFilter = "all" | "waiting" | "processing" | "completed" | "failed";

type DownloadsPanelProps = {
  currentPlaylist: { name: string; tracks: ExportTrack[] } | null;
  stagingBlocks?: StagingBlockData[];
  settingsVersion?: number;
  preloadedTracks?: ExportTrack[] | null;
  preloadedName?: string;
  onClearPreloaded?: () => void;
  onPlayTrack?: (track: { filePath: string; title: string; artists?: string[] }) => void;
};

function DownloadsPanel({
  currentPlaylist,
  stagingBlocks,
  settingsVersion,
  preloadedTracks,
  preloadedName,
  onClearPreloaded,
  onPlayTrack,
}: DownloadsPanelProps) {
  const [folderPath, setFolderPath] = useState(() => loadSettings().defaultSongDownloadFolder || loadSettings().defaultLocalMusicFolder);

  const sanitizeFilename = (name: string): string => {
    return name.replace(/[/\\:*?"<>|]/g, "_");
  };

  const handlePlayDownloadedTrack = (track: QueueTrack) => {
    if (!onPlayTrack) return;
    const cleanTitle = sanitizeFilename(track.title);
    const cleanArtists = sanitizeFilename(track.artists.join(", "));
    const filePath = `${folderPath}/${cleanArtists} - ${cleanTitle}.mp3`;
    onPlayTrack({ filePath, title: track.title, artists: track.artists });
  };
  const [playlistSource, setPlaylistSource] = useState<PlaylistSource>("current");
  const [playlistFilePath, setPlaylistFilePath] = useState("");
  const [playlistName, setPlaylistName] = useState("");

  // Requisitos del sistema
  const [isYtdlpInstalled, setIsYtdlpInstalled] = useState(false);
  const [isFfmpegInstalled, setIsFfmpegInstalled] = useState(true);
  const [isCheckingSystem, setIsCheckingSystem] = useState(true);
  const [installingStatus, setInstallingStatus] = useState<"idle" | "installing" | "done" | "error">("idle");
  const [installError, setInstallError] = useState<string | null>(null);

  // Cola de descargas
  const [queue, setQueue] = useState<QueueTrack[]>([]);
  const [isDownloadingQueue, setIsDownloadingQueue] = useState(false);
  const [existingMode, setExistingMode] = useState<"omit" | "overwrite">("omit");
  const [error, setError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);

  // Escanear carpeta de destino para omitir canciones ya existentes
  async function checkExistingLocalTracks() {
    if (!folderPath.trim()) return;
    try {
      const localFiles = await invoke<{ title?: string; artists: string[]; filename: string; path: string }[]>(
        "scan_local_folder",
        { path: folderPath }
      );

      setQueue((prev) =>
        prev.map((track) => {
          if (
            track.status === "downloading" ||
            track.status === "searching" ||
            track.status === "converting" ||
            track.status === "completed"
          ) {
            return track;
          }

          if (existingMode === "overwrite") {
            if (track.status === "in_disk") {
              return { ...track, status: "waiting", progress: 0 };
            }
            return track;
          }

          // existingMode === "omit"
          const match = localFiles.find((file) => {
            const fileTrack = {
              title: file.title || file.filename.replace(/\.[^/.]+$/, ""),
              artists: file.artists,
            };
            const res = isMatch(track, fileTrack);
            return res.matched;
          });

          if (match) {
            return {
              ...track,
              status: "in_disk",
              progress: 100,
            };
          } else if (track.status === "in_disk") {
            return {
              ...track,
              status: "waiting",
              progress: 0,
            };
          }

          return track;
        })
      );
    } catch (err) {
      console.warn("No se pudo escanear carpeta de destino para verificar canciones existentes:", err);
    }
  }

  // Selector Manual de YouTube (Cola de selecciones pendientes)
  const [manualSelectionQueue, setManualSelectionQueue] = useState<{
    track: QueueTrack;
    options: YoutubeSearchResult[];
  }[]>([]);

  const currentManualSelection = manualSelectionQueue[0] || null;

  function popManualSelection() {
    setManualSelectionQueue((prev) => prev.slice(1));
  }

  function handleOmitManualSelection() {
    if (!currentManualSelection) return;
    const item = currentManualSelection;
    popManualSelection();
    runSingleDownloadRef.current(item.track);
  }

  function handleCancelManualTrack() {
    if (!currentManualSelection) return;
    const item = currentManualSelection;
    popManualSelection();
    setQueue((prev) =>
      prev.map((t) =>
        t.id === item.track.id
          ? {
              ...t,
              status: "failed",
              error: "Descarga cancelada por el usuario",
              progress: 0,
            }
          : t
      )
    );
  }

  async function handleSelectVersionForTrack(track: QueueTrack) {
    try {
      setQueue((prev) =>
        prev.map((t) => (t.id === track.id ? { ...t, status: "searching", progress: 0 } : t))
      );
      const searchResults = await invoke<YoutubeSearchResult[]>("search_youtube", {
        query: `${track.artists.join(" ")} ${track.title}`,
      });
      if (searchResults && searchResults.length > 0) {
        setManualSelectionQueue((prev) => [...prev, { track, options: searchResults }]);
      } else {
        setError(`No se encontraron opciones de YouTube para "${track.title}"`);
        setQueue((prev) =>
          prev.map((t) => (t.id === track.id ? { ...t, status: track.status === "searching" ? "waiting" : track.status } : t))
        );
      }
    } catch (err) {
      console.error("Error al buscar opciones de YouTube:", err);
      setError(`Error al buscar opciones para "${track.title}"`);
      setQueue((prev) =>
        prev.map((t) => (t.id === track.id ? { ...t, status: track.status === "searching" ? "waiting" : track.status } : t))
      );
    }
  }

  async function handleRedownloadTrack(track: QueueTrack) {
    setQueue((prev) =>
      prev.map((t) => (t.id === track.id ? { ...t, status: "searching", progress: 0 } : t))
    );
    await runSingleDownloadRef.current(track);
  }

  // Buscador e interacción principal
  const [searchQuery, setSearchQuery] = useState("");



  // Modal Dialog "Ver Detalle"
  const [isExpanded, setIsExpanded] = useState(false);
  const [dialogSearch, setDialogSearch] = useState("");
  const [dialogFilter, setDialogFilter] = useState<QueueFilter>("all");
  const dialogRef = useRef<HTMLDialogElement>(null);

  // Refs para el bucle de descarga seguro y concurrente
  const queueRef = useRef<QueueTrack[]>([]);
  queueRef.current = queue;

  const isDownloadingRef = useRef(false);
  isDownloadingRef.current = isDownloadingQueue;

  const runSingleDownloadRef = useRef<(track: QueueTrack, selectedCustomUrl?: string) => Promise<void>>(async () => {});

  // Sincronizar ajustes al cambiar
  useEffect(() => {
    const settings = loadSettings();
    setFolderPath(settings.defaultSongDownloadFolder || settings.defaultLocalMusicFolder);
  }, [settingsVersion]);

  // Verificar canciones existentes en la carpeta de destino al cambiar de carpeta o modo
  useEffect(() => {
    if (folderPath && queue.length > 0) {
      checkExistingLocalTracks();
    }
  }, [folderPath, existingMode, queue.length]);

  // Cargar canciones pre-cargadas desde otros módulos (ej: Comparar Playlist)
  useEffect(() => {
    if (preloadedTracks && preloadedTracks.length > 0) {
      setPlaylistName(preloadedName || "Playlist Comparada");
      
      const newQueue: QueueTrack[] = preloadedTracks.map((track) => {
        const id = `${track.artists.join(", ")} - ${track.title}`;
        return {
          id,
          title: track.title,
          artists: track.artists,
          albumName: track.albumName,
          albumImageUrl: track.albumImageUrl,
          releaseDate: track.releaseDate,
          status: "waiting",
          progress: 0,
          attempts: 0,
        };
      });
      setQueue(newQueue);
      setError(null);
      setNotice(`Se cargaron ${newQueue.length} canciones faltantes en la cola de descarga.`);
      
      if (onClearPreloaded) {
        onClearPreloaded();
      }
    }
  }, [preloadedTracks, preloadedName, onClearPreloaded]);

  // Sincronizar apertura/cierre del diálogo modal
  useEffect(() => {
    const dialog = dialogRef.current;
    if (!dialog) return;
    if (isExpanded) {
      dialog.showModal();
    } else {
      dialog.close();
    }
  }, [isExpanded]);

  // Manejador de Escape para modales en DownloadsPanel
  useEffect(() => {
    if (!isExpanded && !currentManualSelection) return;
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        if (currentManualSelection) {
          handleOmitManualSelection();
        } else if (isExpanded) {
          handleCloseDialog();
        }
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [isExpanded, currentManualSelection, handleOmitManualSelection]);

  // Verificar requisitos de sistema
  useEffect(() => {
    async function checkSystem() {
      setIsCheckingSystem(true);
      try {
        const ytdlp = await invoke<boolean>("check_ytdlp_installed");
        const ffmpeg = await invoke<boolean>("check_ffmpeg_installed");
        setIsYtdlpInstalled(ytdlp);
        setIsFfmpegInstalled(ffmpeg);
      } catch (err) {
        console.error("Error al comprobar requisitos:", err);
      } finally {
        setIsCheckingSystem(false);
      }
    }
    checkSystem();
  }, []);

  // Escuchar eventos de progreso emitidos desde Rust
  useEffect(() => {
    let unlistenFn: (() => void) | null = null;

    const setupListener = async () => {
      const unlisten = await listen<{
        trackKey: string;
        percentage: number;
        speed: string;
        eta: string;
        status: string;
      }>("download-progress", (event) => {
        const { trackKey, percentage, speed, eta, status } = event.payload;
        setQueue((prevQueue) =>
          prevQueue.map((track) => {
            if (track.id === trackKey) {
              return {
                ...track,
                status: status as any,
                progress: percentage,
                speed,
                eta,
              };
            }
            return track;
          })
        );
      });
      unlistenFn = unlisten;
    };

    setupListener();

    return () => {
      if (unlistenFn) {
        unlistenFn();
      }
    };
  }, []);

  // Ref para `checkAndRun` para evitar dependencias circulares
  const checkAndRunRef = useRef<() => Promise<void>>(() => Promise.resolve());

  // Ejecutar descarga de una sola canción (individual o masiva)
  const runSingleDownload = useCallback(
    async (track: QueueTrack, selectedCustomUrl?: string) => {
      try {
        let customUrl = selectedCustomUrl || track.customUrl || null;

        // Si la selección manual está activada en Ajustes y no hemos seleccionado URL previa
        if (!customUrl && loadSettings().manualYoutubeSelection) {
          try {
            const searchResults = await invoke<YoutubeSearchResult[]>("search_youtube", {
              query: `${track.artists.join(" ")} ${track.title}`,
            });
            if (searchResults && searchResults.length > 0) {
              setManualSelectionQueue((prev) => [...prev, { track, options: searchResults }]);
              return;
            }
          } catch (searchErr) {
            console.warn("No se pudieron cargar opciones manuales, continuando descarga automática:", searchErr);
          }
        }

        setQueue((prev) =>
          prev.map((t) =>
            t.id === track.id ? { ...t, status: "downloading", progress: 0 } : t
          )
        );

        await invoke("download_track", {
          title: track.title,
          artists: track.artists,
          downloadDir: folderPath,
          albumName: track.albumName || null,
          coverUrl: track.albumImageUrl || null,
          releaseYear: track.releaseDate ? track.releaseDate.substring(0, 4) : null,
          customUrl,
        });

        setQueue((prev) =>
          prev.map((t) =>
            t.id === track.id ? { ...t, status: "completed", progress: 100 } : t
          )
        );
      } catch (err) {
        console.error("Error al descargar track:", track.id, err);
        setQueue((prev) =>
          prev.map((t) =>
            t.id === track.id
              ? {
                  ...t,
                  status: "failed",
                  error: String(err),
                  progress: 0,
                  attempts: t.attempts + 1,
                }
              : t
          )
        );
      } finally {
        if (isDownloadingRef.current) {
          setTimeout(() => {
            checkAndRunRef.current();
          }, 100);
        }
      }
    },
    [folderPath]
  );

  runSingleDownloadRef.current = runSingleDownload;

  // Bucle de descargas concurrentes
  const checkAndRun = useCallback(async () => {
    if (!isDownloadingRef.current) return;

    const MAX_CONCURRENT = 3; // Límite seguro de descargas simultáneas

    // Obtener cuántas canciones están actualmente procesándose
    const currentProcessing = queueRef.current.filter(
      (t) =>
        t.status === "searching" ||
        t.status === "downloading" ||
        t.status === "converting"
    );

    // Si ya alcanzamos el límite concurrente, no hacemos nada, esperaremos a que una termine
    if (currentProcessing.length >= MAX_CONCURRENT) return;

    // Espacios libres disponibles
    const slotsAvailable = MAX_CONCURRENT - currentProcessing.length;

    // Buscar las siguientes canciones elegibles
    const nextTracks = queueRef.current
      .filter(
        (t) => t.status === "waiting" || (t.status === "failed" && t.attempts < 5)
      )
      .slice(0, slotsAvailable);

    if (nextTracks.length === 0) {
      // Si no hay tareas procesándose ni canciones pendientes, la cola ha terminado
      if (currentProcessing.length === 0) {
        setIsDownloadingQueue(false);
        const failedCount = queueRef.current.filter((t) => t.status === "failed").length;
        if (failedCount > 0) {
          setError(`Descargas finalizadas. ${failedCount} canciones fallaron tras 5 intentos.`);
        } else {
          setNotice("¡Todas las descargas se han completado con éxito!");
        }
      }
      return;
    }

    // Lanzar las descargas de forma asíncrona (sin await) en los slots libres
    for (const track of nextTracks) {
      // Cambiar el estado inmediatamente en la cola para reservar el slot
      setQueue((prev) =>
        prev.map((t) =>
          t.id === track.id ? { ...t, status: "searching", progress: 0 } : t
        )
      );

      runSingleDownload(track);
    }
  }, [runSingleDownload]);

  checkAndRunRef.current = checkAndRun;

  useEffect(() => {
    if (isDownloadingQueue) {
      checkAndRun();
    }
  }, [isDownloadingQueue, checkAndRun]);

  // Instalar yt-dlp automáticamente
  async function handleInstallYtdlp() {
    setInstallingStatus("installing");
    setInstallError(null);
    try {
      await invoke("ensure_ytdlp_downloaded");
      setIsYtdlpInstalled(true);
      setInstallingStatus("done");
    } catch (err) {
      console.error(err);
      setInstallingStatus("error");
      setInstallError(err instanceof Error ? err.message : String(err));
    }
  }

  // Buscar carpeta de descargas
  async function selectDirectory() {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: "Selecciona una carpeta para guardar las canciones",
      });
      if (typeof selected === "string") {
        setFolderPath(selected);
        setError(null);
      }
    } catch (err) {
      console.error(err);
    }
  }

  // Cargar archivo exportado de playlist
  async function selectPlaylistFile() {
    try {
      const selected = await open({
        directory: false,
        multiple: false,
        filters: [{ name: "Playlists", extensions: ["json", "txt"] }],
        title: "Selecciona una playlist exportada",
      });
      if (typeof selected === "string") {
        setPlaylistFilePath(selected);
        const fileContent = await invoke<string>("read_playlist_file", { path: selected });
        const tracks = parsePlaylistFile(fileContent, selected);
        
        const filename = selected.split(/[/\\]/).pop() || "Playlist Importada";
        setPlaylistName(filename);

        loadTracksIntoQueue(tracks);
        setError(null);
      }
    } catch (err) {
      console.error(err);
      setError("No se pudo leer el archivo de playlist.");
    }
  }

  // Transformar tracks cargados a formato de cola
  function loadTracksIntoQueue(tracks: ExportTrack[]) {
    const newQueue: QueueTrack[] = tracks.map((track) => {
      const id = `${track.artists.join(", ")} - ${track.title}`;
      return {
        id,
        title: track.title,
        artists: track.artists,
        albumName: track.albumName,
        albumImageUrl: track.albumImageUrl,
        releaseDate: track.releaseDate,
        status: "waiting",
        progress: 0,
        attempts: 0,
      };
    });
    setQueue(newQueue);
    setError(null);
    setNotice(`Se cargaron ${newQueue.length} canciones en la cola de descarga.`);
  }

  // Usar Playlist Unificada (Mesa de Trabajo)
  function loadUnifiedPlaylist() {
    const tracks = stagingToExportTracks(stagingBlocks || []);
    if (tracks.length === 0) {
      setError("La Mesa de Trabajo (Playlist Unificada) está vacía. Añade canciones o playlists a la mesa primero.");
      return;
    }
    const name = stagingBlocks && stagingBlocks.length === 1 ? stagingBlocks[0].title : `Mesa de Trabajo (${stagingBlocks?.length || 0} fuentes)`;
    setPlaylistName(name);
    loadTracksIntoQueue(tracks);
  }

  // Usar playlist actual de pantalla
  function loadCurrentPlaylist() {
    if (!currentPlaylist) {
      setError("No hay ninguna playlist activa cargada en pantalla.");
      return;
    }
    setPlaylistName(currentPlaylist.name);
    loadTracksIntoQueue(currentPlaylist.tracks);
  }

  // Limpiar y resetear cola
  function handleClearQueue() {
    setQueue([]);
    setIsDownloadingQueue(false);
    setError(null);
    setNotice("Cola de descargas limpiada.");
  }

  // Reintentar canciones fallidas (reiniciar intentos)
  function handleRetryFailed() {
    setQueue(prev =>
      prev.map(t =>
        t.status === "failed"
          ? { ...t, status: "waiting", attempts: 0, error: undefined }
          : t
      )
    );
    setError(null);
    setNotice("Se restablecieron las canciones fallidas para un nuevo intento.");
  }

  // Eliminar canción individual de la cola
  function handleRemoveTrack(id: string) {
    setQueue(prev => prev.filter(t => t.id !== id));
  }

  // Cerrar diálogo modal
  const handleCloseDialog = () => {
    setIsExpanded(false);
    setDialogSearch("");
    setDialogFilter("all");
  };

  // Filtrado de la tabla principal
  const filteredQueue = queue.filter(
    (track) =>
      track.title.toLowerCase().includes(searchQuery.toLowerCase()) ||
      track.artists.some((a) => a.toLowerCase().includes(searchQuery.toLowerCase()))
  );

  // Filtrado del modal detallado
  const filteredDialogQueue = queue.filter((track) => {
    // 1. Filtrar por búsqueda
    const q = dialogSearch.toLowerCase().trim();
    const matchesSearch =
      !q ||
      track.title.toLowerCase().includes(q) ||
      track.artists.some((a) => a.toLowerCase().includes(q));

    if (!matchesSearch) return false;

    // 2. Filtrar por estado
    if (dialogFilter === "all") return true;
    if (dialogFilter === "waiting") return track.status === "waiting";
    if (dialogFilter === "completed") return track.status === "completed" || track.status === "in_disk";
    if (dialogFilter === "failed") return track.status === "failed";
    if (dialogFilter === "processing") {
      return (
        track.status === "searching" ||
        track.status === "downloading" ||
        track.status === "converting"
      );
    }

    return true;
  });

  // Estadísticas globales
  const stats = {
    total: queue.length,
    completed: queue.filter((t) => t.status === "completed" || t.status === "in_disk").length,
    downloading: queue.filter(
      (t) =>
        t.status === "downloading" ||
        t.status === "searching" ||
        t.status === "converting"
    ).length,
    failed: queue.filter((t) => t.status === "failed").length,
    waiting: queue.filter((t) => t.status === "waiting").length,
  };

  const hasFailedAttempts = queue.some(t => t.status === "failed");

  return (
    <div className="downloads-panel" style={{ display: "flex", flexDirection: "column", gap: "16px" }}>
      {/* TARJETA PRINCIPAL DEL GESTOR DE DESCARGAS */}
      <div style={{ background: "var(--bg-panel)", border: "1px solid var(--border-color)", borderRadius: "18px", padding: "20px", display: "flex", flexDirection: "column", gap: "16px" }}>
        
        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "flex-start", flexWrap: "wrap", gap: "12px" }}>
          <div>
            <span className="section-label">GESTOR DE DESCARGAS</span>
            <h2 style={{ margin: "2px 0 0 0" }}>Cola de Descargas MP3</h2>
            <span style={{ fontSize: "0.84rem", color: "var(--text-secondary)", display: "block", marginTop: "4px" }}>
              Carga playlists o la Mesa de Trabajo y descarga archivos de audio a MP3.
            </span>
          </div>

          <span className="status-pill" style={{ fontSize: "0.78rem", color: "var(--accent)" }}>
            {stats.downloading > 0 ? `${stats.downloading} Descarga(s) activa(s)` : "Sin descargas activas"} • {stats.completed} Completadas
          </span>
        </div>

        {/* Alerta de requisitos de sistema */}
        {!isCheckingSystem && (!isYtdlpInstalled || !isFfmpegInstalled) && (
          <div style={{ background: "rgba(235, 87, 87, 0.08)", border: "1px solid rgba(235, 87, 87, 0.3)", borderRadius: "12px", padding: "14px 18px", marginTop: "4px" }}>
            <h4 style={{ margin: "0 0 6px 0", color: "#eb5757", fontSize: "0.95rem" }}>Requisitos del Sistema Faltantes</h4>
            
            {!isYtdlpInstalled && (
              <div style={{ marginBottom: "10px" }}>
                <p style={{ fontSize: "0.85rem", color: "var(--text-secondary)", margin: 0 }}>
                  El motor de descarga (<strong>yt-dlp</strong>) no está instalado.
                </p>
                {installingStatus === "idle" && (
                  <button className="primary-button" onClick={handleInstallYtdlp} type="button" style={{ background: "#eb5757", color: "#fff", marginTop: "6px", fontSize: "0.8rem", padding: "6px 12px" }}>
                    Instalar yt-dlp automáticamente
                  </button>
                )}
                {installingStatus === "installing" && (
                  <p style={{ color: "var(--accent)", fontSize: "0.85rem", margin: "6px 0 0 0" }}>Descargando e instalando yt-dlp... Por favor espera.</p>
                )}
                {installingStatus === "done" && (
                  <p style={{ color: "var(--accent)", fontSize: "0.85rem", margin: "6px 0 0 0", display: "flex", alignItems: "center", gap: "4px" }}>
                    <CheckIcon /> Instalación completada.
                  </p>
                )}
                {installingStatus === "error" && (
                  <p style={{ color: "#eb5757", fontSize: "0.85rem", margin: "6px 0 0 0" }}>Error de instalación: {installError}</p>
                )}
              </div>
            )}

            {!isFfmpegInstalled && (
              <div>
                <p style={{ fontSize: "0.85rem", color: "var(--text-secondary)", margin: 0 }}>
                  <WarningIcon style={{ color: "#eb5757", marginRight: "4px" }} /> <strong>FFmpeg no detectado:</strong> La conversión a MP3 requiere <code>ffmpeg</code> en el PATH del sistema.
                </p>
              </div>
            )}
          </div>
        )}

        {/* Inputs de Configuración */}
        <div style={{ display: "flex", flexDirection: "column", gap: "14px", marginTop: "4px" }}>
          {/* Carpeta de Descargas */}
          <div>
            <label style={{ fontSize: "0.8rem", color: "var(--text-secondary)", display: "block", marginBottom: "4px" }}>1. Carpeta de destino (MP3)</label>
            <div style={{ display: "flex", gap: "10px", flexWrap: "wrap", alignItems: "center" }}>
              <div style={{ flex: 1, minWidth: "280px" }}>
                <input
                  type="text"
                  readOnly
                  placeholder="Selecciona carpeta para guardar los MP3..."
                  value={folderPath}
                  className="btn-action"
                  style={{ width: "100%", textAlign: "left", background: "var(--input-bg)", border: "1px solid var(--border-color)", borderRadius: "12px", padding: "10px 14px", fontSize: "0.85rem", color: "var(--text-primary)" }}
                />
              </div>
              <button
                type="button"
                className="secondary-button"
                onClick={selectDirectory}
                disabled={isDownloadingQueue}
                style={{ display: "inline-flex", alignItems: "center", gap: "4px" }}
              >
                <FolderIcon style={{ width: 14, height: 14 }} />
                <span>Examinar...</span>
              </button>
            </div>

            {/* Opciones para canciones ya existentes */}
            <div style={{ marginTop: "12px", background: "rgba(255, 255, 255, 0.03)", padding: "12px 14px", borderRadius: "10px", border: "1px solid rgba(255, 255, 255, 0.07)" }}>
              <label style={{ fontSize: "0.85rem", color: "rgba(244, 247, 245, 0.78)", fontWeight: 600, display: "block", marginBottom: "8px" }}>
                Si una canción ya existe en la carpeta de destino:
              </label>
              <div style={{ display: "flex", gap: "20px", flexWrap: "wrap" }}>
                <label style={{ display: "flex", alignItems: "center", gap: "8px", cursor: "pointer", fontSize: "0.85rem", color: existingMode === "omit" ? "var(--accent)" : "var(--text-secondary)" }}>
                  <input
                    type="radio"
                    name="existingMode"
                    value="omit"
                    checked={existingMode === "omit"}
                    onChange={() => setExistingMode("omit")}
                    disabled={isDownloadingQueue}
                    style={{ accentColor: "var(--accent)", cursor: "pointer" }}
                  />
                  <span>Omitir canciones ya descargadas (En disco)</span>
                </label>

                <label style={{ display: "flex", alignItems: "center", gap: "8px", cursor: "pointer", fontSize: "0.85rem", color: existingMode === "overwrite" ? "var(--accent)" : "var(--text-secondary)" }}>
                  <input
                    type="radio"
                    name="existingMode"
                    value="overwrite"
                    checked={existingMode === "overwrite"}
                    onChange={() => setExistingMode("overwrite")}
                    disabled={isDownloadingQueue}
                    style={{ accentColor: "var(--accent)", cursor: "pointer" }}
                  />
                  <span>Volver a descargar y sobrescribir todas</span>
                </label>
              </div>
            </div>
          </div>

          {/* Origen de la Playlist */}
          <div className="input-group" style={{ marginTop: "16px" }}>
            <label>2. Origen de la Playlist a descargar</label>
            <div className="playlist-source-selector" style={{ marginTop: "8px" }}>
              <label className="radio-label">
                <input
                  type="radio"
                  name="downloadSource"
                  checked={playlistSource === "unified"}
                  onChange={() => setPlaylistSource("unified")}
                  disabled={isDownloadingQueue}
                />
                <span>
                  Usar Playlist Unificada / Mesa de Trabajo ({stagingToExportTracks(stagingBlocks || []).length} canciones)
                </span>
              </label>
              <label className="radio-label" style={{ marginTop: "8px" }}>
                <input
                  type="radio"
                  name="downloadSource"
                  checked={playlistSource === "current"}
                  onChange={() => setPlaylistSource("current")}
                  disabled={isDownloadingQueue}
                />
                <span>Usar playlist activa de búsqueda ({currentPlaylist?.name || "ninguna"})</span>
              </label>
              <label className="radio-label" style={{ marginTop: "8px" }}>
                <input
                  type="radio"
                  name="downloadSource"
                  checked={playlistSource === "file"}
                  onChange={() => setPlaylistSource("file")}
                  disabled={isDownloadingQueue}
                />
                <span>Cargar archivo exportado (.json, .txt)</span>
              </label>
            </div>

            <div style={{ marginTop: "14px", display: "flex", gap: "10px" }}>
              {playlistSource === "unified" ? (
                <button
                  type="button"
                  className="primary-button"
                  onClick={loadUnifiedPlaylist}
                  disabled={isDownloadingQueue || (stagingBlocks || []).length === 0}
                  style={{ display: "inline-flex", alignItems: "center", gap: "6px" }}
                >
                  <ZapIcon /> Cargar de la Mesa de Trabajo ({stagingToExportTracks(stagingBlocks || []).length})
                </button>
              ) : playlistSource === "current" ? (
                <button
                  type="button"
                  className="secondary-button"
                  onClick={loadCurrentPlaylist}
                  disabled={!currentPlaylist || isDownloadingQueue}
                >
                  Cargar Playlist Actual
                </button>
              ) : (
                <button
                  type="button"
                  className="secondary-button"
                  onClick={selectPlaylistFile}
                  disabled={isDownloadingQueue}
                >
                  {playlistFilePath ? `Cambiar archivo: ${playlistFilePath.split(/[/\\]/).pop()}` : "Seleccionar archivo..."}
                </button>
              )}
            </div>
          </div>
        </div>

        {error && <p className="auth-error workspace-error">{error}</p>}
        {notice && <p className="workspace-notice">{notice}</p>}
      </div>

      {/* Cola de Descargas */}
      {queue.length > 0 && (
        <div className="results-container" style={{ marginTop: "28px" }}>
          <div className="results-header" style={{ display: "flex", justifyContent: "space-between", alignItems: "center", flexWrap: "wrap", gap: "12px" }}>
            <div>
              <h3>Cola de Descargas: <strong style={{ color: "var(--accent)" }}>{playlistName}</strong></h3>
              <p className="track-count" style={{ marginTop: "4px" }}>
                Completadas: {stats.completed}/{stats.total} | Activas: {stats.downloading} (Simultáneas) | Espera: {stats.waiting} {stats.failed > 0 && `| Fallidas: ${stats.failed}`}
              </p>
            </div>

            <div style={{ display: "flex", gap: "10px", alignItems: "center" }}>
              {hasFailedAttempts && !isDownloadingQueue && (
                <button
                  className="secondary-button"
                  onClick={handleRetryFailed}
                  type="button"
                  style={{ borderColor: "#f2c94c", color: "#f2c94c", background: "rgba(242, 201, 76, 0.05)" }}
                  title="Restablece las canciones fallidas para volver a intentar"
                >
                  <RefreshIcon style={{ marginRight: "6px" }} /> Reintentar Fallidas
                </button>
              )}

              {isDownloadingQueue ? (
                <button
                  className="secondary-button"
                  onClick={() => {
                    setIsDownloadingQueue(false);
                    setNotice("Descargas en cola pausadas.");
                  }}
                  type="button"
                  style={{ borderColor: "#eb5757", color: "#eb5757" }}
                >
                  <PauseIcon style={{ marginRight: "6px" }} /> Pausar
                </button>
              ) : (
                <button
                  className="primary-button"
                  onClick={() => {
                    if (!folderPath) {
                      setError("Por favor, selecciona una carpeta de destino.");
                      return;
                    }
                    if (!isYtdlpInstalled) {
                      setError("Debes instalar yt-dlp antes de descargar.");
                      return;
                    }
                    setIsDownloadingQueue(true);
                    setError(null);
                    setNotice("Iniciando descargas concurrentes...");
                  }}
                  type="button"
                  disabled={stats.waiting + stats.failed === 0}
                >
                  <PlayIcon style={{ marginRight: "6px" }} /> Descargar Cola
                </button>
              )}

              <button
                className="secondary-button"
                onClick={() => setIsExpanded(true)}
                type="button"
                style={{ display: "inline-flex", alignItems: "center", gap: "6px" }}
              >
                <SearchIcon style={{ width: 14, height: 14 }} />
                <span>Ver Detalle</span>
              </button>

              <button
                className="secondary-button"
                onClick={handleClearQueue}
                type="button"
                disabled={isDownloadingQueue}
                style={{ display: "inline-flex", alignItems: "center", gap: "6px" }}
              >
                <TrashIcon style={{ width: 14, height: 14 }} />
                <span>Limpiar Cola</span>
              </button>
            </div>
          </div>

          {/* Buscador Rápido */}
          <div className="search-input-box" style={{ marginTop: "16px" }}>
            <SearchIcon style={{ width: 14, height: 14 }} />
            <input
              type="text"
              placeholder="Buscar en la cola de descarga..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
            />
          </div>

          {/* Tabla de la Cola */}
          <div className="preview-table-shell" style={{ marginTop: "16px", maxHeight: "380px", overflowY: "auto" }}>
            <table className="preview-table">
              <thead>
                <tr>
                  <th style={{ width: "35%" }}>TÍTULO</th>
                  <th style={{ width: "25%" }}>ARTISTAS</th>
                  <th style={{ width: "160px" }}>PROGRESO</th>
                  <th style={{ width: "130px" }}>ESTADO</th>
                  <th style={{ width: "160px", textAlign: "center" }}>ACCIÓN</th>
                </tr>
              </thead>
              <tbody>
                {filteredQueue.slice(0, 50).map((track) => (
                  <tr key={track.id} style={{ opacity: track.status === "completed" ? 0.6 : 1 }}>
                    <td>
                      <strong>{track.title}</strong>
                      {track.error && (
                        <div style={{ color: "#eb5757", fontSize: "0.75rem", marginTop: "2px" }}>
                          Error: {track.error} {track.attempts > 0 && `(Intento ${track.attempts}/5)`}
                        </div>
                      )}
                    </td>
                    <td>{track.artists.join(", ") || <em style={{ opacity: 0.5 }}>Desconocido</em>}</td>
                    <td>
                      <div style={{ display: "flex", flexDirection: "column", gap: "4px" }}>
                        <div className="progress-bar-container" style={{ background: "rgba(255,255,255,0.06)", height: "8px", borderRadius: "4px", overflow: "hidden" }}>
                          <div
                            className="progress-bar-fill"
                            style={{
                              background: track.status === "failed" ? "#eb5757" : (track.status === "completed" ? "var(--accent)" : "var(--border-active)"),
                              height: "100%",
                              width: `${track.progress}%`,
                              transition: "width 0.3s ease",
                            }}
                          />
                        </div>
                        {(track.status === "downloading" || track.status === "converting" || track.status === "searching") && (
                          <div style={{ display: "flex", justifyContent: "space-between", fontSize: "0.7rem", opacity: 0.6 }}>
                            <span>{track.speed || (track.status === "searching" ? "Buscando..." : "")}</span>
                            <span>{track.eta ? `ETA: ${track.eta}` : ""}</span>
                          </div>
                        )}
                      </div>
                    </td>
                    <td>
                      <span
                        className="status-badge"
                        style={{
                          display: "inline-block",
                          padding: "3px 8px",
                          borderRadius: "4px",
                          fontSize: "0.75rem",
                          fontWeight: "bold",
                          textTransform: "uppercase",
                          backgroundColor:
                            track.status === "completed" || track.status === "in_disk"
                              ? "var(--accent-bg)"
                              : track.status === "failed"
                              ? "rgba(235, 87, 87, 0.12)"
                              : track.status === "downloading"
                              ? "var(--accent-bg)"
                              : track.status === "searching"
                              ? "rgba(242, 201, 76, 0.12)"
                              : "rgba(255, 255, 255, 0.05)",
                          color:
                            track.status === "completed" || track.status === "in_disk"
                              ? "var(--accent)"
                              : track.status === "failed"
                              ? "#eb5757"
                              : track.status === "downloading"
                              ? "var(--accent)"
                              : track.status === "searching"
                              ? "#f2c94c"
                              : "rgba(244, 247, 245, 0.6)",
                        }}
                      >
                        {track.status === "waiting" && "En espera"}
                        {track.status === "searching" && "Buscando..."}
                        {track.status === "downloading" && `Descargando (${Math.round(track.progress)}%)`}
                        {track.status === "converting" && "Ajustando MP3"}
                        {track.status === "completed" && "Completado"}
                        {track.status === "in_disk" && "En disco"}
                        {track.status === "failed" && `Fallido (${track.attempts}/5)`}
                      </span>
                    </td>
                    <td style={{ textAlign: "center", whiteSpace: "nowrap" }}>
                      <div style={{ display: "flex", alignItems: "center", justifyContent: "center", gap: "6px" }}>
                        {(track.status === "completed" || track.status === "in_disk") && onPlayTrack && (
                          <button
                            type="button"
                            className="secondary-button"
                            style={{ padding: "4px 8px", fontSize: "0.76rem", margin: 0, display: "inline-flex", alignItems: "center", gap: "4px" }}
                            onClick={() => handlePlayDownloadedTrack(track)}
                            title="Escuchar pista con el reproductor interno"
                          >
                            <PlayIcon style={{ color: "var(--accent)" }} />
                            <span>Escuchar</span>
                          </button>
                        )}

                        {(track.status === "completed" || track.status === "in_disk") && (
                          <button
                            type="button"
                            className="secondary-button"
                            style={{ padding: "4px 8px", fontSize: "0.76rem", margin: 0, display: "inline-flex", alignItems: "center", gap: "4px" }}
                            onClick={() => handleRedownloadTrack(track)}
                            disabled={isDownloadingQueue}
                            title="Forzar re-descarga de esta canción"
                          >
                            <RefreshIcon style={{ color: "var(--accent)" }} />
                            <span>Re-descargar</span>
                          </button>
                        )}

                        {(track.status === "completed" || track.status === "in_disk" || track.status === "waiting" || track.status === "failed") && (
                          <button
                            type="button"
                            className="secondary-button"
                            style={{ padding: "4px 8px", fontSize: "0.76rem", margin: 0, display: "inline-flex", alignItems: "center", gap: "4px" }}
                            onClick={() => handleSelectVersionForTrack(track)}
                            disabled={isDownloadingQueue}
                            title={track.status === "completed" || track.status === "in_disk" ? "Cambiar versión descargada de YouTube" : "Elegir versión de YouTube"}
                          >
                            <YoutubeIcon style={{ color: "#eb5757" }} />
                            <span>{track.status === "completed" || track.status === "in_disk" ? "Cambiar" : "Elegir"}</span>
                          </button>
                        )}

                        <button
                          type="button"
                          className="delete-track-btn"
                          style={{ padding: "4px 8px", fontSize: "0.8rem", margin: 0 }}
                          onClick={() => handleRemoveTrack(track.id)}
                          disabled={
                            isDownloadingQueue &&
                            (track.status === "downloading" ||
                              track.status === "searching" ||
                              track.status === "converting")
                          }
                          title="Quitar canción de la cola de descargas"
                        >
                          <TrashIcon />
                        </button>
                      </div>
                    </td>
                  </tr>
                ))}
                {filteredQueue.length > 50 && (
                  <tr>
                    <td colSpan={5} className="more-tracks-notice">
                      Mostrando las primeras 50 canciones de la cola. Haz clic en "Ver Detalle" para ver y gestionar la lista completa.
                    </td>
                  </tr>
                )}
              </tbody>
            </table>
          </div>
        </div>
      )}

      {/* Modal Dialog "Ver Detalle" */}
      <dialog
        ref={dialogRef}
        className="expanded-dialog"
        style={{ height: "min(92vh, 880px)", maxHeight: "92vh" }}
        onClose={handleCloseDialog}
        onClick={(e) => {
          if (e.target === dialogRef.current) {
            handleCloseDialog();
          }
        }}
      >
        <div className="dialog-content" style={{ display: "flex", flexDirection: "column", height: "100%" }}>
          <div className="dialog-header">
            <div>
              <span className="section-label">Detalle Completo de Cola de Descargas</span>
              <h3 style={{ margin: 0 }}>Gestión detallada de la cola</h3>
            </div>
            <button
              type="button"
              className="dialog-close-btn"
              onClick={handleCloseDialog}
              aria-label="Cerrar"
            >
              <CloseIcon />
            </button>
          </div>

          <div className="dialog-search-row" style={{ padding: "16px 24px 0 24px" }}>
            <div className="dialog-search-inputs" style={{ display: "flex", gap: "10px", flex: 1 }}>
              <input
                type="search"
                placeholder="Buscar canción o artista en la cola..."
                value={dialogSearch}
                onChange={(e) => setDialogSearch(e.target.value)}
                className="playlist-input dialog-search-input"
                style={{ flex: 1 }}
                autoFocus
              />
              <select
                value={dialogFilter}
                onChange={(e) => setDialogFilter(e.target.value as QueueFilter)}
                className="playlist-input dialog-filter-select"
                style={{ width: "220px" }}
              >
                <option value="all">Todos los estados</option>
                <option value="waiting">En espera</option>
                <option value="processing">Descargando/Procesando</option>
                <option value="completed">Completadas</option>
                <option value="failed">Fallidas</option>
              </select>
            </div>
            <span className="dialog-count" style={{ fontSize: "0.85rem", opacity: 0.7 }}>
              Mostrando {filteredDialogQueue.length} de {queue.length} canciones
            </span>
          </div>

          {/* Subnav-tabs internas del diálogo */}
          <div className="compare-result-tabs" style={{ padding: "12px 24px 0 24px" }}>
            {(
              [
                { id: "all", label: `Todas (${queue.length})` },
                { id: "waiting", label: `En espera (${stats.waiting})` },
                {
                  id: "processing",
                  label: `Procesando (${stats.downloading})`,
                },
                { id: "completed", label: `Completadas (${stats.completed})` },
                { id: "failed", label: `Fallidas (${stats.failed})` },
              ] as const
            ).map((tab) => (
              <button
                key={tab.id}
                type="button"
                className={dialogFilter === tab.id ? "compare-tab-pill active" : "compare-tab-pill"}
                onClick={() => setDialogFilter(tab.id)}
              >
                {tab.label}
              </button>
            ))}
          </div>

          {/* Tabla de canciones del diálogo modal */}
          <div className="dialog-table-shell" style={{ flex: 1, minHeight: 0, overflowY: "auto", margin: "16px 24px 24px 24px" }}>
            {filteredDialogQueue.length > 0 ? (
              <table className="preview-table dialog-table">
                <thead>
                  <tr>
                    <th style={{ width: "35%" }}>TÍTULO</th>
                    <th style={{ width: "25%" }}>ARTISTAS</th>
                    <th style={{ width: "160px" }}>PROGRESO</th>
                    <th style={{ width: "130px" }}>ESTADO</th>
                    <th style={{ width: "160px", textAlign: "center" }}>ACCIÓN</th>
                  </tr>
                </thead>
                <tbody>
                  {filteredDialogQueue.map((track) => (
                    <tr key={track.id} style={{ opacity: track.status === "completed" ? 0.6 : 1 }}>
                      <td>
                        <strong>{track.title}</strong>
                        {track.error && (
                          <div style={{ color: "#eb5757", fontSize: "0.75rem", marginTop: "2px" }}>
                            Error: {track.error} {track.attempts > 0 && `(Intento ${track.attempts}/5)`}
                          </div>
                        )}
                      </td>
                      <td>{track.artists.join(", ")}</td>
                      <td>
                        <div className="progress-bar-container" style={{ background: "rgba(255,255,255,0.06)", height: "8px", borderRadius: "4px", overflow: "hidden" }}>
                          <div
                            className="progress-bar-fill"
                            style={{
                              background: track.status === "failed" ? "#eb5757" : (track.status === "completed" ? "var(--accent)" : "var(--border-active)"),
                              height: "100%",
                              width: `${track.progress}%`,
                              transition: "width 0.3s ease",
                            }}
                          />
                        </div>
                      </td>
                      <td>
                        <span
                          className="status-badge"
                          style={{
                            display: "inline-block",
                            padding: "3px 8px",
                            borderRadius: "4px",
                            fontSize: "0.72rem",
                            fontWeight: "bold",
                            textTransform: "uppercase",
                            backgroundColor:
                              track.status === "completed" || track.status === "in_disk"
                                ? "var(--accent-bg)"
                                : track.status === "failed"
                                ? "rgba(235, 87, 87, 0.12)"
                                : track.status === "downloading"
                                ? "var(--accent-bg)"
                                : track.status === "searching"
                                ? "rgba(242, 201, 76, 0.12)"
                                : "rgba(255, 255, 255, 0.05)",
                            color:
                              track.status === "completed" || track.status === "in_disk"
                                ? "var(--accent)"
                                : track.status === "failed"
                                ? "#eb5757"
                                : track.status === "downloading"
                                ? "var(--accent)"
                                : track.status === "searching"
                                ? "#f2c94c"
                                : "rgba(244, 247, 245, 0.6)",
                          }}
                        >
                          {track.status === "waiting" && "En espera"}
                          {track.status === "searching" && "Buscando..."}
                          {track.status === "downloading" && `Descargando (${Math.round(track.progress)}%)`}
                          {track.status === "converting" && "Ajustando MP3"}
                          {track.status === "completed" && "Completado"}
                          {track.status === "in_disk" && "En disco"}
                          {track.status === "failed" && `Fallido (${track.attempts}/5)`}
                        </span>
                      </td>
                      <td style={{ textAlign: "center", whiteSpace: "nowrap" }}>
                        <div style={{ display: "flex", alignItems: "center", justifyContent: "center", gap: "6px" }}>
                          {(track.status === "completed" || track.status === "in_disk") && onPlayTrack && (
                            <button
                              type="button"
                              className="secondary-button"
                              style={{ padding: "4px 8px", fontSize: "0.76rem", margin: 0, display: "inline-flex", alignItems: "center", gap: "4px" }}
                              onClick={() => handlePlayDownloadedTrack(track)}
                              title="Escuchar pista con el reproductor interno"
                            >
                              <PlayIcon style={{ color: "var(--accent)" }} />
                              <span>Escuchar</span>
                            </button>
                          )}

                          {(track.status === "completed" || track.status === "in_disk") && (
                            <button
                              type="button"
                              className="secondary-button"
                              style={{ padding: "4px 8px", fontSize: "0.76rem", margin: 0, display: "inline-flex", alignItems: "center", gap: "4px" }}
                              onClick={() => handleRedownloadTrack(track)}
                              disabled={isDownloadingQueue}
                              title="Forzar re-descarga de esta canción"
                            >
                              <RefreshIcon style={{ color: "var(--accent)" }} />
                              <span>Re-descargar</span>
                            </button>
                          )}

                          {(track.status === "completed" || track.status === "in_disk" || track.status === "waiting" || track.status === "failed") && (
                            <button
                              type="button"
                              className="secondary-button"
                              style={{ padding: "4px 8px", fontSize: "0.76rem", margin: 0, display: "inline-flex", alignItems: "center", gap: "4px" }}
                              onClick={() => handleSelectVersionForTrack(track)}
                              disabled={isDownloadingQueue}
                              title={track.status === "completed" || track.status === "in_disk" ? "Cambiar versión descargada de YouTube" : "Elegir versión de YouTube"}
                            >
                              <YoutubeIcon style={{ color: "#eb5757" }} />
                              <span>{track.status === "completed" || track.status === "in_disk" ? "Cambiar" : "Elegir"}</span>
                            </button>
                          )}

                          <button
                            type="button"
                            className="delete-track-btn"
                            style={{ padding: "4px 8px", fontSize: "0.8rem", margin: 0 }}
                            onClick={() => handleRemoveTrack(track.id)}
                            disabled={isDownloadingQueue}
                            title="Quitar de la cola"
                          >
                            <TrashIcon />
                          </button>
                        </div>
                      </td>
                  </tr>
                ))}
              </tbody>
            </table>
          ) : (
            <p style={{ padding: "32px", textAlign: "center", opacity: 0.6 }}>
              Ninguna canción coincide con los criterios de búsqueda o filtrado.
            </p>
          )}
        </div>

        <div className="dialog-footer">
          <button type="button" className="secondary-button" onClick={handleCloseDialog}>
            Cerrar
          </button>
        </div>
      </div>
    </dialog>

    {/* MODAL DIALOG: SELECCIÓN MANUAL DE YOUTUBE */}
    {currentManualSelection && (
      <dialog className="expanded-dialog youtube-selection-dialog" open>
        <div className="dialog-content">
          <div className="dialog-header">
            <div style={{ display: "flex", alignItems: "center", gap: "10px" }}>
              <YoutubeIcon style={{ fontSize: "1.4rem", color: "#eb5757" }} />
              <h3>
                Selección Manual de YouTube
                {manualSelectionQueue.length > 1 && (
                  <span style={{ fontSize: "0.85rem", color: "var(--accent)", marginLeft: "10px", fontWeight: "normal" }}>
                    (1 de {manualSelectionQueue.length} pendientes)
                  </span>
                )}
              </h3>
            </div>
            <button
              className="dialog-close-btn"
              onClick={handleOmitManualSelection}
              type="button"
              aria-label="Cerrar (Omitir)"
              title="Omitir y descargar opción recomendada por defecto"
            >
              <CloseIcon />
            </button>
          </div>

          <div className="dialog-body" style={{ padding: "20px 24px" }}>
            <p style={{ fontSize: "0.9rem", color: "var(--text-secondary)", marginBottom: "16px", marginTop: 0 }}>
              Elige cuál de las siguientes opciones de YouTube deseas descargar para:{" "}
              <strong style={{ color: "var(--accent)" }}>
                {currentManualSelection.track.artists.join(", ")} - {currentManualSelection.track.title}
              </strong>
            </p>

            <div className="youtube-options-list">
              {currentManualSelection.options.map((opt) => (
                <div key={opt.id} className="youtube-option-card">
                  {opt.thumbnailUrl ? (
                    <div className="youtube-thumb-wrapper">
                      <img src={opt.thumbnailUrl} alt={opt.title} />
                      <span className="youtube-duration-badge">{opt.duration}</span>
                    </div>
                  ) : (
                    <div className="youtube-thumb-wrapper thumb-placeholder">
                      <MusicIcon style={{ fontSize: "1.4rem", color: "rgba(244, 247, 245, 0.5)" }} />
                      <span className="youtube-duration-badge">{opt.duration}</span>
                    </div>
                  )}

                  <div className="youtube-option-info">
                    <span className="youtube-option-title" title={opt.title}>
                      {opt.title}
                    </span>
                    <span className="youtube-option-channel">
                      Canal: <strong>{opt.channel}</strong>
                    </span>
                  </div>

                  <button
                    className="primary-button"
                    style={{ padding: "8px 16px", fontSize: "0.85rem", flexShrink: 0 }}
                    onClick={() => {
                      const trackToDownload = currentManualSelection.track;
                      const videoUrl = opt.videoUrl;
                      popManualSelection();
                      runSingleDownloadRef.current(trackToDownload, videoUrl);
                    }}
                    type="button"
                  >
                    Descargar
                  </button>
                </div>
              ))}
            </div>
          </div>

          <div className="dialog-footer" style={{ justifyContent: "space-between" }}>
            <button
              className="secondary-button"
              style={{ color: "#eb5757", borderColor: "rgba(235, 87, 87, 0.3)" }}
              onClick={handleCancelManualTrack}
              type="button"
            >
              Cancelar Descarga
            </button>

            <button
              className="secondary-button"
              onClick={handleOmitManualSelection}
              type="button"
            >
              Omitir (Descargar Recomendado)
            </button>
          </div>
        </div>
      </dialog>
    )}
    </div>
  );
}

export default DownloadsPanel;
