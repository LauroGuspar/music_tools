import type { ExportTrack } from "../../types/export-track";

export function exportTxt(tracks: ExportTrack[]): string {
  return tracks.map((track) => `${track.title} - ${track.artists.join(", ")}`).join("\n");
}
