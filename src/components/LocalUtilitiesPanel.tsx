import { useState, useMemo } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import { normalizeTitle, isMatch } from "../utils/songMatcher";

type LocalTrack = {
  path: string;
  title?: string;
  artists: string[];
  filename: string;
  size: number;
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

function LocalUtilitiesPanel() {
  const [folderPath, setFolderPath] = useState("");
  const [tracks, setTracks] = useState<LocalTrack[]>([]);
  const [isScanning, setIsScanning] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);

  // Tab de filtro: "exact" (por tamaño de archivo) o "semantic" (por metadatos de canción)
  const [activeSubTab, setActiveSubTab] = useState<"exact" | "semantic">("exact");

  // Buscador
  const [searchQuery, setSearchQuery] = useState("");
  const [searchScope, setSearchScope] = useState<"all" | "title" | "artist" | "path">("all");

  // Estado de confirmación de borrado
  const [deletingPath, setDeletingPath] = useState<string | null>(null);
  const [isDeleting, setIsDeleting] = useState(false);

  // Selector de carpeta
  async function selectDirectory() {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: "Selecciona una carpeta de música para buscar duplicados",
      });
      if (typeof selected === "string") {
        setFolderPath(selected);
        setError(null);
        setTracks([]);
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
    setNotice("Escaneando archivos de música locales. Esto puede tardar unos segundos...");
    setTracks([]);

    try {
      const scannedTracks = await invoke<LocalTrack[]>("scan_local_folder", { path: folderPath });
      setTracks(scannedTracks);
      setNotice(`Escaneo completado. Se encontraron ${scannedTracks.length} archivos de audio.`);
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
    // Ordenar de mayor a menor tamaño para priorizar espacio liberado
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
      const title = track.title || track.filename;
      const normTitle = normalizeTitle(title);
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

      for (const track of groupTracks) {
        let added = false;
        for (const subGroup of subGroups) {
          const representative = subGroup[0];
          const match = isMatch(
            { title: track.title || track.filename, artists: track.artists },
            { title: representative.title || representative.filename, artists: representative.artists }
          );
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

  // Filtrado según búsqueda
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
      // Mantener el grupo solo si sigue teniendo al menos 2 elementos (es decir, duplicados tras el filtro)
      .filter((g) => g.tracks.length > 1);
  }, [activeSubTab, exactGroups, semanticGroups, searchQuery, searchScope]);

  // Borrado de archivo
  async function handleDeleteFile(path: string) {
    setIsDeleting(true);
    setError(null);
    try {
      await invoke("delete_file", { path });
      // Remover track del estado
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

  return (
    <section className="workspace-panel">
      <div className="panel-header-section">
        <p className="section-label">Utilidades locales</p>
        <h2>Detector de archivos duplicados</h2>
        <p className="workspace-copy">
          Escanea tu biblioteca de música en el disco local para liberar espacio o limpiar duplicados exactos e imprecisiones de etiquetas.
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
        <button className="playlist-btn secondary-btn" onClick={selectDirectory} type="button">
          Buscar Carpeta
        </button>
        <button
          className="playlist-btn primary-btn"
          disabled={!folderPath || isScanning}
          onClick={handleScan}
          type="button"
        >
          {isScanning ? "Escaneando..." : "Escanear"}
        </button>
      </div>

      {error ? <p className="auth-error workspace-error">{error}</p> : null}
      {notice ? <p className="workspace-notice">{notice}</p> : null}

      {/* Si ya hay pistas escaneadas, mostrar interfaz de duplicados */}
      {tracks.length > 0 ? (
        <div className="duplicates-workspace">
          <div className="duplicates-controls-row">
            {/* Pestañas de Sub-filtro */}
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

            {/* Buscador dentro de Duplicados */}
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

          {/* Listado de Grupos de Duplicados */}
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
                                📋
                              </button>
                            </div>
                            {track.title ? (
                              <div className="track-metadata-preview">
                                🏷️ {track.title} {track.artists.length > 0 ? `· ${track.artists.join(", ")}` : ""}
                              </div>
                            ) : null}
                          </div>

                          <div className="track-action-col">
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
      ) : (
        !isScanning && (
          <div className="workspace-panel workspace-empty-state" style={{ marginTop: "32px" }}>
            <span className="large-icon">📂</span>
            <h3>Ningún escaneo activo</h3>
            <p className="workspace-copy">
              Selecciona una carpeta local de tu disco (por ejemplo, tu carpeta de descargas o biblioteca de música) para iniciar el análisis en busca de archivos de audio idénticos o repetidos.
            </p>
          </div>
        )
      )}
    </section>
  );
}

export default LocalUtilitiesPanel;
