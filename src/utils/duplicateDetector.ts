import type { StagingBlockData, StagingTrack } from "../components/StagingWorkspace";
import type { ExportTrack } from "../types/export-track";
import { normalizeTitle, normalizeArtist, isMatchNormalized, type MatchResult } from "./songMatcher";

export interface DuplicateMatchInfo {
  blockId: string;
  blockTitle: string;
  trackId: string;
  trackTitle: string;
  artistOrChannel: string;
  matchType: "exact_url" | "exact_meta" | "probable";
  score: number;
}

export interface DuplicateAnalysisResult {
  totalTracks: number;
  duplicateCount: number;
  duplicateTrackIds: Set<string>;
  duplicateMap: Map<string, DuplicateMatchInfo[]>; // trackId -> list of duplicate info
}

/**
 * Analyzes staging blocks to detect duplicate tracks based on URL and normalized title + artist metadata.
 */
export function analyzeStagingDuplicates(blocks: StagingBlockData[]): DuplicateAnalysisResult {
  const allTracksWithMeta: Array<{
    blockId: string;
    blockTitle: string;
    track: StagingTrack;
    normTitle: string;
    normArtists: string[];
  }> = [];

  let totalTracks = 0;

  for (const block of blocks) {
    for (const track of block.tracks) {
      totalTracks++;
      const artistsList = track.artistOrChannel ? track.artistOrChannel.split(/[,/&]+/).map((a) => a.trim()) : [];
      allTracksWithMeta.push({
        blockId: block.id,
        blockTitle: block.title,
        track,
        normTitle: normalizeTitle(track.title),
        normArtists: artistsList.map(normalizeArtist),
      });
    }
  }

  const duplicateTrackIds = new Set<string>();
  const duplicateMap = new Map<string, DuplicateMatchInfo[]>();

  for (let i = 0; i < allTracksWithMeta.length; i++) {
    const current = allTracksWithMeta[i];

    for (let j = i + 1; j < allTracksWithMeta.length; j++) {
      const other = allTracksWithMeta[j];

      let isDup = false;
      let matchType: "exact_url" | "exact_meta" | "probable" = "probable";
      let score = 0;

      // 1. Check exact videoUrl match if both exist
      if (current.track.videoUrl && other.track.videoUrl && current.track.videoUrl === other.track.videoUrl) {
        isDup = true;
        matchType = "exact_url";
        score = 1.0;
      } else {
        // 2. Check metadata match
        const matchRes: MatchResult = isMatchNormalized(
          { normTitle: current.normTitle, normArtists: current.normArtists },
          { normTitle: other.normTitle, normArtists: other.normArtists },
          0.82
        );

        if (matchRes.matched) {
          isDup = true;
          matchType = matchRes.type === "exact" ? "exact_meta" : "probable";
          score = matchRes.score;
        }
      }

      if (isDup) {
        duplicateTrackIds.add(current.track.id);
        duplicateTrackIds.add(other.track.id);

        const listCurrent = duplicateMap.get(current.track.id) || [];
        listCurrent.push({
          blockId: other.blockId,
          blockTitle: other.blockTitle,
          trackId: other.track.id,
          trackTitle: other.track.title,
          artistOrChannel: other.track.artistOrChannel,
          matchType,
          score,
        });
        duplicateMap.set(current.track.id, listCurrent);

        const listOther = duplicateMap.get(other.track.id) || [];
        listOther.push({
          blockId: current.blockId,
          blockTitle: current.blockTitle,
          trackId: current.track.id,
          trackTitle: current.track.title,
          artistOrChannel: current.track.artistOrChannel,
          matchType,
          score,
        });
        duplicateMap.set(other.track.id, listOther);
      }
    }
  }

  return {
    totalTracks,
    duplicateCount: duplicateTrackIds.size,
    duplicateTrackIds,
    duplicateMap,
  };
}

/**
 * Removes duplicate tracks from staging blocks, keeping the first instance of each duplicate set.
 */
export function deduplicateStagingBlocks(blocks: StagingBlockData[]): {
  newBlocks: StagingBlockData[];
  removedCount: number;
} {
  const analysis = analyzeStagingDuplicates(blocks);
  if (analysis.duplicateCount === 0) {
    return { newBlocks: blocks, removedCount: 0 };
  }

  const seenKeys = new Set<string>();
  let removedCount = 0;

  const newBlocks = blocks
    .map((block) => {
      const filteredTracks = block.tracks.filter((track) => {
        const normTitle = normalizeTitle(track.title);
        const normArtists = track.artistOrChannel
          ? track.artistOrChannel.split(/[,/&]+/).map((a) => normalizeArtist(a.trim())).sort().join("|")
          : "";
        
        // Key for duplicate grouping
        const key = track.videoUrl
          ? `url:${track.videoUrl}`
          : `meta:${normTitle}::${normArtists}`;

        if (seenKeys.has(key)) {
          removedCount++;
          return false;
        }

        seenKeys.add(key);
        return true;
      });

      return {
        ...block,
        tracks: filteredTracks,
      };
    })
    .filter((block) => block.tracks.length > 0);

  return { newBlocks, removedCount };
}

/**
 * Helper to convert staging tracks to ExportTrack format for export/downloads/comparison.
 */
export function stagingToExportTracks(
  blocks: StagingBlockData[],
  selectedMap?: Record<string, Set<string>>
): ExportTrack[] {
  const result: ExportTrack[] = [];

  for (const block of blocks) {
    const selectedSet = selectedMap ? selectedMap[block.id] : null;
    for (const track of block.tracks) {
      if (!selectedSet || selectedSet.has(track.id)) {
        result.push({
          title: track.title,
          artists: track.artistOrChannel ? track.artistOrChannel.split(/[,/&]+/).map((a) => a.trim()) : ["Desconocido"],
          albumName: track.albumName,
          albumImageUrl: track.thumbnailUrl,
          releaseDate: track.releaseDate,
        });
      }
    }
  }

  return result;
}
