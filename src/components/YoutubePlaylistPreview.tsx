import { useState, useMemo } from "react";
import { CheckIcon, CloseIcon } from "./Icons";

export interface YoutubeTrackInfo {
  id: string;
  title: string;
  channel: string;
  duration: string;
  thumbnailUrl: string;
  videoUrl: string;
}

export interface YoutubePlaylistData {
  id: string;
  title: string;
  tracks: YoutubeTrackInfo[];
}

type YoutubePlaylistPreviewProps = {
  playlist: YoutubePlaylistData;
  onAddTracks: (selectedTracks: YoutubeTrackInfo[]) => void;
  onClose: () => void;
};

export function YoutubePlaylistPreview({
  playlist,
  onAddTracks,
  onClose,
}: YoutubePlaylistPreviewProps) {
  const [selectedIds, setSelectedIds] = useState<Set<string>>(() => {
    return new Set(playlist.tracks.map((t) => t.id));
  });
  const [filterQuery, setFilterQuery] = useState("");

  const filteredTracks = useMemo(() => {
    if (!filterQuery.trim()) return playlist.tracks;
    const q = filterQuery.toLowerCase();
    return playlist.tracks.filter(
      (t) => t.title.toLowerCase().includes(q) || t.channel.toLowerCase().includes(q)
    );
  }, [playlist.tracks, filterQuery]);

  const allFilteredSelected = useMemo(() => {
    if (filteredTracks.length === 0) return false;
    return filteredTracks.every((t) => selectedIds.has(t.id));
  }, [filteredTracks, selectedIds]);

  const toggleSelectAll = () => {
    const next = new Set(selectedIds);
    if (allFilteredSelected) {
      filteredTracks.forEach((t) => next.delete(t.id));
    } else {
      filteredTracks.forEach((t) => next.add(t.id));
    }
    setSelectedIds(next);
  };

  const toggleTrack = (id: string) => {
    const next = new Set(selectedIds);
    if (next.has(id)) {
      next.delete(id);
    } else {
      next.add(id);
    }
    setSelectedIds(next);
  };

  const handleConfirm = () => {
    const selected = playlist.tracks.filter((t) => selectedIds.has(t.id));
    onAddTracks(selected);
  };

  return (
    <section className="workspace-panel youtube-playlist-preview" style={{ marginTop: "1rem" }}>
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: "1rem" }}>
        <div>
          <p className="section-label">Playlist de YouTube</p>
          <h2>{playlist.title}</h2>
          <p className="workspace-copy" style={{ margin: 0 }}>
            {selectedIds.size} de {playlist.tracks.length} canciones seleccionadas
          </p>
        </div>
        <button
          type="button"
          className="icon-button"
          onClick={onClose}
          title="Cerrar vista previa"
          style={{ background: "transparent", border: "none", cursor: "pointer", color: "inherit" }}
        >
          <CloseIcon style={{ width: 20, height: 20 }} />
        </button>
      </div>

      <div style={{ display: "flex", gap: "0.5rem", marginBottom: "1rem" }}>
        <input
          type="text"
          className="playlist-input"
          placeholder="Filtrar en esta playlist..."
          value={filterQuery}
          onChange={(e) => setFilterQuery(e.target.value)}
          style={{ flex: 1 }}
        />
        <button type="button" className="secondary-button" onClick={toggleSelectAll}>
          {allFilteredSelected ? "Desmarcar visibles" : "Marcar visibles"}
        </button>
      </div>

      <div className="table-wrapper" style={{ maxHeight: "360px", overflowY: "auto" }}>
        <table className="preview-table">
          <thead>
            <tr>
              <th style={{ width: "40px", textAlign: "center" }}>
                <input
                  type="checkbox"
                  checked={allFilteredSelected}
                  onChange={toggleSelectAll}
                />
              </th>
              <th style={{ width: "60px" }}>Portada</th>
              <th>Título</th>
              <th>Canal / Uploader</th>
              <th style={{ width: "80px", textAlign: "right" }}>Duración</th>
            </tr>
          </thead>
          <tbody>
            {filteredTracks.map((track) => {
              const isChecked = selectedIds.has(track.id);
              return (
                <tr
                  key={track.id}
                  onClick={() => toggleTrack(track.id)}
                  style={{ cursor: "pointer", backgroundColor: isChecked ? "rgba(255,255,255,0.03)" : undefined }}
                >
                  <td style={{ textAlign: "center" }} onClick={(e) => e.stopPropagation()}>
                    <input
                      type="checkbox"
                      checked={isChecked}
                      onChange={() => toggleTrack(track.id)}
                    />
                  </td>
                  <td>
                    {track.thumbnailUrl ? (
                      <img
                        src={track.thumbnailUrl}
                        alt={track.title}
                        style={{ width: 44, height: 32, objectFit: "cover", borderRadius: 4 }}
                      />
                    ) : (
                      <div style={{ width: 44, height: 32, background: "#333", borderRadius: 4 }} />
                    )}
                  </td>
                  <td>
                    <strong>{track.title}</strong>
                  </td>
                  <td>{track.channel}</td>
                  <td style={{ textAlign: "right" }}>{track.duration}</td>
                </tr>
              );
            })}
          </tbody>
        </table>
      </div>

      <div style={{ marginTop: "1rem", display: "flex", justifyContent: "flex-end", gap: "0.5rem" }}>
        <button type="button" className="secondary-button" onClick={onClose}>
          Cancelar
        </button>
        <button
          type="button"
          className="primary-button"
          disabled={selectedIds.size === 0}
          onClick={handleConfirm}
        >
          <CheckIcon style={{ width: 16, height: 16, marginRight: 6 }} />
          Añadir {selectedIds.size} canción{selectedIds.size === 1 ? "" : "es"} a la cola
        </button>
      </div>
    </section>
  );
}
