import type { ExportTrack } from "../../types/export-track";

export function exportJson(tracks: ExportTrack[]): string {
  return JSON.stringify(tracks, null, 2);
}
