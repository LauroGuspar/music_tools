import type { ExportTrack } from "../../types/export-track";

type SpotifyPlaylistDetails = {
  accessToken: string;
  fetcher?: typeof fetch;
  playlistId: string;
};

type SpotifyAlbumDetails = {
  accessToken: string;
  fetcher?: typeof fetch;
  albumId: string;
};

type SpotifyTrackDetails = {
  accessToken: string;
  fetcher?: typeof fetch;
  trackId: string;
};

type SpotifySearchOptions = {
  accessToken: string;
  fetcher?: typeof fetch;
  query: string;
  types?: Array<"track" | "album" | "playlist">;
  limit?: number;
};

type PlaylistSummaryResponse = {
  id: string;
  name: string;
};

type PlaylistItemsResponse = {
  items?: Array<{
    item?: {
      name?: string;
      artists?: Array<{ name?: string }>;
      album?: {
        name?: string;
        images?: Array<{ url?: string }>;
        release_date?: string;
      };
      duration_ms?: number;
    } | null;
    track?: {
      name?: string;
      artists?: Array<{ name?: string }>;
      album?: {
        name?: string;
        images?: Array<{ url?: string }>;
        release_date?: string;
      };
      duration_ms?: number;
    } | null;
  }>;
  next?: string | null;
};

type SingleTrackResponse = {
  id: string;
  name: string;
  artists?: Array<{ name?: string }>;
  album?: {
    name?: string;
    images?: Array<{ url?: string }>;
    release_date?: string;
  };
  duration_ms?: number;
};

type AlbumSummaryResponse = {
  id: string;
  name: string;
  artists?: Array<{ name?: string }>;
  images?: Array<{ url?: string }>;
  release_date?: string;
  tracks?: {
    items?: Array<{
      id?: string;
      name?: string;
      artists?: Array<{ name?: string }>;
      duration_ms?: number;
    }>;
    next?: string | null;
  };
};

type AlbumTracksPageResponse = {
  items?: Array<{
    id?: string;
    name?: string;
    artists?: Array<{ name?: string }>;
    duration_ms?: number;
  }>;
  next?: string | null;
};

type SpotifySearchResponse = {
  tracks?: {
    items?: SingleTrackResponse[];
    total?: number;
  };
  albums?: {
    items?: Array<{
      id: string;
      name: string;
      artists?: Array<{ name?: string }>;
      images?: Array<{ url?: string }>;
      release_date?: string;
      total_tracks?: number;
    }>;
    total?: number;
  };
  playlists?: {
    items?: Array<{
      id: string;
      name: string;
      images?: Array<{ url?: string }>;
      tracks?: { total?: number };
    } | null>;
    total?: number;
  };
};

type SpotifyApiErrorResponse = {
  error?: {
    message?: string;
    status?: number;
  };
};

type PlaylistDetails = {
  id: string;
  name: string;
  tracks: ExportTrack[];
};

type AlbumDetails = {
  id: string;
  name: string;
  artists?: string[];
  imageUrl?: string;
  releaseDate?: string;
  tracks: ExportTrack[];
};

type SpotifySearchResults = {
  tracks: Array<ExportTrack & { id: string }>;
  albums: Array<{
    id: string;
    name: string;
    artists: string[];
    imageUrl?: string;
    releaseDate?: string;
    totalTracks?: number;
  }>;
  playlists: Array<{
    id: string;
    name: string;
    imageUrl?: string;
    totalTracks?: number;
  }>;
};

const SPOTIFY_API_BASE_URL = "https://api.spotify.com/v1";

export async function fetchPlaylistDetails(
  options: SpotifyPlaylistDetails,
): Promise<PlaylistDetails> {
  const fetcher = options.fetcher ?? fetch;
  const summary = await spotifyGet<PlaylistSummaryResponse>(
    fetcher,
    `${SPOTIFY_API_BASE_URL}/playlists/${options.playlistId}?fields=id,name`,
    options.accessToken,
  );

  const tracks: ExportTrack[] = [];
  let nextUrl: string | null = `${SPOTIFY_API_BASE_URL}/playlists/${options.playlistId}/items?fields=items(track(name,duration_ms,artists(name),album(name,images,release_date)),item(name,duration_ms,artists(name),album(name,images,release_date))),next&limit=100`;

  while (nextUrl) {
    const page: PlaylistItemsResponse = await spotifyGet<PlaylistItemsResponse>(
      fetcher,
      nextUrl,
      options.accessToken,
    );
    tracks.push(...normalizePlaylistItems(page.items ?? []));
    nextUrl = page.next ?? null;
  }

  return {
    id: summary.id,
    name: summary.name,
    tracks,
  };
}

export async function fetchAlbumDetails(
  options: SpotifyAlbumDetails,
): Promise<AlbumDetails> {
  const fetcher = options.fetcher ?? fetch;
  const album = await spotifyGet<AlbumSummaryResponse>(
    fetcher,
    `${SPOTIFY_API_BASE_URL}/albums/${options.albumId}`,
    options.accessToken,
  );

  const albumName = album.name?.trim() || "Álbum de Spotify";
  const albumImageUrl = album.images?.[0]?.url;
  const releaseDate = album.release_date;
  const albumArtists = album.artists?.map((a) => a.name?.trim()).filter(isNonEmptyString) ?? [];

  const tracks: ExportTrack[] = [];

  if (album.tracks?.items) {
    for (const item of album.tracks.items) {
      const trackName = item.name?.trim();
      const trackArtists = item.artists?.map((a) => a.name?.trim()).filter(isNonEmptyString) ?? [];
      if (trackName) {
        tracks.push({
          title: trackName,
          artists: trackArtists.length > 0 ? trackArtists : albumArtists,
          albumName,
          albumImageUrl,
          releaseDate,
          duration: formatDurationMs(item.duration_ms),
        });
      }
    }
  }

  let nextUrl = album.tracks?.next ?? null;
  while (nextUrl) {
    const page: AlbumTracksPageResponse = await spotifyGet<AlbumTracksPageResponse>(
      fetcher,
      nextUrl,
      options.accessToken,
    );
    if (page.items) {
      for (const item of page.items) {
        const trackName = item.name?.trim();
        const trackArtists = item.artists?.map((a) => a.name?.trim()).filter(isNonEmptyString) ?? [];
        if (trackName) {
          tracks.push({
            title: trackName,
            artists: trackArtists.length > 0 ? trackArtists : albumArtists,
            albumName,
            albumImageUrl,
            releaseDate,
            duration: formatDurationMs(item.duration_ms),
          });
        }
      }
    }
    nextUrl = page.next ?? null;
  }

  return {
    id: album.id,
    name: albumName,
    artists: albumArtists,
    imageUrl: albumImageUrl,
    releaseDate,
    tracks,
  };
}

export async function fetchTrackDetails(
  options: SpotifyTrackDetails,
): Promise<ExportTrack & { id: string }> {
  const fetcher = options.fetcher ?? fetch;
  const data = await spotifyGet<SingleTrackResponse>(
    fetcher,
    `${SPOTIFY_API_BASE_URL}/tracks/${options.trackId}`,
    options.accessToken,
  );

  const artists = data.artists?.map((a) => a.name?.trim()).filter(isNonEmptyString) ?? [];
  return {
    id: data.id,
    title: data.name?.trim() || "Canción de Spotify",
    artists: artists.length > 0 ? artists : ["Artista Desconocido"],
    albumName: data.album?.name?.trim(),
    albumImageUrl: data.album?.images?.[0]?.url,
    releaseDate: data.album?.release_date,
    duration: formatDurationMs(data.duration_ms),
  };
}

export async function searchSpotify(
  options: SpotifySearchOptions,
): Promise<SpotifySearchResults> {
  const fetcher = options.fetcher ?? fetch;
  const types = options.types ?? ["track", "album", "playlist"];
  const limit = options.limit ?? 10;
  const searchUrl = `${SPOTIFY_API_BASE_URL}/search?q=${encodeURIComponent(options.query)}&type=${types.join(",")}&limit=${limit}`;

  const res = await spotifyGet<SpotifySearchResponse>(
    fetcher,
    searchUrl,
    options.accessToken,
  );

  const tracks: Array<ExportTrack & { id: string }> = (res.tracks?.items ?? []).flatMap((t) => {
    const title = t.name?.trim();
    const artists = t.artists?.map((a) => a.name?.trim()).filter(isNonEmptyString) ?? [];
    if (!title) return [];
    return [{
      id: t.id,
      title,
      artists: artists.length > 0 ? artists : ["Artista Desconocido"],
      albumName: t.album?.name?.trim(),
      albumImageUrl: t.album?.images?.[0]?.url,
      releaseDate: t.album?.release_date,
      duration: formatDurationMs(t.duration_ms),
    }];
  });

  const albums = (res.albums?.items ?? []).flatMap((a) => {
    const name = a.name?.trim();
    if (!name) return [];
    return [{
      id: a.id,
      name,
      artists: a.artists?.map((art) => art.name?.trim()).filter(isNonEmptyString) ?? [],
      imageUrl: a.images?.[0]?.url,
      releaseDate: a.release_date,
      totalTracks: a.total_tracks,
    }];
  });

  const playlists = (res.playlists?.items ?? []).flatMap((p) => {
    if (!p || !p.name) return [];
    return [{
      id: p.id,
      name: p.name.trim(),
      imageUrl: p.images?.[0]?.url,
      totalTracks: p.tracks?.total,
    }];
  });

  return {
    tracks,
    albums,
    playlists,
  };
}

async function spotifyGet<T>(
  fetcher: typeof fetch,
  url: string,
  accessToken: string,
): Promise<T> {
  const response = await fetcher(url, {
    headers: {
      Authorization: `Bearer ${accessToken}`,
    },
  });

  const payload = (await response.json()) as T | SpotifyApiErrorResponse;

  if (!response.ok) {
    const errorPayload = payload as SpotifyApiErrorResponse;
    throw new Error(errorPayload.error?.message ?? `Spotify request failed with ${response.status}`);
  }

  return payload as T;
}

function formatDurationMs(durationMs?: number): string | undefined {
  if (!durationMs || isNaN(durationMs)) return undefined;
  const totalSeconds = Math.floor(durationMs / 1000);
  const minutes = Math.floor(totalSeconds / 60);
  const seconds = totalSeconds % 60;
  return `${minutes}:${seconds.toString().padStart(2, "0")}`;
}

function normalizePlaylistItems(items: NonNullable<PlaylistItemsResponse["items"]>): ExportTrack[] {
  return items.flatMap((item) => {
    const playlistEntry = item.item ?? item.track;
    const trackName = playlistEntry?.name?.trim();
    const artists = playlistEntry?.artists?.map((artist) => artist.name?.trim()).filter(isNonEmptyString) ?? [];
    const albumName = playlistEntry?.album?.name?.trim();
    const albumImageUrl = playlistEntry?.album?.images?.[0]?.url;
    const releaseDate = playlistEntry?.album?.release_date;
    const duration = formatDurationMs(playlistEntry?.duration_ms);

    if (!trackName || artists.length === 0) {
      return [];
    }

    return [{
      title: trackName,
      artists,
      albumName,
      albumImageUrl,
      releaseDate,
      duration,
    }];
  });
}

function isNonEmptyString(value: string | undefined): value is string {
  return typeof value === "string" && value.length > 0;
}

export type {
  PlaylistDetails,
  SpotifyPlaylistDetails,
  AlbumDetails,
  SpotifyAlbumDetails,
  SpotifyTrackDetails,
  SpotifySearchOptions,
  SpotifySearchResults,
};
