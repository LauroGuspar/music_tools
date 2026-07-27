import { describe, expect, it, vi } from "vitest";
import { saveExportedFile } from "./saveExportedFile";
import { loadSettings } from "../../utils/settings";

vi.mock("../../utils/settings", () => ({
  loadSettings: vi.fn().mockReturnValue({
    defaultLocalMusicFolder: "",
    defaultPlaylistExportFolder: "",
    defaultSongDownloadFolder: "",
    fuzzyThreshold: 0.8,
  }),
}));

describe("saveExportedFile", () => {
  it("uses a json extension and suggested name for json exports", async () => {
    const pickSavePath = vi.fn().mockResolvedValue("/tmp/road-trip.json");
    const writeFile = vi.fn().mockResolvedValue(undefined);

    await expect(
      saveExportedFile({
        content: "[]",
        format: "json",
        pickSavePath,
        playlistName: "Road Trip",
        writeFile,
      }),
    ).resolves.toBe(true);

    expect(pickSavePath).toHaveBeenCalledWith({
      defaultPath: "road-trip.json",
      filters: [{ extensions: ["json"], name: "JSON" }],
    });
    expect(writeFile).toHaveBeenCalledWith({
      content: "[]",
      path: "/tmp/road-trip.json",
    });
  });

  it("uses a txt extension and falls back to playlist export name", async () => {
    const pickSavePath = vi.fn().mockResolvedValue("/tmp/playlist-export.txt");
    const writeFile = vi.fn().mockResolvedValue(undefined);

    await expect(
      saveExportedFile({
        content: "hello",
        format: "txt",
        pickSavePath,
        playlistName: "",
        writeFile,
      }),
    ).resolves.toBe(true);

    expect(pickSavePath).toHaveBeenCalledWith({
      defaultPath: "playlist-export.txt",
      filters: [{ extensions: ["txt"], name: "TXT" }],
    });
    expect(writeFile).toHaveBeenCalledWith({
      content: "hello",
      path: "/tmp/playlist-export.txt",
    });
  });

  it("does nothing when the user cancels the save dialog", async () => {
    const pickSavePath = vi.fn().mockResolvedValue(null);
    const writeFile = vi.fn().mockResolvedValue(undefined);

    await expect(
      saveExportedFile({
        content: "hello",
        format: "txt",
        pickSavePath,
        playlistName: "Road Trip",
        writeFile,
      }),
    ).resolves.toBe(false);

    expect(writeFile).not.toHaveBeenCalled();
  });

  it("appends default playlist export folder to path if present in settings", async () => {
    vi.mocked(loadSettings).mockReturnValueOnce({
      spotifyClientId: "",
      defaultLocalMusicFolder: "",
      defaultPlaylistExportFolder: "/my/custom/downloads",
      defaultSongDownloadFolder: "",
      fuzzyThreshold: 0.8,
    });

    const pickSavePath = vi.fn().mockResolvedValue("/my/custom/downloads/road-trip.json");
    const writeFile = vi.fn().mockResolvedValue(undefined);

    await expect(
      saveExportedFile({
        content: "[]",
        format: "json",
        pickSavePath,
        playlistName: "Road Trip",
        writeFile,
      }),
    ).resolves.toBe(true);

    expect(pickSavePath).toHaveBeenCalledWith({
      defaultPath: "/my/custom/downloads/road-trip.json",
      filters: [{ extensions: ["json"], name: "JSON" }],
    });
  });
});
