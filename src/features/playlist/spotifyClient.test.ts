import { describe, expect, it, vi } from "vitest";
import {
  fetchPlaylistDetails,
  fetchAlbumDetails,
  fetchTrackDetails,
  searchSpotify,
} from "./spotifyClient";

function jsonResponse(body: unknown, init: ResponseInit = {}): Response {
  return new Response(JSON.stringify(body), {
    headers: { "Content-Type": "application/json" },
    ...init,
  });
}

describe("fetchPlaylistDetails", () => {
  it("loads playlist metadata and paginates track items", async () => {
    const fetcher = vi
      .fn()
      .mockResolvedValueOnce(
        jsonResponse({
          name: "Road Trip",
          id: "playlist-123",
          tracks: { total: 3 },
        }),
      )
      .mockResolvedValueOnce(
        jsonResponse({
          items: [
            {
              track: {
                name: "One More Time",
                artists: [{ name: "Daft Punk" }],
              },
            },
            {
              track: {
                name: "Digital Love",
                artists: [{ name: "Daft Punk" }],
              },
            },
          ],
          next: "https://api.spotify.com/v1/playlists/playlist-123/items?offset=2&limit=2",
        }),
      )
      .mockResolvedValueOnce(
        jsonResponse({
          items: [
            {
              track: {
                name: "D.A.N.C.E.",
                artists: [{ name: "Justice" }],
              },
            },
          ],
          next: null,
        }),
      );

    const playlist = await fetchPlaylistDetails({
      accessToken: "token-123",
      fetcher,
      playlistId: "playlist-123",
    });

    expect(fetcher).toHaveBeenCalledTimes(3);
    expect(playlist).toEqual({
      id: "playlist-123",
      name: "Road Trip",
      tracks: [
        { title: "One More Time", artists: ["Daft Punk"] },
        { title: "Digital Love", artists: ["Daft Punk"] },
        { title: "D.A.N.C.E.", artists: ["Justice"] },
      ],
    });
  });

  it("ignores playlist items without a usable track", async () => {
    const fetcher = vi
      .fn()
      .mockResolvedValueOnce(
        jsonResponse({
          name: "Mixed",
          id: "playlist-123",
          tracks: { total: 2 },
        }),
      )
      .mockResolvedValueOnce(
        jsonResponse({
          items: [
            { track: null },
            {
              track: {
                name: "Around the World",
                artists: [{ name: "Daft Punk" }],
              },
            },
          ],
          next: null,
        }),
      );

    const playlist = await fetchPlaylistDetails({
      accessToken: "token-123",
      fetcher,
      playlistId: "playlist-123",
    });

    expect(playlist.tracks).toEqual([
      { title: "Around the World", artists: ["Daft Punk"] },
    ]);
  });

  it("supports playlist item responses that expose song data under item", async () => {
    const fetcher = vi
      .fn()
      .mockResolvedValueOnce(
        jsonResponse({
          name: "Migrated Playlist",
          id: "playlist-123",
          tracks: { total: 1 },
        }),
      )
      .mockResolvedValueOnce(
        jsonResponse({
          items: [
            {
              item: {
                name: "Harder Better Faster Stronger",
                artists: [{ name: "Daft Punk" }],
              },
            },
          ],
          next: null,
        }),
      );

    const playlist = await fetchPlaylistDetails({
      accessToken: "token-123",
      fetcher,
      playlistId: "playlist-123",
    });

    expect(playlist.tracks).toEqual([
      { title: "Harder Better Faster Stronger", artists: ["Daft Punk"] },
    ]);
  });

  it("surfaces a helpful error for spotify api failures", async () => {
    const fetcher = vi.fn().mockResolvedValueOnce(
      jsonResponse(
        {
          error: {
            status: 403,
            message: "Forbidden.",
          },
        },
        { status: 403, statusText: "Forbidden" },
      ),
    );

    await expect(
      fetchPlaylistDetails({
        accessToken: "token-123",
        fetcher,
        playlistId: "playlist-123",
      }),
    ).rejects.toThrow("Forbidden.");
  });
});

describe("fetchAlbumDetails", () => {
  it("loads album details, inherits album metadata and paginates tracks", async () => {
    const fetcher = vi
      .fn()
      .mockResolvedValueOnce(
        jsonResponse({
          id: "album-456",
          name: "Discovery",
          artists: [{ name: "Daft Punk" }],
          images: [{ url: "https://example.com/discovery.jpg" }],
          release_date: "2001-03-12",
          tracks: {
            items: [
              {
                id: "t1",
                name: "One More Time",
                artists: [{ name: "Daft Punk" }],
                duration_ms: 320000,
              },
              {
                id: "t2",
                name: "Aerodynamic",
                artists: [{ name: "Daft Punk" }],
                duration_ms: 207000,
              },
            ],
            next: "https://api.spotify.com/v1/albums/album-456/tracks?offset=2&limit=2",
          },
        }),
      )
      .mockResolvedValueOnce(
        jsonResponse({
          items: [
            {
              id: "t3",
              name: "Digital Love",
              artists: [{ name: "Daft Punk" }],
              duration_ms: 298000,
            },
          ],
          next: null,
        }),
      );

    const album = await fetchAlbumDetails({
      accessToken: "token-123",
      fetcher,
      albumId: "album-456",
    });

    expect(fetcher).toHaveBeenCalledTimes(2);
    expect(album.id).toBe("album-456");
    expect(album.name).toBe("Discovery");
    expect(album.artists).toEqual(["Daft Punk"]);
    expect(album.imageUrl).toBe("https://example.com/discovery.jpg");
    expect(album.releaseDate).toBe("2001-03-12");
    expect(album.tracks).toHaveLength(3);
    expect(album.tracks[0]).toEqual({
      title: "One More Time",
      artists: ["Daft Punk"],
      albumName: "Discovery",
      albumImageUrl: "https://example.com/discovery.jpg",
      releaseDate: "2001-03-12",
      duration: "5:20",
    });
    expect(album.tracks[1].duration).toBe("3:27");
    expect(album.tracks[2].title).toBe("Digital Love");
  });

  it("handles empty tracks in album gracefully", async () => {
    const fetcher = vi.fn().mockResolvedValueOnce(
      jsonResponse({
        id: "album-empty",
        name: "Empty Album",
        artists: [{ name: "Artist" }],
        tracks: { items: [], next: null },
      }),
    );

    const album = await fetchAlbumDetails({
      accessToken: "token-123",
      fetcher,
      albumId: "album-empty",
    });

    expect(album.tracks).toEqual([]);
  });
});

describe("fetchTrackDetails", () => {
  it("fetches single track details correctly", async () => {
    const fetcher = vi.fn().mockResolvedValueOnce(
      jsonResponse({
        id: "track-789",
        name: "Get Lucky",
        artists: [{ name: "Daft Punk" }, { name: "Pharrell Williams" }],
        album: {
          name: "Random Access Memories",
          images: [{ url: "https://example.com/ram.jpg" }],
          release_date: "2013-05-17",
        },
        duration_ms: 248000,
      }),
    );

    const track = await fetchTrackDetails({
      accessToken: "token-123",
      fetcher,
      trackId: "track-789",
    });

    expect(track).toEqual({
      id: "track-789",
      title: "Get Lucky",
      artists: ["Daft Punk", "Pharrell Williams"],
      albumName: "Random Access Memories",
      albumImageUrl: "https://example.com/ram.jpg",
      releaseDate: "2013-05-17",
      duration: "4:08",
    });
  });
});

describe("searchSpotify", () => {
  it("searches tracks, albums and playlists and normalizes the results", async () => {
    const fetcher = vi.fn().mockResolvedValueOnce(
      jsonResponse({
        tracks: {
          items: [
            {
              id: "t1",
              name: "Starboy",
              artists: [{ name: "The Weeknd" }, { name: "Daft Punk" }],
              album: {
                name: "Starboy",
                images: [{ url: "https://example.com/starboy.jpg" }],
                release_date: "2016-11-25",
              },
              duration_ms: 230000,
            },
          ],
        },
        albums: {
          items: [
            {
              id: "a1",
              name: "Starboy",
              artists: [{ name: "The Weeknd" }],
              images: [{ url: "https://example.com/starboy.jpg" }],
              release_date: "2016-11-25",
              total_tracks: 18,
            },
          ],
        },
        playlists: {
          items: [
            {
              id: "p1",
              name: "This Is The Weeknd",
              images: [{ url: "https://example.com/thisis.jpg" }],
              tracks: { total: 50 },
            },
          ],
        },
      }),
    );

    const results = await searchSpotify({
      accessToken: "token-123",
      fetcher,
      query: "The Weeknd",
    });

    expect(results.tracks).toHaveLength(1);
    expect(results.tracks[0].title).toBe("Starboy");
    expect(results.tracks[0].duration).toBe("3:50");

    expect(results.albums).toHaveLength(1);
    expect(results.albums[0].name).toBe("Starboy");
    expect(results.albums[0].totalTracks).toBe(18);

    expect(results.playlists).toHaveLength(1);
    expect(results.playlists[0].name).toBe("This Is The Weeknd");
    expect(results.playlists[0].totalTracks).toBe(50);
  });
});
