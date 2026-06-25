import { describe, expect, it } from "vitest";
import { parsePlaylistInput } from "./parsePlaylistInput";

describe("parsePlaylistInput", () => {
  it("accepts a direct playlist id", () => {
    expect(parsePlaylistInput("37i9dQZF1DXcBWIGoYBM5M")).toEqual({
      playlistId: "37i9dQZF1DXcBWIGoYBM5M",
    });
  });

  it("extracts id from a spotify playlist url", () => {
    expect(
      parsePlaylistInput("https://open.spotify.com/playlist/37i9dQZF1DXcBWIGoYBM5M")
    ).toEqual({
      playlistId: "37i9dQZF1DXcBWIGoYBM5M",
    });
  });

  it("throws on invalid input", () => {
    expect(() => parsePlaylistInput("not-valid")).toThrow(
      "Invalid Spotify playlist URL or ID",
    );
  });
});
