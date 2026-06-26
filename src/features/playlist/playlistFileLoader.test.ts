import { describe, expect, it } from "vitest";
import { parsePlaylistTxt, parsePlaylistJson, parsePlaylistFile } from "./playlistFileLoader";

describe("playlistFileLoader", () => {
  describe("parsePlaylistTxt", () => {
    it("parses classic title - artist lines", () => {
      const content = "Yesterday - The Beatles\nCreep - Radiohead, Thom Yorke\nLet It Be";
      const tracks = parsePlaylistTxt(content);

      expect(tracks).toHaveLength(3);
      expect(tracks[0]).toEqual({ title: "Yesterday", artists: ["The Beatles"] });
      expect(tracks[1]).toEqual({ title: "Creep", artists: ["Radiohead", "Thom Yorke"] });
      expect(tracks[2]).toEqual({ title: "Let It Be", artists: [] });
    });

    it("ignores empty lines", () => {
      const content = "\n\nSong A - Artist A\n\n";
      const tracks = parsePlaylistTxt(content);
      expect(tracks).toHaveLength(1);
      expect(tracks[0].title).toBe("Song A");
    });
  });

  describe("parsePlaylistJson", () => {
    it("parses valid array of ExportTrack", () => {
      const content = JSON.stringify([
        { title: "Song 1", artists: ["Artist 1", "Artist 2"] },
        { title: "Song 2", artists: ["Artist 3"] },
      ]);
      const tracks = parsePlaylistJson(content);
      expect(tracks).toHaveLength(2);
      expect(tracks[0].title).toBe("Song 1");
      expect(tracks[0].artists).toEqual(["Artist 1", "Artist 2"]);
    });

    it("parses alternative shapes (name/artist)", () => {
      const content = JSON.stringify([
        { name: "Song 1", artist: "Artist 1" },
      ]);
      const tracks = parsePlaylistJson(content);
      expect(tracks).toHaveLength(1);
      expect(tracks[0]).toEqual({ title: "Song 1", artists: ["Artist 1"] });
    });

    it("throws error for invalid JSON shapes", () => {
      expect(() => parsePlaylistJson('{"title": "not an array"}')).toThrow();
      expect(() => parsePlaylistJson(JSON.stringify([{ artists: ["Artist"] }]))).toThrow();
    });
  });

  describe("parsePlaylistFile", () => {
    it("parses correctly based on filename", () => {
      const jsonContent = JSON.stringify([{ title: "Song", artists: ["Artist"] }]);
      const txtContent = "Song - Artist";

      expect(parsePlaylistFile(jsonContent, "playlist.json")).toHaveLength(1);
      expect(parsePlaylistFile(txtContent, "playlist.txt")).toHaveLength(1);
    });
  });
});
