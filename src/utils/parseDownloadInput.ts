export type DownloadInputType =
  | { type: "youtube_playlist"; playlistId: string; rawUrl: string }
  | { type: "youtube_video"; videoId: string; rawUrl: string }
  | { type: "spotify_playlist"; playlistId: string; rawUrl: string }
  | { type: "spotify_track"; trackId: string; rawUrl: string }
  | { type: "search_query"; query: string };

/**
 * Normaliza y clasifica automáticamente cualquier entrada introducida por el usuario
 * en el Smart Omnibar de descargas.
 */
export function parseDownloadInput(input: string): DownloadInputType {
  const trimmed = input.trim();
  if (!trimmed) {
    return { type: "search_query", query: "" };
  }

  // 1. URL de Vídeo individual de YouTube (prioridad sobre enlace watch con playlist mix)
  // Ejemplos:
  // - https://www.youtube.com/watch?v=dQw4w9WgXcQ
  // - https://youtu.be/dQw4w9WgXcQ
  // - https://www.youtube.com/shorts/dQw4w9WgXcQ
  // - https://music.youtube.com/watch?v=dQw4w9WgXcQ
  const ytVideoRegex = /(?:youtube\.com\/(?:watch\?.*v=|embed\/|v\/|shorts\/)|youtu\.be\/|music\.youtube\.com\/watch\?.*v=)([a-zA-Z0-9_-]{11})/i;
  
  // 2. URL de Playlist de YouTube (explicitamente playlist?list=)
  const ytExplicitPlaylistRegex = /(?:youtube\.com|music\.youtube\.com)\/playlist\?.*list=([a-zA-Z0-9_-]+)/i;
  const ytExplicitMatch = trimmed.match(ytExplicitPlaylistRegex);
  if (ytExplicitMatch) {
    return {
      type: "youtube_playlist",
      playlistId: ytExplicitMatch[1],
      rawUrl: trimmed,
    };
  }

  // 3. Chequear si es un vídeo individual
  const ytVideoMatch = trimmed.match(ytVideoRegex);
  if (ytVideoMatch) {
    return {
      type: "youtube_video",
      videoId: ytVideoMatch[1],
      rawUrl: trimmed,
    };
  }

  // 4. URL de Playlist de YouTube genérica (fallback list=)
  const ytPlaylistRegex = /(?:youtube\.com|music\.youtube\.com).*[?&]list=([a-zA-Z0-9_-]+)/i;
  const ytPlaylistMatch = trimmed.match(ytPlaylistRegex);
  if (ytPlaylistMatch) {
    return {
      type: "youtube_playlist",
      playlistId: ytPlaylistMatch[1],
      rawUrl: trimmed,
    };
  }

  // 3. URL de Playlist de Spotify
  const spotifyPlaylistRegex = /(?:open\.spotify\.com\/playlist\/|spotify:playlist:)([a-zA-Z0-9]{22})/i;
  const spotifyPlaylistMatch = trimmed.match(spotifyPlaylistRegex);
  if (spotifyPlaylistMatch) {
    return {
      type: "spotify_playlist",
      playlistId: spotifyPlaylistMatch[1],
      rawUrl: trimmed,
    };
  }

  // 4. URL de Pista individual de Spotify
  const spotifyTrackRegex = /(?:open\.spotify\.com\/track\/|spotify:track:)([a-zA-Z0-9]{22})/i;
  const spotifyTrackMatch = trimmed.match(spotifyTrackRegex);
  if (spotifyTrackMatch) {
    return {
      type: "spotify_track",
      trackId: spotifyTrackMatch[1],
      rawUrl: trimmed,
    };
  }

  // 5. URL directa de YouTube genérica (fallback)
  if (trimmed.startsWith("http://") || trimmed.startsWith("https://")) {
    if (trimmed.includes("youtube.com") || trimmed.includes("youtu.be")) {
      return { type: "youtube_video", videoId: trimmed, rawUrl: trimmed };
    }
  }

  // 6. Texto libre para búsqueda en YouTube
  return {
    type: "search_query",
    query: trimmed,
  };
}
