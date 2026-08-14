import { describe, it, expect } from "vitest";
import { parseDownloadInput } from "./parseDownloadInput";

describe("parseDownloadInput", () => {
  it("detects YouTube Playlist URLs", () => {
    const res = parseDownloadInput("https://www.youtube.com/playlist?list=PLrAXtmErZgOdP_8GztsuL9mbyL5x");
    expect(res.type).toBe("youtube_playlist");
    if (res.type === "youtube_playlist") {
      expect(res.playlistId).toBe("PLrAXtmErZgOdP_8GztsuL9mbyL5x");
    }
  });

  it("detects YouTube Video URLs", () => {
    const res1 = parseDownloadInput("https://www.youtube.com/watch?v=dQw4w9WgXcQ");
    expect(res1.type).toBe("youtube_video");
    if (res1.type === "youtube_video") {
      expect(res1.videoId).toBe("dQw4w9WgXcQ");
    }

    const res2 = parseDownloadInput("https://youtu.be/dQw4w9WgXcQ");
    expect(res2.type).toBe("youtube_video");
    if (res2.type === "youtube_video") {
      expect(res2.videoId).toBe("dQw4w9WgXcQ");
    }
  });

  it("detects Spotify Playlist URLs and URIs", () => {
    const res1 = parseDownloadInput("https://open.spotify.com/playlist/37i9dQZF1DXcBWIGoYBM5M");
    expect(res1.type).toBe("spotify_playlist");
    if (res1.type === "spotify_playlist") {
      expect(res1.playlistId).toBe("37i9dQZF1DXcBWIGoYBM5M");
    }

    const res2 = parseDownloadInput("https://open.spotify.com/intl-es/playlist/37i9dQZF1DXcBWIGoYBM5M?si=12345");
    expect(res2.type).toBe("spotify_playlist");
    if (res2.type === "spotify_playlist") {
      expect(res2.playlistId).toBe("37i9dQZF1DXcBWIGoYBM5M");
    }

    const res3 = parseDownloadInput("spotify:playlist:37i9dQZF1DXcBWIGoYBM5M");
    expect(res3.type).toBe("spotify_playlist");
    if (res3.type === "spotify_playlist") {
      expect(res3.playlistId).toBe("37i9dQZF1DXcBWIGoYBM5M");
    }
  });

  it("detects Spotify Album URLs and URIs", () => {
    const res1 = parseDownloadInput("https://open.spotify.com/album/4aawyAB9vmqN3uQ7FjRGTy");
    expect(res1.type).toBe("spotify_album");
    if (res1.type === "spotify_album") {
      expect(res1.albumId).toBe("4aawyAB9vmqN3uQ7FjRGTy");
    }

    const res2 = parseDownloadInput("https://open.spotify.com/intl-pt/album/4aawyAB9vmqN3uQ7FjRGTy?si=abcdef");
    expect(res2.type).toBe("spotify_album");
    if (res2.type === "spotify_album") {
      expect(res2.albumId).toBe("4aawyAB9vmqN3uQ7FjRGTy");
    }

    const res3 = parseDownloadInput("spotify:album:4aawyAB9vmqN3uQ7FjRGTy");
    expect(res3.type).toBe("spotify_album");
    if (res3.type === "spotify_album") {
      expect(res3.albumId).toBe("4aawyAB9vmqN3uQ7FjRGTy");
    }
  });

  it("detects Spotify Track URLs and URIs", () => {
    const res1 = parseDownloadInput("https://open.spotify.com/track/4cOdK2wGLETKBW3PvgPWqT");
    expect(res1.type).toBe("spotify_track");
    if (res1.type === "spotify_track") {
      expect(res1.trackId).toBe("4cOdK2wGLETKBW3PvgPWqT");
    }

    const res2 = parseDownloadInput("https://open.spotify.com/intl-es/track/4cOdK2wGLETKBW3PvgPWqT?si=xyz");
    expect(res2.type).toBe("spotify_track");
    if (res2.type === "spotify_track") {
      expect(res2.trackId).toBe("4cOdK2wGLETKBW3PvgPWqT");
    }

    const res3 = parseDownloadInput("spotify:track:4cOdK2wGLETKBW3PvgPWqT");
    expect(res3.type).toBe("spotify_track");
    if (res3.type === "spotify_track") {
      expect(res3.trackId).toBe("4cOdK2wGLETKBW3PvgPWqT");
    }
  });

  it("falls back to free text search", () => {
    const res = parseDownloadInput("Daft Punk - Get Lucky");
    expect(res.type).toBe("search_query");
    if (res.type === "search_query") {
      expect(res.query).toBe("Daft Punk - Get Lucky");
    }
  });
});
