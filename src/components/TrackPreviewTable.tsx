import { useState, useRef, useEffect, useMemo } from "react";
import type { ExportTrack } from "../types/export-track";
import { CloseIcon, DownloadIcon } from "./Icons";

type TrackPreviewTableProps = {
  playlistName: string;
  tracks: ExportTrack[];
  onSendToDownloads?: (tracks: ExportTrack[], playlistName: string) => void;
};

function TrackPreviewTable(props: TrackPreviewTableProps) {
  const [isExpanded, setIsExpanded] = useState(false);
  const [searchQuery, setSearchQuery] = useState("");
  const [searchField, setSearchField] = useState<"all" | "title" | "artist">("all");
  const dialogRef = useRef<HTMLDialogElement>(null);

  // Set storing original indices of selected tracks
  const [selectedIndices, setSelectedIndices] = useState<Set<number>>(() => {
    return new Set(props.tracks.map((_, i) => i));
  });

  // Re-initialize selection if tracks array changes
  useEffect(() => {
    setSelectedIndices(new Set(props.tracks.map((_, i) => i)));
  }, [props.tracks]);

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

  const indexedTracks = useMemo(() => {
    return props.tracks.map((track, originalIndex) => ({ track, originalIndex }));
  }, [props.tracks]);

  const filteredTracks = useMemo(() => {
    const query = searchQuery.toLowerCase().trim();
    if (!query) return indexedTracks;

    return indexedTracks.filter(({ track }) => {
      const matchesTitle = track.title.toLowerCase().includes(query);
      const matchesArtist = track.artists.some((artist) => artist.toLowerCase().includes(query));

      if (searchField === "title") return matchesTitle;
      if (searchField === "artist") return matchesArtist;
      return matchesTitle || matchesArtist;
    });
  }, [indexedTracks, searchQuery, searchField]);

  const allFilteredSelected = useMemo(() => {
    if (filteredTracks.length === 0) return false;
    return filteredTracks.every(({ originalIndex }) => selectedIndices.has(originalIndex));
  }, [filteredTracks, selectedIndices]);

  const toggleSelectAllFiltered = () => {
    const next = new Set(selectedIndices);
    if (allFilteredSelected) {
      filteredTracks.forEach(({ originalIndex }) => next.delete(originalIndex));
    } else {
      filteredTracks.forEach(({ originalIndex }) => next.add(originalIndex));
    }
    setSelectedIndices(next);
  };

  const toggleTrack = (originalIndex: number) => {
    const next = new Set(selectedIndices);
    if (next.has(originalIndex)) {
      next.delete(originalIndex);
    } else {
      next.add(originalIndex);
    }
    setSelectedIndices(next);
  };

  const handleSendToDownloadsClick = () => {
    if (!props.onSendToDownloads) return;
    const selectedTracks = props.tracks.filter((_, idx) => selectedIndices.has(idx));
    props.onSendToDownloads(selectedTracks, props.playlistName);
    setIsExpanded(false);
  };

  return (
    <section className="workspace-panel" aria-label="Vista previa de canciones">
      <div className="preview-header">
        <div>
          <p className="section-label">Vista previa de Playlist</p>
          <h2>{props.playlistName}</h2>
          <p className="workspace-copy" style={{ margin: 0 }}>
            {selectedIndices.size} de {props.tracks.length} canciones seleccionadas
          </p>
        </div>
        <div className="preview-actions-row" style={{ display: "flex", gap: "0.5rem", alignItems: "center" }}>
          {props.onSendToDownloads && (
            <button
              type="button"
              className="primary-button"
              disabled={selectedIndices.size === 0}
              onClick={handleSendToDownloadsClick}
              style={{ fontSize: "0.85rem", padding: "6px 12px" }}
            >
              <DownloadIcon style={{ width: 14, height: 14, marginRight: 6 }} />
              Descargar {selectedIndices.size}
            </button>
          )}
          <button
            type="button"
            className="secondary-button detail-btn"
            onClick={() => setIsExpanded(true)}
          >
            Ver Detalle
          </button>
        </div>
      </div>

      <div className="preview-table-shell" style={{ marginTop: "1rem" }}>
        <table className="preview-table">
          <thead>
            <tr>
              <th style={{ width: "40px", textAlign: "center" }}>#</th>
              <th>Canción</th>
              <th>Artistas</th>
            </tr>
          </thead>
          <tbody>
            {props.tracks.slice(0, 100).map((track, index) => {
              const isSelected = selectedIndices.has(index);
              return (
                <tr
                  key={`${track.title}-${index}`}
                  style={{ opacity: isSelected ? 1 : 0.4 }}
                >
                  <td style={{ textAlign: "center" }}>{index + 1}</td>
                  <td>{track.title}</td>
                  <td>{track.artists.join(", ")}</td>
                </tr>
              );
            })}
            {props.tracks.length > 100 && (
              <tr>
                <td colSpan={3} className="more-tracks-notice">
                  Mostrando las primeras 100 canciones. Haz clic en "Ver Detalle" para buscar y gestionar la selección completa.
                </td>
              </tr>
            )}
          </tbody>
        </table>
      </div>

      {/* Expanded Modal Dialog "Ver Detalle" */}
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
              <span className="section-label">Detalle de Playlist</span>
              <h3>{props.playlistName}</h3>
              <p className="workspace-copy" style={{ margin: 0 }}>
                {selectedIndices.size} de {props.tracks.length} canciones seleccionadas
              </p>
            </div>
            <button
              type="button"
              className="dialog-close-btn"
              onClick={() => setIsExpanded(false)}
              aria-label="Cerrar"
            >
              <CloseIcon />
            </button>
          </div>

          <div className="dialog-search-row">
            <div className="dialog-search-inputs">
              <input
                type="search"
                placeholder="Buscar en la playlist..."
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

            <button
              type="button"
              className="secondary-button"
              onClick={toggleSelectAllFiltered}
              style={{ fontSize: "0.85rem" }}
            >
              {allFilteredSelected ? "Desmarcar visibles" : "Marcar visibles"}
            </button>
          </div>

          <div className="dialog-table-shell">
            <table className="preview-table dialog-table">
              <thead>
                <tr>
                  <th style={{ width: "40px", textAlign: "center" }}>
                    <input
                      type="checkbox"
                      checked={allFilteredSelected}
                      onChange={toggleSelectAllFiltered}
                    />
                  </th>
                  <th style={{ width: "40px" }}>#</th>
                  <th>Canción</th>
                  <th>Artistas</th>
                </tr>
              </thead>
              <tbody>
                {filteredTracks.map(({ track, originalIndex }, displayIdx) => {
                  const isChecked = selectedIndices.has(originalIndex);
                  return (
                    <tr
                      key={`${track.title}-${originalIndex}`}
                      onClick={() => toggleTrack(originalIndex)}
                      style={{ cursor: "pointer", backgroundColor: isChecked ? "rgba(255,255,255,0.03)" : undefined }}
                    >
                      <td style={{ textAlign: "center" }} onClick={(e) => e.stopPropagation()}>
                        <input
                          type="checkbox"
                          checked={isChecked}
                          onChange={() => toggleTrack(originalIndex)}
                        />
                      </td>
                      <td>{displayIdx + 1}</td>
                      <td>
                        <strong style={{ opacity: isChecked ? 1 : 0.5 }}>{track.title}</strong>
                      </td>
                      <td style={{ opacity: isChecked ? 1 : 0.5 }}>{track.artists.join(", ")}</td>
                    </tr>
                  );
                })}
                {filteredTracks.length === 0 && (
                  <tr>
                    <td colSpan={4} style={{ textAlign: "center", padding: "24px" }}>
                      No se encontraron canciones que coincidan con la búsqueda.
                    </td>
                  </tr>
                )}
              </tbody>
            </table>
          </div>

          <div style={{ marginTop: "1rem", display: "flex", justifyContent: "flex-end", gap: "0.5rem" }}>
            <button type="button" className="secondary-button" onClick={handleClose}>
              Cerrar
            </button>
            {props.onSendToDownloads && (
              <button
                type="button"
                className="primary-button"
                disabled={selectedIndices.size === 0}
                onClick={handleSendToDownloadsClick}
              >
                <DownloadIcon style={{ width: 16, height: 16, marginRight: 6 }} />
                Enviar {selectedIndices.size} canciones a Descargas
              </button>
            )}
          </div>
        </div>
      </dialog>
    </section>
  );
}

export default TrackPreviewTable;
