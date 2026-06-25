import type { ExportTrack } from "../types/export-track";

type TrackPreviewTableProps = {
  playlistName: string;
  tracks: ExportTrack[];
};

function TrackPreviewTable(props: TrackPreviewTableProps) {
  return (
    <section className="workspace-panel" aria-label="Vista previa de canciones">
      <div className="preview-header">
        <div>
          <p className="section-label">Vista previa</p>
          <h2>{props.playlistName}</h2>
        </div>
        <p className="track-count">{props.tracks.length} canciones</p>
      </div>

      <div className="preview-table-shell">
        <table className="preview-table">
          <thead>
            <tr>
              <th>#</th>
              <th>Cancion</th>
              <th>Artistas</th>
            </tr>
          </thead>
          <tbody>
            {props.tracks.map((track, index) => (
              <tr key={`${track.title}-${index}`}>
                <td>{index + 1}</td>
                <td>{track.title}</td>
                <td>{track.artists.join(", ")}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </section>
  );
}

export default TrackPreviewTable;
