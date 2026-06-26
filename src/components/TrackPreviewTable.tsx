import { useState, useRef, useEffect } from "react";
import type { ExportTrack } from "../types/export-track";

type TrackPreviewTableProps = {
  playlistName: string;
  tracks: ExportTrack[];
};

function TrackPreviewTable(props: TrackPreviewTableProps) {
  const [isExpanded, setIsExpanded] = useState(false);
  const [searchQuery, setSearchQuery] = useState("");
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
  };

  const filteredTracks = props.tracks.filter((track) => {
    const query = searchQuery.toLowerCase().trim();
    if (!query) return true;
    return (
      track.title.toLowerCase().includes(query) ||
      track.artists.some((artist) => artist.toLowerCase().includes(query))
    );
  });

  return (
    <section className="workspace-panel" aria-label="Vista previa de canciones">
      <div className="preview-header">
        <div>
          <p className="section-label">Vista previa</p>
          <h2>{props.playlistName}</h2>
        </div>
        <div className="preview-actions-row">
          <p className="track-count">{props.tracks.length} canciones</p>
          <button
            type="button"
            className="secondary-button detail-btn"
            onClick={() => setIsExpanded(true)}
          >
            Ver Detalle
          </button>
        </div>
      </div>

      <div className="preview-table-shell">
        <table className="preview-table">
          <thead>
            <tr>
              <th>#</th>
              <th>Canción</th>
              <th>Artistas</th>
            </tr>
          </thead>
          <tbody>
            {props.tracks.slice(0, 100).map((track, index) => (
              <tr key={`${track.title}-${index}`}>
                <td>{index + 1}</td>
                <td>{track.title}</td>
                <td>{track.artists.join(", ")}</td>
              </tr>
            ))}
            {props.tracks.length > 100 && (
              <tr>
                <td colSpan={3} className="more-tracks-notice">
                  Mostrando las primeras 100 canciones. Haz clic en "Ver Detalle" para buscar y ver la lista completa.
                </td>
              </tr>
            )}
          </tbody>
        </table>
      </div>

      {/* Expanded Modal Dialog */}
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
            <input
              type="search"
              placeholder="Buscar por canción o artista..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className="playlist-input dialog-search-input"
              autoFocus
            />
            <span className="dialog-count">
              {filteredTracks.length} de {props.tracks.length}
            </span>
          </div>

          <div className="dialog-table-shell">
            <table className="preview-table dialog-table">
              <thead>
                <tr>
                  <th>#</th>
                  <th>Canción</th>
                  <th>Artistas</th>
                </tr>
              </thead>
              <tbody>
                {filteredTracks.map((track, index) => (
                  <tr key={`${track.title}-${index}`}>
                    <td>{index + 1}</td>
                    <td>{track.title}</td>
                    <td>{track.artists.join(", ")}</td>
                  </tr>
                ))}
                {filteredTracks.length === 0 && (
                  <tr>
                    <td colSpan={3} style={{ textAlign: "center", padding: "24px" }}>
                      No se encontraron canciones que coincidan con la búsqueda.
                    </td>
                  </tr>
                )}
              </tbody>
            </table>
          </div>
        </div>
      </dialog>
    </section>
  );
}

export default TrackPreviewTable;
