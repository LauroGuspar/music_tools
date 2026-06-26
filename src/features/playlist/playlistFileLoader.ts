import type { ExportTrack } from "../../types/export-track";

export function parsePlaylistTxt(content: string): ExportTrack[] {
  const lines = content.split(/\r?\n/);
  const tracks: ExportTrack[] = [];

  for (const line of lines) {
    const trimmed = line.trim();
    if (!trimmed) continue;

    const dashIndex = trimmed.lastIndexOf(" - ");
    if (dashIndex !== -1) {
      const title = trimmed.substring(0, dashIndex).trim();
      const artistsStr = trimmed.substring(dashIndex + 3).trim();
      const artists = artistsStr
        .split(",")
        .map((s) => s.trim())
        .filter(Boolean);

      if (title) {
        tracks.push({ title, artists });
        continue;
      }
    }

    tracks.push({ title: trimmed, artists: [] });
  }

  return tracks;
}

export function parsePlaylistJson(content: string): ExportTrack[] {
  const data = JSON.parse(content);
  if (!Array.isArray(data)) {
    throw new Error("El archivo JSON debe contener un arreglo de canciones.");
  }

  return data.map((item: any, index) => {
    const title = String(item.title || item.name || "").trim();
    const artists = Array.isArray(item.artists)
      ? item.artists.map((a: any) => String(a || "").trim()).filter(Boolean)
      : typeof item.artist === "string" && item.artist.trim()
        ? [item.artist.trim()]
        : [];

    if (!title) {
      throw new Error(`Canción inválida en el índice ${index}: el título está vacío.`);
    }

    return { title, artists };
  });
}

export function parsePlaylistFile(content: string, filename: string): ExportTrack[] {
  const cleanFilename = filename.toLowerCase();
  if (cleanFilename.endsWith(".json")) {
    return parsePlaylistJson(content);
  } else if (cleanFilename.endsWith(".txt")) {
    return parsePlaylistTxt(content);
  } else {
    // Try JSON first, fallback to TXT
    try {
      return parsePlaylistJson(content);
    } catch {
      return parsePlaylistTxt(content);
    }
  }
}
