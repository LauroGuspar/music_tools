import type { ExportTrack } from "../../types/export-track";

type SpotifyPlaylistDetails = {
  accessToken: string;
  fetcher?: typeof fetch;
  playlistId: string;
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
    } | null;
    track?: {
      name?: string;
      artists?: Array<{ name?: string }>;
      album?: {
        name?: string;
        images?: Array<{ url?: string }>;
        release_date?: string;
      };
    } | null;
  }>;
  next?: string | null;
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
  let nextUrl: string | null = `${SPOTIFY_API_BASE_URL}/playlists/${options.playlistId}/items?fields=items(track(name,artists(name),album(name,images,release_date)),item(name,artists(name),album(name,images,release_date))),next&limit=100`;

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

function normalizePlaylistItems(items: NonNullable<PlaylistItemsResponse["items"]>): ExportTrack[] {
  return items.flatMap((item) => {
    const playlistEntry = item.item ?? item.track;
    const trackName = playlistEntry?.name?.trim();
    const artists = playlistEntry?.artists?.map((artist) => artist.name?.trim()).filter(isNonEmptyString) ?? [];
    const albumName = playlistEntry?.album?.name?.trim();
    const albumImageUrl = playlistEntry?.album?.images?.[0]?.url;
    const releaseDate = playlistEntry?.album?.release_date;

    if (!trackName || artists.length === 0) {
      return [];
    }

    return [{
      title: trackName,
      artists,
      albumName,
      albumImageUrl,
      releaseDate,
    }];
  });
}

function isNonEmptyString(value: string | undefined): value is string {
  return typeof value === "string" && value.length > 0;
}

export type { PlaylistDetails, SpotifyPlaylistDetails };
