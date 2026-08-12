import { useState, useMemo, useEffect } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { openPath } from "@tauri-apps/plugin-opener";
import { invoke } from "@tauri-apps/api/core";
import { normalizeTitle, normalizeArtist, isMatchNormalized } from "../utils/songMatcher";
import { loadSettings } from "../utils/settings";
import { CopyIcon, TagIcon, PlayIcon, WarningIcon, CloseIcon } from "./Icons";

type LocalTrack = {
  path: string;
  title?: string;
  artists: string[];
  album?: string;
  year?: number;
  filename: string;
  size: number;
  normTitle: string;
  normArtists: string[];
};

type CorruptFile = {
  path: string;
  filename: string;
  size: number;
  errorReason: string;
};

type DuplicateGroup = {
  key: string;
  title?: string;
  sizeLabel?: string;
  tracks: LocalTrack[];
};

function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 Bytes";
  const k = 1024;
  const sizes = ["Bytes", "KB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + " " + sizes[i];
}

function getFileExtension(path: string): string {
  const dotIndex = path.lastIndexOf(".");
  if (dotIndex === -1) return "unknown";
  return path.substring(dotIndex + 1).toUpperCase();
}

import type { StagingBlockData } from "./StagingWorkspace";
import type { ExportTrack } from "../types/export-track";

type LocalUtilitiesPanelProps = {
  settingsVersion?: number;
  onPlayTrack?: (track: { filePath: string; title: string; artists?: string[] }) => void;
  stagingBlocks?: StagingBlockData[];
  onAddTracksToStaging?: (tracks: ExportTrack[], sourceTitle: string) => void;
};

function LocalUtilitiesPanel({
  settingsVersion,
  onPlayTrack,
  stagingBlocks,
  onAddTracksToStaging,
}: LocalUtilitiesPanelProps) {
  const stagingCount = stagingBlocks?.reduce((acc, b) => acc + b.tracks.length, 0) || 0;
  const [folderPath, setFolderPath] = useState(() => loadSettings().defaultLocalMusicFolder);

  // Sync settings when settingsVersion changes
  useEffect(() => {
    setFolderPath(loadSettings().defaultLocalMusicFolder);
  }, [settingsVersion]);
  const [utilityType, setUtilityType] = useState<"duplicates" | "corrupt">("duplicates");
  const [tracks, setTracks] = useState<LocalTrack[]>([]);
  const [corruptFiles, setCorruptFiles] = useState<CorruptFile[]>([]);
  const [isScanning, setIsScanning] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);

  // Tab de filtro para duplicados: "exact" (por tamaño de archivo) o "semantic" (por metadatos)
  const [activeSubTab, setActiveSubTab] = useState<"exact" | "semantic">("exact");

  // Buscador de duplicados
  const [searchQuery, setSearchQuery] = useState("");
  const [searchScope, setSearchScope] = useState<"all" | "title" | "artist" | "path">("all");

  // Buscador de corruptos
  const [corruptSearchQuery, setCorruptSearchQuery] = useState("");

  // Estado de confirmación de borrado
  const [deletingPath, setDeletingPath] = useState<string | null>(null);
  const [isDeleting, setIsDeleting] = useState(false);

  // Tag Editor modal state
  const [editingTrack, setEditingTrack] = useState<{
    path: string;
    title: string;
    artists: string;
    album: string;
    year: string;
  } | null>(null);
  const [isSavingTags, setIsSavingTags] = useState(false);

  function handleOpenEditTags(track: LocalTrack) {
    const dotIdx = track.filename.lastIndexOf(".");
    const filenameNoExt = dotIdx !== -1 ? track.filename.substring(0, dotIdx) : track.filename;
    setEditingTrack({
      path: track.path,
      title: track.title || filenameNoExt,
      artists: track.artists.join(", "),
      album: track.album || "",
      year: track.year ? String(track.year) : "",
    });
  }

  async function handleSaveTags() {
    if (!editingTrack) return;
    setIsSavingTags(true);
    setError(null);
    try {
      const artistsList = editingTrack.artists
        .split(",")
        .map((s) => s.trim())
        .filter(Boolean);
      const parsedYear = editingTrack.year.trim() ? parseInt(editingTrack.year.trim(), 10) : undefined;
      await invoke("save_track_tags", {
        filePath: editingTrack.path,
        title: editingTrack.title.trim(),
        artists: artistsList,
        album: editingTrack.album.trim() || null,
        year: parsedYear && !isNaN(parsedYear) ? parsedYear : null,
      });
      setNotice("Metadatos del archivo guardados correctamente.");
      setEditingTrack(null);
      if (folderPath) {
        handleScan();
      }
    } catch (err) {
      console.error(err);
      setError("Error al guardar etiquetas en el archivo: " + String(err));
    } finally {
      setIsSavingTags(false);
    }
  }

  // Selector de carpeta
  async function selectDirectory() {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: "Selecciona una carpeta de música para analizar",
      });
      if (typeof selected === "string") {
        setFolderPath(selected);
        setError(null);
        setTracks([]);
        setCorruptFiles([]);
        setNotice(null);
      }
    } catch (err) {
      console.error(err);
      setError("No se pudo seleccionar la carpeta.");
    }
  }

  // Ejecución del escaneo
  async function handleScan() {
    if (!folderPath) {
      setError("Por favor, selecciona una carpeta primero.");
      return;
    }

    setIsScanning(true);
    setError(null);
    setNotice("Analizando la carpeta seleccionada en busca de archivos de música...");
    setTracks([]);
    setCorruptFiles([]);

    try {
      if (utilityType === "duplicates") {
        const scannedTracks = await invoke<LocalTrack[]>("scan_local_folder", { path: folderPath });
        const normTracks = scannedTracks.map((t) => ({
          ...t,
          normTitle: normalizeTitle(t.title || t.filename),
          normArtists: t.artists.map(normalizeArtist),
        }));
        setTracks(normTracks);
        setNotice(`Escaneo completado. Se encontraron ${scannedTracks.length} archivos de audio.`);
      } else {
        const scannedCorrupt = await invoke<CorruptFile[]>("scan_corrupt_files", { path: folderPath });
        setCorruptFiles(scannedCorrupt);
        setNotice(`Escaneo completado. Se encontraron ${scannedCorrupt.length} archivos dañados o corruptos.`);
      }
    } catch (err) {
      console.error(err);
      setError(err instanceof Error ? err.message : "Error durante el escaneo de la carpeta.");
      setNotice(null);
    } finally {
      setIsScanning(false);
    }
  }

  // Agrupamiento por Tamaño (Exactos)
  const exactGroups = useMemo<DuplicateGroup[]>(() => {
    const groupsMap = new Map<number, LocalTrack[]>();
    for (const track of tracks) {
      const size = track.size || 0;
      if (size === 0) continue;
      if (!groupsMap.has(size)) {
        groupsMap.set(size, []);
      }
      groupsMap.get(size)!.push(track);
    }

    const result: DuplicateGroup[] = [];
    for (const [size, groupTracks] of groupsMap.entries()) {
      if (groupTracks.length > 1) {
        result.push({
          key: `exact-${size}`,
          sizeLabel: formatBytes(size),
          tracks: groupTracks,
        });
      }
    }
    return result.sort((a, b) => {
      const sizeA = a.tracks[0]?.size || 0;
      const sizeB = b.tracks[0]?.size || 0;
      return sizeB - sizeA;
    });
  }, [tracks]);

  // Agrupamiento por Metadatos (Semánticos)
  const semanticGroups = useMemo<DuplicateGroup[]>(() => {
    const titleGroups = new Map<string, LocalTrack[]>();
    for (const track of tracks) {
      const normTitle = track.normTitle;
      if (!normTitle) continue;
      if (!titleGroups.has(normTitle)) {
        titleGroups.set(normTitle, []);
      }
      titleGroups.get(normTitle)!.push(track);
    }

    const result: DuplicateGroup[] = [];
    let groupIndex = 0;

    for (const [normTitle, groupTracks] of titleGroups.entries()) {
      if (groupTracks.length < 2) continue;

      const subGroups: LocalTrack[][] = [];

      const threshold = loadSettings().fuzzyThreshold;

      for (const track of groupTracks) {
        let added = false;
        for (const subGroup of subGroups) {
          const representative = subGroup[0];
          const match = isMatchNormalized(track, representative, threshold);
          if (match.matched) {
            subGroup.push(track);
            added = true;
            break;
          }
        }
        if (!added) {
          subGroups.push([track]);
        }
      }

      for (const subGroup of subGroups) {
        if (subGroup.length > 1) {
          const representative = subGroup[0];
          const displayTitle = representative.title || representative.filename;
          const displayArtists = representative.artists.length > 0 ? ` - ${representative.artists.join(", ")}` : "";
          result.push({
            key: `semantic-${normTitle}-${groupIndex++}`,
            title: `${displayTitle}${displayArtists}`,
            tracks: subGroup,
          });
        }
      }
    }

    return result;
  }, [tracks]);

  // Filtrado de duplicados según búsqueda
  const filteredGroups = useMemo(() => {
    const currentGroups = activeSubTab === "exact" ? exactGroups : semanticGroups;
    if (!searchQuery.trim()) return currentGroups;

    const q = searchQuery.toLowerCase().trim();
    return currentGroups
      .map((g) => {
        const filteredTracks = g.tracks.filter((t) => {
          const matchesTitle = (t.title || t.filename).toLowerCase().includes(q);
          const matchesArtist = t.artists.some((a) => a.toLowerCase().includes(q));
          const matchesPath = t.path.toLowerCase().includes(q);

          if (searchScope === "title") return matchesTitle;
          if (searchScope === "artist") return matchesArtist;
          if (searchScope === "path") return matchesPath;
          return matchesTitle || matchesArtist || matchesPath;
        });

        return { ...g, tracks: filteredTracks };
      })
      .filter((g) => g.tracks.length > 1);
  }, [activeSubTab, exactGroups, semanticGroups, searchQuery, searchScope]);

  // Filtrado de archivos corruptos según búsqueda
  const filteredCorruptFiles = useMemo(() => {
    if (!corruptSearchQuery.trim()) return corruptFiles;
    const q = corruptSearchQuery.toLowerCase().trim();
    return corruptFiles.filter(
      (f) =>
        f.filename.toLowerCase().includes(q) ||
        f.path.toLowerCase().includes(q) ||
        f.errorReason.toLowerCase().includes(q)
    );
  }, [corruptFiles, corruptSearchQuery]);

  // Borrado de archivo duplicado
  async function handleDeleteFile(path: string) {
    setIsDeleting(true);
    setError(null);
    try {
      await invoke("delete_file", { path });
      setTracks((prev) => prev.filter((t) => t.path !== path));
      setNotice("Archivo eliminado con éxito.");
      setDeletingPath(null);
    } catch (err) {
      console.error(err);
      setError(err instanceof Error ? err.message : "No se pudo eliminar el archivo.");
    } finally {
      setIsDeleting(false);
    }
  }

  // Borrado de archivo dañado/corrupto
  async function handleDeleteCorrupt(path: string) {
    setIsDeleting(true);
    setError(null);
    try {
      await invoke("delete_file", { path });
      setCorruptFiles((prev) => prev.filter((f) => f.path !== path));
      setNotice("Archivo dañado eliminado con éxito del disco.");
      setDeletingPath(null);
    } catch (err) {
      console.error(err);
      setError(err instanceof Error ? err.message : "No se pudo eliminar el archivo.");
    } finally {
      setIsDeleting(false);
    }
  }

  // Reproducir archivo localmente
  async function handlePlayTrack(path: string, title?: string, artists?: string[]) {
    const filename = path.substring(Math.max(path.lastIndexOf("/"), path.lastIndexOf("\\")) + 1);
    if (onPlayTrack) {
      onPlayTrack({
        filePath: path,
        title: title || filename,
        artists: artists || [],
      });
    } else {
      try {
        await openPath(path);
        setNotice("Abriendo archivo de audio con el reproductor del sistema...");
        setTimeout(() => setNotice(null), 3000);
      } catch (err) {
        console.error(err);
        setError("No se pudo reproducir el archivo de audio.");
      }
    }
  }

  // Copiar ruta al portapapeles
  async function copyToClipboard(text: string) {
    try {
      await navigator.clipboard.writeText(text);
      setNotice("Ruta copiada al portapapeles.");
      setTimeout(() => setNotice(null), 3000);
    } catch (err) {
      console.error("Error al copiar al portapapeles:", err);
    }
  }

  // Cambiar utilidad reseteando notificaciones
  function handleUtilityTabChange(type: "duplicates" | "corrupt") {
    setUtilityType(type);
    setError(null);
    setNotice(null);
    setSearchQuery("");
    setCorruptSearchQuery("");
  }

  const handleSendLocalTracksToStaging = () => {
    if (!onAddTracksToStaging || tracks.length === 0) return;
    const exportTracks: ExportTrack[] = tracks.map((t) => ({
      title: t.title || t.filename,
      artists: t.artists.length > 0 ? t.artists : ["Desconocido"],
      albumName: t.album,
      releaseDate: t.year ? String(t.year) : undefined,
    }));
    const folderName = folderPath.substring(Math.max(folderPath.lastIndexOf("/"), folderPath.lastIndexOf("\\")) + 1) || "Carpeta Local";
    onAddTracksToStaging(exportTracks, `Archivos locales (${folderName})`);
    setNotice(`Añadidas ${exportTracks.length} canciones locales a la Playlist Unificada.`);
  };

  return (
    <section className="workspace-panel">
      <div className="panel-header-section">
        <p className="section-label">Utilidades locales</p>
        <h2>Herramientas de Limpieza del Disco</h2>
        
        {/* Selector de Utilidad Principal */}
        <div className="subnav-tabs main-utility-tabs" style={{ marginTop: "16px", marginBottom: "8px" }}>
          <button
            aria-pressed={utilityType === "duplicates"}
            className={utilityType === "duplicates" ? "subnav-tab is-active" : "subnav-tab"}
            onClick={() => handleUtilityTabChange("duplicates")}
            type="button"
          >
            Detector de Duplicados en Disco
          </button>
          <button
            aria-pressed={utilityType === "corrupt"}
            className={utilityType === "corrupt" ? "subnav-tab is-active" : "subnav-tab"}
            onClick={() => handleUtilityTabChange("corrupt")}
            type="button"
          >
            Detector de Archivos Dañados
          </button>
        </div>

        <p className="workspace-copy" style={{ marginTop: "12px" }}>
          {utilityType === "duplicates"
            ? "Escanea tu biblioteca de música local en busca de pistas repetidas con el mismo tamaño exacto o con metadatos de audio idénticos."
            : "Escanea tu carpeta local en busca de archivos de música con cabeceras de audio dañadas, vacíos (0 bytes) o con errores al leer los metadatos."}
        </p>
      </div>

      {/* Selector de Carpeta */}
      <div className="directory-selector-row">
        <input
          className="playlist-input read-only-input"
          placeholder="Selecciona una carpeta en tu disco..."
          readOnly
          type="text"
          value={folderPath}
        />
        <button className="secondary-button" onClick={selectDirectory} type="button">
          Buscar Carpeta
        </button>
        <button
          className="primary-button"
          disabled={!folderPath || isScanning}
          onClick={handleScan}
          type="button"
        >
          {isScanning ? "Analizando..." : "Escanear"}
        </button>
        {tracks.length > 0 && onAddTracksToStaging && (
          <button
            className="secondary-button"
            onClick={handleSendLocalTracksToStaging}
            type="button"
            title={`Enviar las canciones escaneadas de la carpeta a la Mesa de Trabajo (Actualmente en Mesa: ${stagingCount})`}
          >
            ➕ Enviar a Mesa de Trabajo ({tracks.length})
          </button>
        )}
      </div>

      {error ? <p className="auth-error workspace-error">{error}</p> : null}
      {notice ? <p className="workspace-notice">{notice}</p> : null}

      {/* RENDERIZADOR: DETECTOR DE DUPLICADOS */}
      {utilityType === "duplicates" && tracks.length > 0 && (
        <div className="duplicates-workspace">
          <div className="duplicates-controls-row">
            <div className="subnav-tabs duplicates-tabs">
              <button
                aria-pressed={activeSubTab === "exact"}
                className={activeSubTab === "exact" ? "subnav-tab is-active" : "subnav-tab"}
                onClick={() => {
                  setActiveSubTab("exact");
                  setSearchQuery("");
                }}
                type="button"
              >
                Exacto (Tamaño: {exactGroups.length} grupos)
              </button>
              <button
                aria-pressed={activeSubTab === "semantic"}
                className={activeSubTab === "semantic" ? "subnav-tab is-active" : "subnav-tab"}
                onClick={() => {
                  setActiveSubTab("semantic");
                  setSearchQuery("");
                }}
                type="button"
              >
                Semántico (Etiquetas: {semanticGroups.length} grupos)
              </button>
            </div>

            <div className="dialog-search-inputs duplicates-search">
              <input
                className="playlist-input dialog-search-input"
                onChange={(e) => setSearchQuery(e.target.value)}
                placeholder="Filtrar duplicados..."
                type="text"
                value={searchQuery}
              />
              <select
                className="playlist-input dialog-filter-select"
                onChange={(e) => setSearchScope(e.target.value as any)}
                value={searchScope}
              >
                <option value="all">Todo</option>
                <option value="title">Título</option>
                <option value="artist">Artista</option>
                <option value="path">Ruta</option>
              </select>
            </div>
          </div>

          <div className="duplicate-groups-list">
            {filteredGroups.length === 0 ? (
              <div className="workspace-empty-state" style={{ marginTop: "24px" }}>
                <h3>No se encontraron duplicados</h3>
                <p className="workspace-copy">
                  {searchQuery.trim()
                    ? "Prueba modificando los criterios del buscador."
                    : "¡Felicidades! Tu biblioteca local en esta carpeta está limpia de duplicados en esta categoría."}
                </p>
              </div>
            ) : (
              filteredGroups.map((group) => (
                <div className="duplicate-group-card" key={group.key}>
                  <div className="duplicate-group-header">
                    {activeSubTab === "exact" ? (
                      <span className="duplicate-badge exact-badge">
                        Mismo Tamaño: {group.sizeLabel}
                      </span>
                    ) : (
                      <span className="duplicate-title-label">
                        Coincidencia: <strong>{group.title}</strong>
                      </span>
                    )}
                    <span className="duplicate-count-badge">
                      {group.tracks.length} archivos
                    </span>
                  </div>

                  <div className="duplicate-tracks-list">
                    {group.tracks.map((track) => {
                      const extension = getFileExtension(track.path);
                      return (
                        <div className="duplicate-track-row" key={track.path}>
                          <div className="track-info-col">
                            <div className="track-primary-info">
                              <span className="file-extension-badge">{extension}</span>
                              <span className="track-filename" title={track.filename}>
                                {track.filename}
                              </span>
                              <span className="track-filesize">
                                ({formatBytes(track.size)})
                              </span>
                            </div>
                            <div className="track-secondary-info">
                              <span className="track-fullpath" title={track.path}>
                                {track.path}
                              </span>
                              <button
                                className="copy-path-btn"
                                onClick={() => copyToClipboard(track.path)}
                                title="Copiar ruta"
                                type="button"
                              >
                                <CopyIcon />
                              </button>
                            </div>
                            {track.title ? (
                              <div className="track-metadata-preview">
                                <TagIcon style={{ marginRight: "4px" }} /> {track.title} {track.artists.length > 0 ? `· ${track.artists.join(", ")}` : ""}
                              </div>
                            ) : null}
                          </div>

                          <div className="track-action-col">
                            {deletingPath !== track.path && (
                              <>
                                <button
                                  className="secondary-button"
                                  onClick={() => handlePlayTrack(track.path, track.title, track.artists)}
                                  style={{ marginRight: "8px", padding: "6px 12px", fontSize: "0.85rem" }}
                                  title="Reproducir canción"
                                  type="button"
                                >
                                  <PlayIcon style={{ marginRight: "4px" }} /> Reproducir
                                </button>
                                <button
                                  className="secondary-button"
                                  onClick={() => handleOpenEditTags(track)}
                                  style={{ marginRight: "8px", padding: "6px 12px", fontSize: "0.85rem" }}
                                  title="Editar metadatos ID3 del archivo"
                                  type="button"
                                >
                                  <TagIcon style={{ marginRight: "4px" }} /> Editar Tags
                                </button>
                              </>
                            )}
                            {deletingPath === track.path ? (
                              <div className="confirm-delete-box">
                                <span className="confirm-label">¿Borrar del disco?</span>
                                <button
                                  className="action-btn danger-confirm-btn"
                                  disabled={isDeleting}
                                  onClick={() => handleDeleteFile(track.path)}
                                  type="button"
                                >
                                  Sí
                                </button>
                                <button
                                  className="action-btn cancel-confirm-btn"
                                  disabled={isDeleting}
                                  onClick={() => setDeletingPath(null)}
                                  type="button"
                                >
                                  No
                                </button>
                              </div>
                            ) : (
                              <button
                                className="delete-track-btn"
                                onClick={() => setDeletingPath(track.path)}
                                title="Borrar archivo definitivamente del disco"
                                type="button"
                              >
                                Borrar
                              </button>
                            )}
                          </div>
                        </div>
                      );
                    })}
                  </div>
                </div>
              ))
            )}
          </div>
        </div>
      )}

      {/* RENDERIZADOR: DETECTOR DE ARCHIVOS DAÑADOS */}
      {utilityType === "corrupt" && corruptFiles.length > 0 && (
        <div className="duplicates-workspace">
          <div className="duplicates-controls-row">
            <div>
              <h3 style={{ margin: 0, color: "#f4f7f5" }}>Archivos Dañados Encontrados ({filteredCorruptFiles.length})</h3>
            </div>
            
            <div className="dialog-search-inputs duplicates-search">
              <input
                className="playlist-input dialog-search-input"
                onChange={(e) => setCorruptSearchQuery(e.target.value)}
                placeholder="Filtrar archivos dañados..."
                type="text"
                value={corruptSearchQuery}
              />
            </div>
          </div>

          <div className="duplicate-groups-list">
            {filteredCorruptFiles.length === 0 ? (
              <div className="workspace-empty-state" style={{ marginTop: "24px" }}>
                <h3>No se encontraron archivos dañados</h3>
                <p className="workspace-copy">
                  Prueba modificando la búsqueda o felicítate por una biblioteca de música saludable.
                </p>
              </div>
            ) : (
              <div className="duplicate-group-card">
                <div className="duplicate-group-header" style={{ background: "rgba(235, 87, 87, 0.08)", borderBottom: "1px solid rgba(235, 87, 87, 0.2)" }}>
                  <span className="duplicate-badge" style={{ background: "rgba(235, 87, 87, 0.15)", color: "#eb5757", border: "1px solid rgba(235, 87, 87, 0.3)" }}>
                    Archivos con errores de lectura/decodificación
                  </span>
                  <span className="duplicate-count-badge" style={{ background: "rgba(235, 87, 87, 0.1)", color: "#eb5757" }}>
                    {filteredCorruptFiles.length} detectados
                  </span>
                </div>

                <div className="duplicate-tracks-list">
                  {filteredCorruptFiles.map((file) => {
                    const extension = getFileExtension(file.path);
                    return (
                      <div className="duplicate-track-row" key={file.path}>
                        <div className="track-info-col">
                          <div className="track-primary-info">
                            <span className="file-extension-badge" style={{ background: "#3d1e1e", color: "#eb5757" }}>{extension}</span>
                            <span className="track-filename" title={file.filename}>
                              {file.filename}
                            </span>
                            <span className="track-filesize">
                              ({formatBytes(file.size)})
                            </span>
                          </div>
                          
                          <div className="track-secondary-info">
                            <span className="track-fullpath" title={file.path}>
                              {file.path}
                            </span>
                            <button
                              className="copy-path-btn"
                              onClick={() => copyToClipboard(file.path)}
                              title="Copiar ruta"
                              type="button"
                            >
                              <CopyIcon />
                            </button>
                          </div>
                          
                          <div className="track-metadata-preview" style={{ background: "rgba(235, 87, 87, 0.05)", color: "#eb5757" }}>
                            <WarningIcon style={{ marginRight: "4px" }} /> Razón: {file.errorReason}
                          </div>
                        </div>

                        <div className="track-action-col">
                          {deletingPath !== file.path && (
                            <button
                              className="secondary-button"
                              onClick={() => handlePlayTrack(file.path)}
                              style={{ marginRight: "8px", padding: "6px 12px", fontSize: "0.85rem" }}
                              title="Reproducir archivo de audio"
                              type="button"
                            >
                              <PlayIcon style={{ marginRight: "4px" }} /> Reproducir
                            </button>
                          )}
                          {deletingPath === file.path ? (
                            <div className="confirm-delete-box">
                              <span className="confirm-label">¿Borrar?</span>
                              <button
                                className="action-btn danger-confirm-btn"
                                disabled={isDeleting}
                                onClick={() => handleDeleteCorrupt(file.path)}
                                type="button"
                              >
                                Sí
                              </button>
                              <button
                                className="action-btn cancel-confirm-btn"
                                disabled={isDeleting}
                                onClick={() => setDeletingPath(null)}
                                type="button"
                              >
                                No
                              </button>
                            </div>
                          ) : (
                            <button
                              className="delete-track-btn"
                              onClick={() => setDeletingPath(file.path)}
                              title="Borrar archivo corrupto definitivamente del disco"
                              type="button"
                            >
                              Borrar
                            </button>
                          )}
                        </div>
                      </div>
                    );
                  })}
                </div>
              </div>
            )}
          </div>
        </div>
      )}

      {/* MODAL DIALOG: EDITOR DE METADATOS (TAG EDITOR) */}
      {editingTrack && (
        <dialog className="expanded-dialog tag-editor-dialog" open>
          <div className="dialog-content">
            <div className="dialog-header">
              <div style={{ display: "flex", alignItems: "center", gap: "10px" }}>
                <TagIcon style={{ color: "#79e5a7" }} />
                <h3>Editar Metadatos del Archivo</h3>
              </div>
              <button
                className="dialog-close-btn"
                onClick={() => setEditingTrack(null)}
                type="button"
                aria-label="Cerrar"
              >
                <CloseIcon />
              </button>
            </div>

            <div className="dialog-body" style={{ padding: "20px 28px", display: "flex", flexDirection: "column", gap: "16px" }}>
              <p className="track-fullpath" style={{ margin: 0, fontSize: "0.82rem", wordBreak: "break-all", color: "rgba(244, 247, 245, 0.65)" }}>
                Ruta: <strong style={{ color: "#f4f7f5" }}>{editingTrack.path}</strong>
              </p>

              <label style={{ display: "flex", flexDirection: "column", gap: "6px", fontSize: "0.88rem" }}>
                <span>Título del tema:</span>
                <input
                  className="playlist-input"
                  type="text"
                  value={editingTrack.title}
                  onChange={(e) => setEditingTrack({ ...editingTrack, title: e.target.value })}
                  placeholder="ej. Bohemian Rhapsody"
                />
              </label>

              <label style={{ display: "flex", flexDirection: "column", gap: "6px", fontSize: "0.88rem" }}>
                <span>Artistas (separados por comas):</span>
                <input
                  className="playlist-input"
                  type="text"
                  value={editingTrack.artists}
                  onChange={(e) => setEditingTrack({ ...editingTrack, artists: e.target.value })}
                  placeholder="ej. Queen, Freddie Mercury"
                />
              </label>

              <label style={{ display: "flex", flexDirection: "column", gap: "6px", fontSize: "0.88rem" }}>
                <span>Álbum (opcional):</span>
                <input
                  className="playlist-input"
                  type="text"
                  value={editingTrack.album}
                  onChange={(e) => setEditingTrack({ ...editingTrack, album: e.target.value })}
                  placeholder="ej. A Night at the Opera"
                />
              </label>

              <label style={{ display: "flex", flexDirection: "column", gap: "6px", fontSize: "0.88rem" }}>
                <span>Año de lanzamiento (opcional):</span>
                <input
                  className="playlist-input"
                  type="number"
                  value={editingTrack.year}
                  onChange={(e) => setEditingTrack({ ...editingTrack, year: e.target.value })}
                  placeholder="ej. 1975"
                />
              </label>
            </div>

            <div className="dialog-footer">
              <button
                className="secondary-button"
                onClick={() => setEditingTrack(null)}
                type="button"
              >
                Cancelar
              </button>
              <button
                className="primary-button"
                disabled={isSavingTags || !editingTrack.title.trim()}
                onClick={handleSaveTags}
                type="button"
              >
                {isSavingTags ? "Guardando..." : "Guardar Cambios"}
              </button>
            </div>
          </div>
        </dialog>
      )}
    </section>
  );
}

export default LocalUtilitiesPanel;
