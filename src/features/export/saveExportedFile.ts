import { invoke } from "@tauri-apps/api/core";
import { save } from "@tauri-apps/plugin-dialog";

type ExportFormat = "json" | "txt";

type SaveExportedFileOptions = {
  content: string;
  format: ExportFormat;
  pickSavePath?: (options: {
    defaultPath: string;
    filters: Array<{ extensions: string[]; name: string }>;
  }) => Promise<string | null>;
  playlistName: string;
  writeFile?: (payload: { content: string; path: string }) => Promise<void>;
};

export async function saveExportedFile(options: SaveExportedFileOptions): Promise<boolean> {
  const defaultFileName = buildDefaultFileName(options.playlistName, options.format);
  const pickSavePath = options.pickSavePath ?? save;
  const writeFile = options.writeFile ?? ((payload) => invoke("write_exported_file", { payload }));
  const path = await pickSavePath({
    defaultPath: defaultFileName,
    filters: [{ extensions: [options.format], name: options.format.toUpperCase() }],
  });

  if (!path) {
    return false;
  }

  await writeFile({
    content: options.content,
    path,
  });

  return true;
}

function buildDefaultFileName(playlistName: string, format: ExportFormat): string {
  const baseName = slugify(playlistName) || "playlist-export";
  return `${baseName}.${format}`;
}

function slugify(value: string): string {
  return value
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "");
}

export type { ExportFormat, SaveExportedFileOptions };
