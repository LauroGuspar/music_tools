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

  it("detects Spotify Playlist URLs", () => {
    const res = parseDownloadInput("https://open.spotify.com/playlist/37i9dQZF1DXcBWIGoYBM5M");
    expect(res.type).toBe("spotify_playlist");
    if (res.type === "spotify_playlist") {
      expect(res.playlistId).toBe("37i9dQZF1DXcBWIGoYBM5M");
    }
  });

  it("detects Spotify Track URLs", () => {
    const res = parseDownloadInput("https://open.spotify.com/track/4cOdK2wGLETKBW3PvgPWqT");
    expect(res.type).toBe("spotify_track");
    if (res.type === "spotify_track") {
      expect(res.trackId).toBe("4cOdK2wGLETKBW3PvgPWqT");
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
