import { useState, useMemo, useEffect } from "react";
import { CloseIcon, YoutubeIcon, MusicIcon } from "./Icons";

export interface UnifiedTrackInfo {
  id: string;
  title: string;
  artistOrChannel: string;
  thumbnailUrl?: string;
  duration?: string;
  videoUrl?: string;
  albumName?: string;
  releaseDate?: string;
}

export interface UniversalPlaylistData {
  id: string;
  title: string;
  source: "youtube" | "spotify";
  tracks: UnifiedTrackInfo[];
}

type UniversalPlaylistPreviewProps = {
  playlist: UniversalPlaylistData;
  onAddTracksToQueue: (selectedTracks: UnifiedTrackInfo[]) => void;
  onClose?: () => void;
};

export function UniversalPlaylistPreview({
  playlist,
  onAddTracksToQueue,
  onClose,
}: UniversalPlaylistPreviewProps) {
  const [selectedIds, setSelectedIds] = useState<Set<string>>(() => {
    return new Set(playlist.tracks.map((t) => t.id));
  });
  const [filterQuery, setFilterQuery] = useState("");

  // Sync selectedIds and reset search filter when playlist changes
  useEffect(() => {
    setSelectedIds(new Set(playlist.tracks.map((t) => t.id)));
    setFilterQuery("");
  }, [playlist.id, playlist.tracks]);

  const filteredTracks = useMemo(() => {
    if (!filterQuery.trim()) return playlist.tracks;
    const q = filterQuery.toLowerCase();
    return playlist.tracks.filter(
      (t) => t.title.toLowerCase().includes(q) || t.artistOrChannel.toLowerCase().includes(q)
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

  const handleConfirmAddQueue = () => {
    const selected = playlist.tracks.filter((t) => selectedIds.has(t.id));
    onAddTracksToQueue(selected);
  };

  const hasAnyThumbnail = useMemo(() => {
    return playlist.tracks.some((t) => Boolean(t.thumbnailUrl));
  }, [playlist.tracks]);

  const hasAnyDuration = useMemo(() => {
    return playlist.tracks.some((t) => Boolean(t.duration && t.duration !== "-"));
  }, [playlist.tracks]);

  return (
    <section className="workspace-panel universal-playlist-preview" style={{ marginTop: "1rem" }}>
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "flex-start", marginBottom: "1rem" }}>
        <div>
          <p className="section-label" style={{ display: "inline-flex", alignItems: "center", gap: "6px" }}>
            {playlist.source === "youtube" ? (
              <>
                <YoutubeIcon style={{ width: 14, height: 14, color: "#FF0000" }} /> PLAYLIST DE YOUTUBE
              </>
            ) : (
              <>
                <MusicIcon style={{ width: 14, height: 14, color: "#1DB954" }} /> PLAYLIST DE SPOTIFY
              </>
            )}
          </p>
          <h2 style={{ margin: "4px 0" }}>{playlist.title}</h2>
          <p className="workspace-copy" style={{ margin: 0 }}>
            {selectedIds.size} de {playlist.tracks.length} canciones seleccionadas
          </p>
        </div>
        {onClose && (
          <button
            type="button"
            className="icon-button"
            onClick={onClose}
            title="Cerrar vista previa"
            style={{ background: "transparent", border: "none", cursor: "pointer", color: "inherit" }}
          >
            <CloseIcon style={{ width: 20, height: 20 }} />
          </button>
        )}
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

      <div className="table-wrapper" style={{ maxHeight: "420px", overflowY: "auto" }}>
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
              {hasAnyThumbnail && <th style={{ width: "60px" }}>PORTADA</th>}
              <th>TÍTULO</th>
              <th>CANAL / ARTISTA</th>
              {hasAnyDuration && <th style={{ width: "90px", textAlign: "right" }}>DURACIÓN</th>}
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
                  {hasAnyThumbnail && (
                    <td>
                      {track.thumbnailUrl ? (
                        <img
                          src={track.thumbnailUrl}
                          alt={track.title}
                          style={{ width: 44, height: 32, objectFit: "cover", borderRadius: 4 }}
                        />
                      ) : (
                        <div style={{ width: 44, height: 32, background: "rgba(255,255,255,0.05)", borderRadius: 4 }} />
                      )}
                    </td>
                  )}
                  <td>
                    <strong style={{ opacity: isChecked ? 1 : 0.5 }}>{track.title}</strong>
                  </td>
                  <td style={{ opacity: isChecked ? 1 : 0.5 }}>{track.artistOrChannel}</td>
                  {hasAnyDuration && (
                    <td style={{ textAlign: "right", opacity: isChecked ? 1 : 0.5 }}>
                      {track.duration || "-"}
                    </td>
                  )}
                </tr>
              );
            })}
          </tbody>
        </table>
      </div>

      <div style={{ marginTop: "1rem", display: "flex", justifyContent: "flex-end", gap: "0.5rem", flexWrap: "wrap" }}>
        {onClose && (
          <button type="button" className="secondary-button" onClick={onClose}>
            Cancelar
          </button>
        )}
        <button
          type="button"
          className="primary-button"
          disabled={selectedIds.size === 0}
          onClick={handleConfirmAddQueue}
        >
          Agregar {selectedIds.size} canción{selectedIds.size === 1 ? "" : "es"} a la Mesa de Trabajo
        </button>
      </div>
    </section>
  );
}
