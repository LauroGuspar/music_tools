import { useState, useMemo, useEffect } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { openPath } from "@tauri-apps/plugin-opener";
import { invoke } from "@tauri-apps/api/core";
import { normalizeTitle, normalizeArtist, isMatchNormalized } from "../utils/songMatcher";
import { loadSettings } from "../utils/settings";
import { CopyIcon, TagIcon, PlayIcon, WarningIcon, CloseIcon, PlusIcon, FolderIcon, SearchIcon, TrashIcon } from "./Icons";
import type { StagingBlockData } from "./StagingWorkspace";
import type { ExportTrack } from "../types/export-track";

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
  if (dotIndex === -1) return "UNKNOWN";
  return path.substring(dotIndex + 1).toUpperCase();
}

type LocalUtilitiesPanelProps = {
  settingsVersion?: number;
  onPlayTrack?: (track: { filePath: string; title: string; artists?: string[] }) => void;
  stagingBlocks?: StagingBlockData[];
  onAddTracksToStaging?: (tracks: ExportTrack[], sourceTitle: string) => void;
};

function LocalUtilitiesPanel({
  settingsVersion,
  onPlayTrack,
  stagingBlocks: _stagingBlocks,
  onAddTracksToStaging,
}: LocalUtilitiesPanelProps) {
  const [folderPath, setFolderPath] = useState(() => loadSettings().defaultLocalMusicFolder);

  useEffect(() => {
    setFolderPath(loadSettings().defaultLocalMusicFolder);
  }, [settingsVersion]);

  const [utilityType, setUtilityType] = useState<"duplicates" | "corrupt">("duplicates");
  const [tracks, setTracks] = useState<LocalTrack[]>([]);
  const [corruptFiles, setCorruptFiles] = useState<CorruptFile[]>([]);
  const [isScanning, setIsScanning] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);

  // Tab de filtro para duplicados: "exact" (por tamaño) o "semantic" (por metadatos)
  const [activeSubTab, setActiveSubTab] = useState<"exact" | "semantic">("exact");

  // Buscadores
  const [searchQuery, setSearchQuery] = useState("");
  const [corruptSearchQuery, setCorruptSearchQuery] = useState("");

  // Estado de borrado
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

  // Cerrar modal de edición ID3 con tecla Escape
  useEffect(() => {
    if (!editingTrack) return;
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        setEditingTrack(null);
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [editingTrack]);

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
          return matchesTitle || matchesArtist || matchesPath;
        });

        return { ...g, tracks: filteredTracks };
      })
      .filter((g) => g.tracks.length > 1);
  }, [activeSubTab, exactGroups, semanticGroups, searchQuery]);

  // Filtrado de archivos corruptos
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

  async function handleDeleteFile(path: string) {
    setIsDeleting(true);
    setError(null);
    try {
      await invoke("delete_file", { path });
      setTracks((prev) => prev.filter((t) => t.path !== path));
      setNotice("Archivo eliminado con éxito del disco.");
      setDeletingPath(null);
    } catch (err) {
      console.error(err);
      setError(err instanceof Error ? err.message : "No se pudo eliminar el archivo.");
    } finally {
      setIsDeleting(false);
    }
  }

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
    <div style={{ background: "var(--bg-panel)", border: "1px solid var(--border-color)", borderRadius: "18px", padding: "20px", display: "flex", flexDirection: "column", gap: "16px" }}>
      
      {/* CABECERA Y SELECTOR DE HERRAMIENTA SUB-SEGMENTADA */}
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "flex-start", flexWrap: "wrap", gap: "12px" }}>
        <div>
          <span className="section-label">MANTENIMIENTO DE BIBLIOTECA LOCAL</span>
          <h2 style={{ margin: "2px 0 0 0" }}>Herramientas de Limpieza del Disco</h2>
          <span style={{ fontSize: "0.84rem", color: "var(--text-secondary)", display: "block", marginTop: "4px" }}>
            Escanea tus carpetas en busca de archivos de audio duplicados o corruptos y edita sus etiquetas ID3.
          </span>
        </div>

        <div style={{ display: "flex", gap: "6px", background: "var(--input-bg)", padding: "4px", borderRadius: "14px", border: "1px solid var(--border-color)" }}>
          <button
            type="button"
            className={utilityType === "duplicates" ? "tab-btn active" : "tab-btn"}
            onClick={() => handleUtilityTabChange("duplicates")}
            style={{ fontSize: "0.82rem", padding: "8px 14px", whiteSpace: "nowrap" }}
          >
            <CopyIcon style={{ width: 14, height: 14 }} />
            <span>Detector de Duplicados en Disco</span>
          </button>
          <button
            type="button"
            className={utilityType === "corrupt" ? "tab-btn active" : "tab-btn"}
            onClick={() => handleUtilityTabChange("corrupt")}
            style={{ fontSize: "0.82rem", padding: "8px 14px", whiteSpace: "nowrap" }}
          >
            <WarningIcon style={{ width: 14, height: 14 }} />
            <span>Detector de Archivos Dañados</span>
          </button>
        </div>
      </div>

      {/* SELECTOR DE CARPETA LOCAL Y ACCIONES DE ESCANEO */}
      <div style={{ display: "flex", gap: "10px", flexWrap: "wrap", alignItems: "center", marginTop: "4px" }}>
        <div style={{ flex: 1, minWidth: "280px" }}>
          <input
            type="text"
            className="btn-action"
            style={{ width: "100%", textAlign: "left", background: "var(--input-bg)", border: "1px solid var(--border-color)", borderRadius: "12px", padding: "10px 14px", fontSize: "0.85rem", color: "var(--text-primary)" }}
            value={folderPath}
            readOnly
            placeholder="Selecciona una carpeta de música local..."
          />
        </div>
        <button
          type="button"
          className="secondary-button"
          onClick={selectDirectory}
          style={{ display: "inline-flex", alignItems: "center", gap: "4px" }}
        >
          <FolderIcon style={{ width: 14, height: 14 }} />
          <span>Examinar...</span>
        </button>
        <button
          type="button"
          className="primary-button"
          disabled={!folderPath || isScanning}
          onClick={handleScan}
        >
          {isScanning ? "Escaneando..." : "Escanear Carpeta"}
        </button>
        {tracks.length > 0 && onAddTracksToStaging && (
          <button
            type="button"
            className="secondary-button"
            onClick={handleSendLocalTracksToStaging}
            style={{ display: "inline-flex", alignItems: "center", gap: "4px" }}
          >
            <PlusIcon style={{ width: 14, height: 14 }} />
            <span>Enviar Canciones a Mesa ({tracks.length})</span>
          </button>
        )}
      </div>

      {error && <p className="auth-error workspace-error">{error}</p>}
      {notice && <p className="workspace-notice">{notice}</p>}

      {/* VISTA DE DETECTOR DE DUPLICADOS */}
      {utilityType === "duplicates" && (
        <div style={{ display: "flex", flexDirection: "column", gap: "14px", marginTop: "8px" }}>
          <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", flexWrap: "wrap", gap: "10px" }}>
            <div className="compare-result-tabs" style={{ margin: 0 }}>
              <button
                type="button"
                className={activeSubTab === "exact" ? "compare-tab-pill active" : "compare-tab-pill"}
                onClick={() => { setActiveSubTab("exact"); setSearchQuery(""); }}
              >
                Exacto (Mismo Tamaño • {exactGroups.length} grupos)
              </button>
              <button
                type="button"
                className={activeSubTab === "semantic" ? "compare-tab-pill active" : "compare-tab-pill"}
                onClick={() => { setActiveSubTab("semantic"); setSearchQuery(""); }}
              >
                Semántico (Etiquetas ID3 • {semanticGroups.length} grupos)
              </button>
            </div>

            <div className="search-input-box" style={{ width: "260px" }}>
              <SearchIcon style={{ width: 14, height: 14 }} />
              <input
                type="text"
                placeholder="Filtrar archivos duplicados..."
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
              />
            </div>
          </div>

          <div style={{ display: "flex", flexDirection: "column", gap: "14px" }}>
            {tracks.length === 0 ? (
              <p style={{ padding: "30px", textAlign: "center", color: "var(--text-secondary)" }}>
                Selecciona una carpeta y haz clic en "Escanear Carpeta" para buscar archivos duplicados.
              </p>
            ) : filteredGroups.length === 0 ? (
              <p style={{ padding: "30px", textAlign: "center", color: "var(--text-secondary)" }}>
                No se encontraron grupos de archivos duplicados en esta categoría.
              </p>
            ) : (
              filteredGroups.map((group) => (
                <div className="drawer-block" key={group.key} style={{ borderColor: "var(--border-color)" }}>
                  <div className="drawer-block-header" style={{ background: "var(--bg-surface)", padding: "10px 16px" }}>
                    <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
                      <span className="status-pill" style={{ fontSize: "0.75rem", color: "var(--accent)", border: "1px solid var(--border-active)" }}>
                        {activeSubTab === "exact" ? `Mismo Tamaño: ${group.sizeLabel}` : "Etiquetas ID3"}
                      </span>
                      <strong style={{ fontSize: "0.88rem" }}>{group.title || group.sizeLabel}</strong>
                    </div>
                    <span style={{ fontSize: "0.78rem", color: "var(--text-muted)" }}>{group.tracks.length} archivos idénticos</span>
                  </div>

                  <div style={{ padding: "10px 14px", display: "flex", flexDirection: "column", gap: "10px" }}>
                    {group.tracks.map((track) => (
                      <div key={track.path} style={{ display: "flex", alignItems: "center", justifyContent: "space-between", padding: "8px 12px", background: "var(--bg-panel)", borderRadius: "10px", flexWrap: "wrap", gap: "10px" }}>
                        <div style={{ display: "flex", flexDirection: "column", gap: "2px", flex: 1, minWidth: "240px" }}>
                          <div style={{ display: "flex", alignItems: "center", gap: "6px" }}>
                            <span className="status-pill" style={{ fontSize: "0.7rem", padding: "2px 6px" }}>{getFileExtension(track.path)}</span>
                            <strong style={{ fontSize: "0.88rem" }}>{track.filename}</strong>
                            <span style={{ fontSize: "0.78rem", color: "var(--text-muted)" }}>({formatBytes(track.size)})</span>
                          </div>
                          <span style={{ fontSize: "0.75rem", color: "var(--text-secondary)", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
                            {track.path}
                          </span>
                        </div>

                        <div style={{ display: "flex", gap: "6px", alignItems: "center" }}>
                          {deletingPath !== track.path ? (
                            <>
                              <button
                                type="button"
                                className="btn-action"
                                style={{ padding: "4px 10px", fontSize: "0.78rem", display: "inline-flex", alignItems: "center", gap: "4px" }}
                                onClick={() => handlePlayTrack(track.path, track.title, track.artists)}
                              >
                                <PlayIcon style={{ width: 12, height: 12 }} /> Reproducir
                              </button>
                              <button
                                type="button"
                                className="btn-action"
                                style={{ padding: "4px 10px", fontSize: "0.78rem", display: "inline-flex", alignItems: "center", gap: "4px" }}
                                onClick={() => handleOpenEditTags(track)}
                              >
                                <TagIcon style={{ width: 12, height: 12 }} /> Editar Tags
                              </button>
                              <button
                                type="button"
                                className="btn-action btn-danger"
                                style={{ padding: "4px 10px", fontSize: "0.78rem", display: "inline-flex", alignItems: "center", gap: "4px" }}
                                onClick={() => setDeletingPath(track.path)}
                              >
                                <TrashIcon style={{ width: 12, height: 12 }} /> Borrar
                              </button>
                            </>
                          ) : (
                            <div className="confirm-delete-box" style={{ display: "flex", gap: "6px", alignItems: "center" }}>
                              <span style={{ fontSize: "0.78rem", color: "var(--danger)" }}>¿Eliminar del disco?</span>
                              <button
                                type="button"
                                className="btn-action btn-danger"
                                disabled={isDeleting}
                                onClick={() => handleDeleteFile(track.path)}
                                style={{ padding: "4px 8px", fontSize: "0.75rem" }}
                              >
                                Sí
                              </button>
                              <button
                                type="button"
                                className="btn-action"
                                disabled={isDeleting}
                                onClick={() => setDeletingPath(null)}
                                style={{ padding: "4px 8px", fontSize: "0.75rem" }}
                              >
                                No
                              </button>
                            </div>
                          )}
                        </div>
                      </div>
                    ))}
                  </div>
                </div>
              ))
            )}
          </div>
        </div>
      )}

      {/* VISTA DE ARCHIVOS DAÑADOS / CORRUPTOS */}
      {utilityType === "corrupt" && (
        <div style={{ display: "flex", flexDirection: "column", gap: "14px", marginTop: "8px" }}>
          {corruptFiles.length === 0 ? (
            <p style={{ padding: "30px", textAlign: "center", color: "var(--text-secondary)" }}>
              Selecciona una carpeta y haz clic en "Escanear Carpeta" para analizar archivos dañados o corruptos.
            </p>
          ) : (
            <div className="drawer-block" style={{ borderColor: "rgba(235,87,87,0.4)" }}>
              <div className="drawer-block-header" style={{ background: "rgba(235,87,87,0.08)", padding: "10px 16px" }}>
                <span className="status-pill" style={{ borderColor: "#e57373", color: "#e57373" }}>Archivos con errores de decodificación ID3 / 0 Bytes</span>
              </div>
              <div style={{ padding: "12px", display: "flex", flexDirection: "column", gap: "8px" }}>
                {filteredCorruptFiles.map((file) => (
                  <div key={file.path} style={{ display: "flex", justifyContent: "space-between", alignItems: "center", padding: "8px 12px", background: "var(--bg-panel)", borderRadius: "10px", flexWrap: "wrap", gap: "10px" }}>
                    <div>
                      <strong style={{ fontSize: "0.88rem" }}>{file.filename}</strong>
                      <span style={{ fontSize: "0.75rem", color: "#e57373", display: "block" }}>Razón: {file.errorReason}</span>
                    </div>
                    <div style={{ display: "flex", gap: "6px", alignItems: "center" }}>
                      {deletingPath !== file.path ? (
                        <button
                          type="button"
                          className="btn-action btn-danger"
                          style={{ padding: "4px 10px", fontSize: "0.78rem" }}
                          onClick={() => setDeletingPath(file.path)}
                        >
                          Borrar del Disco
                        </button>
                      ) : (
                        <div style={{ display: "flex", gap: "6px", alignItems: "center" }}>
                          <span style={{ fontSize: "0.78rem", color: "var(--danger)" }}>¿Eliminar?</span>
                          <button
                            type="button"
                            className="btn-action btn-danger"
                            disabled={isDeleting}
                            onClick={() => handleDeleteCorrupt(file.path)}
                            style={{ padding: "4px 8px", fontSize: "0.75rem" }}
                          >
                            Sí
                          </button>
                          <button
                            type="button"
                            className="btn-action"
                            disabled={isDeleting}
                            onClick={() => setDeletingPath(null)}
                            style={{ padding: "4px 8px", fontSize: "0.75rem" }}
                          >
                            No
                          </button>
                        </div>
                      )}
                    </div>
                  </div>
                ))}
              </div>
            </div>
          )}
        </div>
      )}

      {/* MODAL DIALOG: EDITOR DE METADATOS ID3 */}
      {editingTrack && (
        <div className="modal-backdrop-overlay" onClick={() => setEditingTrack(null)}>
          <div className="modal-confirm-card" style={{ width: "min(90vw, 540px)", textAlign: "left" }} onClick={(e) => e.stopPropagation()}>
            <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: "16px" }}>
              <div style={{ display: "flex", alignItems: "center", gap: "10px" }}>
                <TagIcon style={{ width: 18, height: 18, color: "var(--accent)" }} />
                <h3 style={{ margin: 0, fontSize: "1.1rem" }}>Editar Metadatos ID3 del Archivo</h3>
              </div>
              <button type="button" className="icon-close-btn" onClick={() => setEditingTrack(null)}>
                <CloseIcon style={{ width: 18, height: 18 }} />
              </button>
            </div>

            <div style={{ display: "flex", flexDirection: "column", gap: "14px" }}>
              <div>
                <label style={{ fontSize: "0.8rem", color: "var(--text-secondary)", display: "block", marginBottom: "4px" }}>Título de la canción</label>
                <input
                  type="text"
                  className="btn-action"
                  style={{ width: "100%", textAlign: "left", background: "var(--input-bg)" }}
                  value={editingTrack.title}
                  onChange={(e) => setEditingTrack({ ...editingTrack, title: e.target.value })}
                />
              </div>
              <div>
                <label style={{ fontSize: "0.8rem", color: "var(--text-secondary)", display: "block", marginBottom: "4px" }}>Artista(s) (separados por coma)</label>
                <input
                  type="text"
                  className="btn-action"
                  style={{ width: "100%", textAlign: "left", background: "var(--input-bg)" }}
                  value={editingTrack.artists}
                  onChange={(e) => setEditingTrack({ ...editingTrack, artists: e.target.value })}
                />
              </div>
              <div>
                <label style={{ fontSize: "0.8rem", color: "var(--text-secondary)", display: "block", marginBottom: "4px" }}>Álbum</label>
                <input
                  type="text"
                  className="btn-action"
                  style={{ width: "100%", textAlign: "left", background: "var(--input-bg)" }}
                  value={editingTrack.album}
                  onChange={(e) => setEditingTrack({ ...editingTrack, album: e.target.value })}
                />
              </div>
              <div>
                <label style={{ fontSize: "0.8rem", color: "var(--text-secondary)", display: "block", marginBottom: "4px" }}>Año de lanzamiento</label>
                <input
                  type="text"
                  className="btn-action"
                  style={{ width: "100%", textAlign: "left", background: "var(--input-bg)" }}
                  value={editingTrack.year}
                  onChange={(e) => setEditingTrack({ ...editingTrack, year: e.target.value })}
                />
              </div>

              <div style={{ display: "flex", justifyContent: "flex-end", gap: "10px", marginTop: "8px" }}>
                <button type="button" className="btn-action" onClick={() => setEditingTrack(null)}>Cancelar</button>
                <button
                  type="button"
                  className="btn-primary"
                  disabled={isSavingTags || !editingTrack.title.trim()}
                  onClick={handleSaveTags}
                >
                  {isSavingTags ? "Guardando..." : "Guardar Cambios ID3"}
                </button>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

export default LocalUtilitiesPanel;
