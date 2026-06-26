import { useState, useRef, useEffect } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import type { ExportTrack } from "../types/export-track";
import { parsePlaylistFile } from "../features/playlist/playlistFileLoader";
import { compareTracksAsync, type ComparisonResult } from "../features/compare/compareEngine";

type ComparisonPanelProps = {
  currentPlaylist: { name: string; tracks: ExportTrack[] } | null;
};

type ComparisonMode = "folder-playlist" | "playlist-playlist" | "folder-folder";
type PlaylistSource = "current" | "file";

type LocalTrack = {
  path: string;
  title?: string;
  artists: string[];
  filename: string;
};

function ComparisonPanel({ currentPlaylist }: ComparisonPanelProps) {
  const [mode, setMode] = useState<ComparisonMode>("folder-playlist");

  // Inputs for Folder vs Playlist
  const [folderPath, setFolderPath] = useState("");
  const [playlistSource, setPlaylistSource] = useState<PlaylistSource>("current");
  const [playlistFilePath, setPlaylistFilePath] = useState("");

  // Inputs for Playlist vs Playlist
  const [playlistFilePath1, setPlaylistFilePath1] = useState("");
  const [playlistFilePath2, setPlaylistFilePath2] = useState("");

  // Inputs for Folder vs Folder
  const [folderPath1, setFolderPath1] = useState("");
  const [folderPath2, setFolderPath2] = useState("");

  // Running states
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [results, setResults] = useState<ComparisonResult | null>(null);
  const [resultsTab, setResultsTab] = useState<"missing" | "probable" | "exact" | "extra">("missing");

  // Modal dialog states & refs
  const [isExpanded, setIsExpanded] = useState(false);
  const [searchQuery, setSearchQuery] = useState("");
  const [searchField, setSearchField] = useState<"all" | "title" | "artist">("all");
  const dialogRef = useRef<HTMLDialogElement>(null);

  useEffect(() => {
    const dialog = dialogRef.current;
    if (!dialog) return;

    if (isExpanded) {
      dialog.showModal();
    } else {
      dialog.close();
    }
  }, [isExpanded]);

  const handleClose = () => {
    setIsExpanded(false);
    setSearchQuery("");
    setSearchField("all");
  };

  const filteredMissing = (results?.missing || []).filter((t) => {
    const q = searchQuery.toLowerCase().trim();
    if (!q) return true;
    const matchesTitle = t.title.toLowerCase().includes(q);
    const matchesArtist = t.artists.some((a) => a.toLowerCase().includes(q));

    if (searchField === "title") return matchesTitle;
    if (searchField === "artist") return matchesArtist;
    return matchesTitle || matchesArtist;
  });

  const filteredProbable = (results?.probableMatches || []).filter((m) => {
    const q = searchQuery.toLowerCase().trim();
    if (!q) return true;
    const matchesTitle = m.source.title.toLowerCase().includes(q) || m.target.title.toLowerCase().includes(q);
    const matchesArtist = m.source.artists.some((a) => a.toLowerCase().includes(q)) || m.target.artists.some((a) => a.toLowerCase().includes(q));

    if (searchField === "title") return matchesTitle;
    if (searchField === "artist") return matchesArtist;
    return matchesTitle || matchesArtist;
  });

  const filteredExact = (results?.exactMatches || []).filter((m) => {
    const q = searchQuery.toLowerCase().trim();
    if (!q) return true;
    const matchesTitle = m.source.title.toLowerCase().includes(q);
    const matchesArtist = m.source.artists.some((a) => a.toLowerCase().includes(q));

    if (searchField === "title") return matchesTitle;
    if (searchField === "artist") return matchesArtist;
    return matchesTitle || matchesArtist;
  });

  const filteredExtra = (results?.extra || []).filter((t) => {
    const q = searchQuery.toLowerCase().trim();
    if (!q) return true;
    const matchesTitle = t.title.toLowerCase().includes(q);
    const matchesArtist = t.artists.some((a) => a.toLowerCase().includes(q));

    if (searchField === "title") return matchesTitle;
    if (searchField === "artist") return matchesArtist;
    return matchesTitle || matchesArtist;
  });

  // Directory picking helper
  async function selectDirectory(setter: (path: string) => void) {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: "Selecciona una carpeta de música",
      });
      if (typeof selected === "string") {
        setter(selected);
        setError(null);
      }
    } catch (err) {
      console.error(err);
    }
  }

  // File picking helper
  async function selectPlaylistFile(setter: (path: string) => void) {
    try {
      const selected = await open({
        directory: false,
        multiple: false,
        filters: [{ name: "Playlists", extensions: ["json", "txt"] }],
        title: "Selecciona una playlist exportada",
      });
      if (typeof selected === "string") {
        setter(selected);
        setError(null);
      }
    } catch (err) {
      console.error(err);
    }
  }

  // Get filename from absolute path
  function getFilename(path: string): string {
    const parts = path.split(/[/\\]/);
    return parts[parts.length - 1] || path;
  }

  // Primary runner
  async function handleCompare() {
    setIsLoading(true);
    setError(null);
    setNotice(null);
    setResults(null);

    try {
      if (mode === "folder-playlist") {
        if (!folderPath) throw new Error("Debes seleccionar una carpeta local.");
        
        let targetTracks: ExportTrack[] = [];
        if (playlistSource === "current") {
          if (!currentPlaylist) {
            throw new Error("No hay ninguna playlist activa cargada. Ve a la pestaña 'Exportar playlist' primero o selecciona un archivo.");
          }
          targetTracks = currentPlaylist.tracks;
        } else {
          if (!playlistFilePath) throw new Error("Debes seleccionar un archivo de playlist.");
          const fileContent = await invoke<string>("read_playlist_file", { path: playlistFilePath });
          targetTracks = parsePlaylistFile(fileContent, playlistFilePath);
        }

        setNotice("Escaneando archivos locales y extrayendo metadatos...");
        const localFiles = await invoke<LocalTrack[]>("scan_local_folder", { path: folderPath });
        
        // Map local files to ExportTracks shape
        const sourceTracks: ExportTrack[] = localFiles.map((f) => ({
          title: f.title || f.filename,
          artists: f.artists,
        }));

        setNotice("Ejecutando motor de comparación...");
        // A is Playlist (source of truth), B is Folder (local files)
        // We want to find: A (Playlist) missing from B (Folder)
        const compareResult = await compareTracksAsync(targetTracks, sourceTracks, (processed, total) => {
          setNotice(`Ejecutando motor de comparación (${processed} de ${total})...`);
        });
        setResults(compareResult);
        setNotice("Comparación finalizada.");
        setResultsTab("missing");
      } 
      else if (mode === "playlist-playlist") {
        if (!playlistFilePath1 || !playlistFilePath2) {
          throw new Error("Debes seleccionar ambos archivos de playlist.");
        }

        const fileContent1 = await invoke<string>("read_playlist_file", { path: playlistFilePath1 });
        const fileContent2 = await invoke<string>("read_playlist_file", { path: playlistFilePath2 });

        const tracks1 = parsePlaylistFile(fileContent1, playlistFilePath1);
        const tracks2 = parsePlaylistFile(fileContent2, playlistFilePath2);

        setNotice("Comparando playlists...");
        const compareResult = await compareTracksAsync(tracks1, tracks2, (processed, total) => {
          setNotice(`Comparando playlists (${processed} de ${total})...`);
        });
        setResults(compareResult);
        setNotice("Comparación finalizada.");
        setResultsTab("missing");
      } 
      else if (mode === "folder-folder") {
        if (!folderPath1 || !folderPath2) {
          throw new Error("Debes seleccionar ambas carpetas locales.");
        }

        setNotice("Escaneando carpeta 1...");
        const files1 = await invoke<LocalTrack[]>("scan_local_folder", { path: folderPath1 });
        setNotice("Escaneando carpeta 2...");
        const files2 = await invoke<LocalTrack[]>("scan_local_folder", { path: folderPath2 });

        const tracks1: ExportTrack[] = files1.map((f) => ({
          title: f.title || f.filename,
          artists: f.artists,
        }));

        const tracks2: ExportTrack[] = files2.map((f) => ({
          title: f.title || f.filename,
          artists: f.artists,
        }));

        setNotice("Comparando carpetas...");
        const compareResult = await compareTracksAsync(tracks1, tracks2, (processed, total) => {
          setNotice(`Comparando carpetas (${processed} de ${total})...`);
        });
        setResults(compareResult);
        setNotice("Comparación finalizada.");
        setResultsTab("missing");
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : "Error al realizar la comparación.");
    } finally {
      setIsLoading(false);
    }
  }

  return (
    <div className="comparison-panel">
      <div className="comparison-header">
        <p className="section-label">Motor de comparación</p>
        <h2>Elige lo que deseas comparar</h2>
      </div>

      {/* Comparison Mode Selector */}
      <div className="comparison-mode-selector">
        <div className="segmented-control" role="tablist" aria-label="Modo de comparación">
          {(
            [
              { id: "folder-playlist", label: "Carpeta vs Playlist" },
              { id: "playlist-playlist", label: "Playlist vs Playlist" },
              { id: "folder-folder", label: "Carpeta vs Carpeta" },
            ] as const
          ).map((item) => (
            <button
              aria-selected={mode === item.id}
              className={mode === item.id ? "segment-button is-active" : "segment-button"}
              disabled={isLoading}
              key={item.id}
              onClick={() => {
                setMode(item.id);
                setError(null);
                setNotice(null);
              }}
              role="tab"
              type="button"
            >
              {item.label}
            </button>
          ))}
        </div>
      </div>

      <div className="workspace-panel">
        {/* Mode 1: Folder vs Playlist */}
        {mode === "folder-playlist" && (
          <div className="inputs-grid">
            <div className="input-group">
              <label>1. Carpeta de música local</label>
              <div className="picker-row">
                <input
                  type="text"
                  readOnly
                  placeholder="Ej: /home/usuario/Musica"
                  value={folderPath}
                  className="playlist-input"
                />
                <button
                  type="button"
                  className="secondary-button"
                  onClick={() => selectDirectory(setFolderPath)}
                  disabled={isLoading}
                >
                  Buscar...
                </button>
              </div>
            </div>

            <div className="input-group" style={{ marginTop: "16px" }}>
              <label>2. Fuente de la Playlist</label>
              <div className="playlist-source-selector" style={{ marginTop: "8px" }}>
                <label className="radio-label">
                  <input
                    type="radio"
                    name="playlistSource"
                    checked={playlistSource === "current"}
                    onChange={() => setPlaylistSource("current")}
                    disabled={isLoading}
                  />
                  <span>Usar playlist cargada en pantalla ({currentPlaylist?.name || "ninguna"})</span>
                </label>
                <label className="radio-label" style={{ marginTop: "8px" }}>
                  <input
                    type="radio"
                    name="playlistSource"
                    checked={playlistSource === "file"}
                    onChange={() => setPlaylistSource("file")}
                    disabled={isLoading}
                  />
                  <span>Cargar archivo exportado (.json, .txt)</span>
                </label>
              </div>

              {playlistSource === "file" && (
                <div className="picker-row" style={{ marginTop: "12px" }}>
                  <input
                    type="text"
                    readOnly
                    placeholder="Ningún archivo seleccionado"
                    value={playlistFilePath ? getFilename(playlistFilePath) : ""}
                    className="playlist-input"
                  />
                  <button
                    type="button"
                    className="secondary-button"
                    onClick={() => selectPlaylistFile(setPlaylistFilePath)}
                    disabled={isLoading}
                  >
                    Seleccionar...
                  </button>
                </div>
              )}
            </div>
          </div>
        )}

        {/* Mode 2: Playlist vs Playlist */}
        {mode === "playlist-playlist" && (
          <div className="inputs-grid">
            <div className="input-group">
              <label>1. Primera Playlist (.json, .txt)</label>
              <div className="picker-row">
                <input
                  type="text"
                  readOnly
                  placeholder="Ningún archivo seleccionado"
                  value={playlistFilePath1 ? getFilename(playlistFilePath1) : ""}
                  className="playlist-input"
                />
                <button
                  type="button"
                  className="secondary-button"
                  onClick={() => selectPlaylistFile(setPlaylistFilePath1)}
                  disabled={isLoading}
                >
                  Seleccionar...
                </button>
              </div>
            </div>

            <div className="input-group" style={{ marginTop: "16px" }}>
              <label>2. Segunda Playlist (.json, .txt)</label>
              <div className="picker-row">
                <input
                  type="text"
                  readOnly
                  placeholder="Ningún archivo seleccionado"
                  value={playlistFilePath2 ? getFilename(playlistFilePath2) : ""}
                  className="playlist-input"
                />
                <button
                  type="button"
                  className="secondary-button"
                  onClick={() => selectPlaylistFile(setPlaylistFilePath2)}
                  disabled={isLoading}
                >
                  Seleccionar...
                </button>
              </div>
            </div>
          </div>
        )}

        {/* Mode 3: Folder vs Folder */}
        {mode === "folder-folder" && (
          <div className="inputs-grid">
            <div className="input-group">
              <label>1. Carpeta de origen</label>
              <div className="picker-row">
                <input
                  type="text"
                  readOnly
                  placeholder="Selecciona carpeta origen"
                  value={folderPath1}
                  className="playlist-input"
                />
                <button
                  type="button"
                  className="secondary-button"
                  onClick={() => selectDirectory(setFolderPath1)}
                  disabled={isLoading}
                >
                  Buscar...
                </button>
              </div>
            </div>

            <div className="input-group" style={{ marginTop: "16px" }}>
              <label>2. Carpeta de destino / a comparar</label>
              <div className="picker-row">
                <input
                  type="text"
                  readOnly
                  placeholder="Selecciona carpeta destino"
                  value={folderPath2}
                  className="playlist-input"
                />
                <button
                  type="button"
                  className="secondary-button"
                  onClick={() => selectDirectory(setFolderPath2)}
                  disabled={isLoading}
                >
                  Buscar...
                </button>
              </div>
            </div>
          </div>
        )}

        {/* Action Button */}
        <div style={{ marginTop: "24px", display: "flex", justifyContent: "flex-end" }}>
          <button
            type="button"
            className="primary-button"
            onClick={handleCompare}
            disabled={
              isLoading ||
              (mode === "folder-playlist" &&
                (!folderPath || (playlistSource === "file" && !playlistFilePath))) ||
              (mode === "playlist-playlist" && (!playlistFilePath1 || !playlistFilePath2)) ||
              (mode === "folder-folder" && (!folderPath1 || !folderPath2))
            }
          >
            {isLoading ? "Comparando..." : "Iniciar Comparación"}
          </button>
        </div>

        {/* Messaging */}
        {error && <p className="auth-error workspace-error">{error}</p>}
        {notice && <p className="workspace-notice">{notice}</p>}
      </div>

      {/* Results Display */}
      {results && (
        <div className="results-container" style={{ marginTop: "28px" }}>
          <div className="results-header">
            <h3>Resultado de la Comparación</h3>
            <div className="preview-actions-row">
              <p className="track-count">
                Coincidencias: {results.exactMatches.length} exactas | {results.probableMatches.length} probables
              </p>
              <button
                type="button"
                className="secondary-button detail-btn"
                onClick={() => setIsExpanded(true)}
              >
                Ver Detalle
              </button>
            </div>
          </div>

          {/* Results Tabs */}
          <div className="subnav-tabs" style={{ marginTop: "16px" }}>
            {(
              [
                { id: "missing", label: `Faltantes (${results.missing.length})` },
                { id: "probable", label: `Probables (${results.probableMatches.length})` },
                { id: "exact", label: `Exactas (${results.exactMatches.length})` },
                { id: "extra", label: `Sólo en destino (${results.extra.length})` },
              ] as const
            ).map((tab) => (
              <button
                key={tab.id}
                type="button"
                className={resultsTab === tab.id ? "subnav-tab is-active" : "subnav-tab"}
                onClick={() => setResultsTab(tab.id)}
              >
                {tab.label}
              </button>
            ))}
          </div>

          {/* Results Lists */}
          <div className="preview-table-shell" style={{ marginTop: "16px" }}>
            {resultsTab === "missing" && (
              results.missing.length > 0 ? (
                <table className="preview-table">
                  <thead>
                    <tr>
                      <th>Título</th>
                      <th>Artistas</th>
                    </tr>
                  </thead>
                  <tbody>
                    {results.missing.slice(0, 100).map((track, idx) => (
                      <tr key={idx}>
                        <td>{track.title}</td>
                        <td>{track.artists.join(", ")}</td>
                      </tr>
                    ))}
                    {results.missing.length > 100 && (
                      <tr>
                        <td colSpan={2} className="more-tracks-notice">
                          Mostrando las primeras 100 canciones. Haz clic en "Ver Detalle" para ver y buscar en la lista completa.
                        </td>
                      </tr>
                    )}
                  </tbody>
                </table>
              ) : (
                <p style={{ padding: "20px", textAlign: "center" }}>No faltan canciones. ¡Todo coincide!</p>
              )
            )}

            {resultsTab === "probable" && (
              results.probableMatches.length > 0 ? (
                <table className="preview-table">
                  <thead>
                    <tr>
                      <th>Original (A)</th>
                      <th>Encontrado (B)</th>
                      <th>Similitud</th>
                    </tr>
                  </thead>
                  <tbody>
                    {results.probableMatches.slice(0, 100).map((match, idx) => (
                      <tr key={idx}>
                        <td>
                          <strong>{match.source.title}</strong>
                          <div style={{ fontSize: "0.85em", opacity: 0.7 }}>
                            {match.source.artists.join(", ")}
                          </div>
                        </td>
                        <td>
                          <strong>{match.target.title}</strong>
                          <div style={{ fontSize: "0.85em", opacity: 0.7 }}>
                            {match.target.artists.join(", ")}
                          </div>
                        </td>
                        <td>{Math.round(match.score * 100)}%</td>
                      </tr>
                    ))}
                    {results.probableMatches.length > 100 && (
                      <tr>
                        <td colSpan={3} className="more-tracks-notice">
                          Mostrando las primeras 100 canciones. Haz clic en "Ver Detalle" para ver y buscar en la lista completa.
                        </td>
                      </tr>
                    )}
                  </tbody>
                </table>
              ) : (
                <p style={{ padding: "20px", textAlign: "center" }}>No hay coincidencias parciales o dudosas.</p>
              )
            )}

            {resultsTab === "exact" && (
              results.exactMatches.length > 0 ? (
                <table className="preview-table">
                  <thead>
                    <tr>
                      <th>Título</th>
                      <th>Artistas</th>
                    </tr>
                  </thead>
                  <tbody>
                    {results.exactMatches.slice(0, 100).map((match, idx) => (
                      <tr key={idx}>
                        <td>{match.source.title}</td>
                        <td>{match.source.artists.join(", ")}</td>
                      </tr>
                    ))}
                    {results.exactMatches.length > 100 && (
                      <tr>
                        <td colSpan={2} className="more-tracks-notice">
                          Mostrando las primeras 100 canciones. Haz clic en "Ver Detalle" para ver y buscar en la lista completa.
                        </td>
                      </tr>
                    )}
                  </tbody>
                </table>
              ) : (
                <p style={{ padding: "20px", textAlign: "center" }}>No hay coincidencias exactas.</p>
              )
            )}

            {resultsTab === "extra" && (
              results.extra.length > 0 ? (
                <table className="preview-table">
                  <thead>
                    <tr>
                      <th>Título / Archivo</th>
                      <th>Artistas</th>
                    </tr>
                  </thead>
                  <tbody>
                    {results.extra.slice(0, 100).map((track, idx) => (
                      <tr key={idx}>
                        <td>{track.title}</td>
                        <td>{track.artists.join(", ")}</td>
                      </tr>
                    ))}
                    {results.extra.length > 100 && (
                      <tr>
                        <td colSpan={2} className="more-tracks-notice">
                          Mostrando las primeras 100 canciones. Haz clic en "Ver Detalle" para ver y buscar en la lista completa.
                        </td>
                      </tr>
                    )}
                  </tbody>
                </table>
              ) : (
                <p style={{ padding: "20px", textAlign: "center" }}>No hay archivos extra en el destino.</p>
              )
            )}
          </div>
        </div>
      )}

      {/* Expanded Modal Dialog */}
      {results && (
        <dialog
          ref={dialogRef}
          className="expanded-dialog"
          onClose={handleClose}
          onClick={(e) => {
            if (e.target === dialogRef.current) {
              setIsExpanded(false);
            }
          }}
        >
          <div className="dialog-content">
            <div className="dialog-header">
              <div>
                <span className="section-label">Detalle de Comparación</span>
                <h3>Resultados detallados</h3>
              </div>
              <button
                type="button"
                className="dialog-close-btn"
                onClick={() => setIsExpanded(false)}
                aria-label="Cerrar"
              >
                ✕
              </button>
            </div>

            <div className="dialog-search-row">
              <div className="dialog-search-inputs">
                <input
                  type="search"
                  placeholder="Buscar..."
                  value={searchQuery}
                  onChange={(e) => setSearchQuery(e.target.value)}
                  className="playlist-input dialog-search-input"
                  autoFocus
                />
                <select
                  value={searchField}
                  onChange={(e) => setSearchField(e.target.value as any)}
                  className="playlist-input dialog-filter-select"
                >
                  <option value="all">Canción o Artista</option>
                  <option value="title">Sólo Canción</option>
                  <option value="artist">Sólo Artista</option>
                </select>
              </div>
              <span className="dialog-count">
                Mostrando {
                  resultsTab === "missing" ? filteredMissing.length :
                  resultsTab === "probable" ? filteredProbable.length :
                  resultsTab === "exact" ? filteredExact.length :
                  filteredExtra.length
                } de {
                  resultsTab === "missing" ? results.missing.length :
                  resultsTab === "probable" ? results.probableMatches.length :
                  resultsTab === "exact" ? results.exactMatches.length :
                  results.extra.length
                }
              </span>
            </div>

            <div className="subnav-tabs dialog-tabs" style={{ marginTop: "16px", paddingLeft: "24px" }}>
              {(
                [
                  { id: "missing", label: `Faltantes (${results.missing.length})` },
                  { id: "probable", label: `Probables (${results.probableMatches.length})` },
                  { id: "exact", label: `Exactas (${results.exactMatches.length})` },
                  { id: "extra", label: `Sólo en destino (${results.extra.length})` },
                ] as const
              ).map((tab) => (
                <button
                  key={tab.id}
                  type="button"
                  className={resultsTab === tab.id ? "subnav-tab is-active" : "subnav-tab"}
                  onClick={() => setResultsTab(tab.id)}
                >
                  {tab.label}
                </button>
              ))}
            </div>

            <div className="dialog-table-shell">
              {resultsTab === "missing" && (
                filteredMissing.length > 0 ? (
                  <table className="preview-table dialog-table">
                    <thead>
                      <tr>
                        <th>Título</th>
                        <th>Artistas</th>
                      </tr>
                    </thead>
                    <tbody>
                      {filteredMissing.map((track, idx) => (
                        <tr key={idx}>
                          <td>{track.title}</td>
                          <td>{track.artists.join(", ")}</td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                ) : (
                  <p style={{ padding: "24px", textAlign: "center" }}>No hay canciones en esta pestaña que coincidan.</p>
                )
              )}

              {resultsTab === "probable" && (
                filteredProbable.length > 0 ? (
                  <table className="preview-table dialog-table">
                    <thead>
                      <tr>
                        <th>Original (A)</th>
                        <th>Encontrado (B)</th>
                        <th>Similitud</th>
                      </tr>
                    </thead>
                    <tbody>
                      {filteredProbable.map((match, idx) => (
                        <tr key={idx}>
                          <td>
                            <strong>{match.source.title}</strong>
                            <div style={{ fontSize: "0.85em", opacity: 0.7 }}>
                              {match.source.artists.join(", ")}
                            </div>
                          </td>
                          <td>
                            <strong>{match.target.title}</strong>
                            <div style={{ fontSize: "0.85em", opacity: 0.7 }}>
                              {match.target.artists.join(", ")}
                            </div>
                          </td>
                          <td>{Math.round(match.score * 100)}%</td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                ) : (
                  <p style={{ padding: "24px", textAlign: "center" }}>No hay canciones en esta pestaña que coincidan.</p>
                )
              )}

              {resultsTab === "exact" && (
                filteredExact.length > 0 ? (
                  <table className="preview-table dialog-table">
                    <thead>
                      <tr>
                        <th>Título</th>
                        <th>Artistas</th>
                      </tr>
                    </thead>
                    <tbody>
                      {filteredExact.map((match, idx) => (
                        <tr key={idx}>
                          <td>{match.source.title}</td>
                          <td>{match.source.artists.join(", ")}</td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                ) : (
                  <p style={{ padding: "24px", textAlign: "center" }}>No hay canciones en esta pestaña que coincidan.</p>
                )
              )}

              {resultsTab === "extra" && (
                filteredExtra.length > 0 ? (
                  <table className="preview-table dialog-table">
                    <thead>
                      <tr>
                        <th>Título / Archivo</th>
                        <th>Artistas</th>
                      </tr>
                    </thead>
                    <tbody>
                      {filteredExtra.map((track, idx) => (
                        <tr key={idx}>
                          <td>{track.title}</td>
                          <td>{track.artists.join(", ")}</td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                ) : (
                  <p style={{ padding: "24px", textAlign: "center" }}>No hay canciones en esta pestaña que coincidan.</p>
                )
              )}
            </div>
          </div>
        </dialog>
      )}
    </div>
  );
}

export default ComparisonPanel;
