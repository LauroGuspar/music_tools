const PLAYLIST_ID_REGEX = /^[A-Za-z0-9]{22}$/;

export function parsePlaylistInput(input: string): { playlistId: string } {
  const value = input.trim();

  if (PLAYLIST_ID_REGEX.test(value)) {
    return { playlistId: value };
  }

  try {
    const url = new URL(value);
    const match = url.pathname.match(/^\/playlist\/([A-Za-z0-9]{22})$/);

    if (url.hostname.includes("spotify.com") && match) {
      return { playlistId: match[1] };
    }
  } catch {
    // Fall through to the shared validation error below.
  }

  throw new Error("Invalid Spotify playlist URL or ID");
}
