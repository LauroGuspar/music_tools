import { YoutubeTrackInfo } from "./YoutubePlaylistPreview";
import { DownloadIcon, PlayIcon, CloseIcon } from "./Icons";

type MediaPreviewCardProps = {
  results: YoutubeTrackInfo[];
  title?: string;
  onDownloadTrack: (track: YoutubeTrackInfo) => void;
  onPlayPreview?: (track: YoutubeTrackInfo) => void;
  onClose?: () => void;
};

export function MediaPreviewCard({
  results,
  title = "Resultados de búsqueda",
  onDownloadTrack,
  onPlayPreview,
  onClose,
}: MediaPreviewCardProps) {
  if (results.length === 0) return null;

  return (
    <section className="workspace-panel media-preview-panel" style={{ marginTop: "1rem" }}>
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: "1rem" }}>
        <div>
          <p className="section-label">YouTube</p>
          <h2>{title}</h2>
        </div>
        {onClose && (
          <button
            type="button"
            className="icon-button"
            onClick={onClose}
            title="Cerrar resultados"
            style={{ background: "transparent", border: "none", cursor: "pointer", color: "inherit" }}
          >
            <CloseIcon style={{ width: 20, height: 20 }} />
          </button>
        )}
      </div>

      <div
        style={{
          display: "grid",
          gridTemplateColumns: "repeat(auto-fill, minmax(260px, 1fr))",
          gap: "1rem",
        }}
      >
        {results.map((track) => (
          <div
            key={track.id}
            className="media-card"
            style={{
              display: "flex",
              flexDirection: "column",
              borderRadius: "8px",
              border: "1px solid rgba(255, 255, 255, 0.1)",
              backgroundColor: "rgba(255, 255, 255, 0.02)",
              overflow: "hidden",
              padding: "0.75rem",
              gap: "0.5rem",
            }}
          >
            <div style={{ position: "relative", width: "100%", aspectRatio: "16/9", borderRadius: "6px", overflow: "hidden" }}>
              <img
                src={track.thumbnailUrl}
                alt={track.title}
                style={{ width: "100%", height: "100%", objectFit: "cover" }}
              />
              {track.duration && (
                <span
                  style={{
                    position: "absolute",
                    bottom: "6px",
                    right: "6px",
                    backgroundColor: "rgba(0,0,0,0.8)",
                    color: "#fff",
                    fontSize: "0.7rem",
                    padding: "2px 6px",
                    borderRadius: "4px",
                    fontWeight: 600,
                  }}
                >
                  {track.duration}
                </span>
              )}
            </div>

            <div style={{ flex: 1, display: "flex", flexDirection: "column", justifyContent: "space-between" }}>
              <div>
                <h4
                  style={{
                    margin: "0 0 4px 0",
                    fontSize: "0.9rem",
                    lineHeight: "1.2",
                    display: "-webkit-box",
                    WebkitLineClamp: 2,
                    WebkitBoxOrient: "vertical",
                    overflow: "hidden",
                  }}
                  title={track.title}
                >
                  {track.title}
                </h4>
                <p style={{ margin: 0, fontSize: "0.8rem", opacity: 0.7 }}>{track.channel}</p>
              </div>

              <div style={{ display: "flex", gap: "0.5rem", marginTop: "0.75rem" }}>
                {onPlayPreview && (
                  <button
                    type="button"
                    className="secondary-button"
                    style={{ flex: 1, fontSize: "0.8rem", padding: "6px 8px" }}
                    onClick={() => onPlayPreview(track)}
                  >
                    <PlayIcon style={{ width: 14, height: 14, marginRight: 4 }} />
                    Previa
                  </button>
                )}
                <button
                  type="button"
                  className="primary-button"
                  style={{ flex: 1, fontSize: "0.8rem", padding: "6px 8px" }}
                  onClick={() => onDownloadTrack(track)}
                >
                  <DownloadIcon style={{ width: 14, height: 14, marginRight: 4 }} />
                  Descargar
                </button>
              </div>
            </div>
          </div>
        ))}
      </div>
    </section>
  );
}
