const PLAYLIST_ID_REGEX = /^[A-Za-z0-9]{22}$/;

export function parsePlaylistInput(input: string): { playlistId: string } {
  const value = input.trim();

  // Validar si es directamente el ID
  if (PLAYLIST_ID_REGEX.test(value)) {
    return { playlistId: value };
  }

  // Soportar formato de URI: spotify:playlist:ID
  if (value.startsWith("spotify:playlist:")) {
    const parts = value.split(":");
    const id = parts[2];
    if (id && PLAYLIST_ID_REGEX.test(id)) {
      return { playlistId: id };
    }
  }

  // Soportar URLs de Spotify
  try {
    const url = new URL(value);
    
    // Captura opcionalmente prefijos como /user/{username}, /embed o /intl-xx
    const match = url.pathname.match(/(?:\/user\/[^/]+|\/embed|\/intl-[a-zA-Z-]+)?\/playlist\/([A-Za-z0-9]{22})/);

    if (url.hostname.includes("spotify.com") && match) {
      return { playlistId: match[1] };
    }
  } catch {
    // Fall through to the shared validation error below.
  }

  throw new Error("Invalid Spotify playlist URL or ID");
}
