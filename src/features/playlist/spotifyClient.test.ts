import { describe, expect, it, vi } from "vitest";
import { fetchPlaylistDetails } from "./spotifyClient";

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
