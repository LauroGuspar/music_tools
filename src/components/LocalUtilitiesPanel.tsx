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

function LocalUtilitiesPanel() {
  const [folderPath, setFolderPath] = useState("");
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
        setTracks(scannedTracks);
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
            Detector de Duplicados
          </button>
          <button
            aria-pressed={utilityType === "corrupt"}
            className={utilityType === "corrupt" ? "subnav-tab is-active" : "subnav-tab"}
            onClick={() => handleUtilityTabChange("corrupt")}
            type="button"
          >
            Archivos Dañados o Corruptos
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
                              📋
                            </button>
                          </div>
                          
                          <div className="track-metadata-preview" style={{ background: "rgba(235, 87, 87, 0.05)", color: "#eb5757" }}>
                            ⚠️ Razón: {file.errorReason}
                          </div>
                        </div>

                        <div className="track-action-col">
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

      {/* Pantalla vacía si no hay archivos escaneados o no coincide con los estados anteriores */}
      {((utilityType === "duplicates" && tracks.length === 0) || (utilityType === "corrupt" && corruptFiles.length === 0)) && !isScanning && (
        <div className="workspace-panel workspace-empty-state" style={{ marginTop: "32px" }}>
          <span className="large-icon">📂</span>
          <h3>Ningún escaneo activo</h3>
          <p className="workspace-copy">
            Selecciona una carpeta local de tu disco y presiona "Escanear" para iniciar el análisis en busca de {utilityType === "duplicates" ? "archivos de audio repetidos" : "archivos dañados o vacíos (0 bytes)"}.
          </p>
        </div>
      )}
    </section>
  );
}

export default LocalUtilitiesPanel;
