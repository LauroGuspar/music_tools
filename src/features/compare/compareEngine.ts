import type { ExportTrack } from "../../types/export-track";
import { isMatchNormalized, normalizeTitle, normalizeArtist } from "../../utils/songMatcher";

export type ComparisonResult = {
  exactMatches: Array<{ source: ExportTrack; target: ExportTrack }>;
  probableMatches: Array<{ source: ExportTrack; target: ExportTrack; score: number }>;
  missing: ExportTrack[];
  extra: ExportTrack[];
};

type TrackWithNormalized = {
  track: ExportTrack;
  normTitle: string;
  normArtists: string[];
};

function toNormalized(t: ExportTrack): TrackWithNormalized {
  return {
    track: t,
    normTitle: normalizeTitle(t.title),
    normArtists: t.artists.map(normalizeArtist),
  };
}

export function compareTracks(
  sourceTracks: ExportTrack[],
  targetTracks: ExportTrack[],
  threshold = 0.80
): ComparisonResult {
  const exactMatches: ComparisonResult["exactMatches"] = [];
  const probableMatches: ComparisonResult["probableMatches"] = [];
  const missing: ExportTrack[] = [];

  const normSource = sourceTracks.map(toNormalized);
  const remainingTargets = targetTracks.map(toNormalized);

  for (const source of normSource) {
    let matchedIndex = -1;
    for (let i = 0; i < remainingTargets.length; i++) {
      const matchResult = isMatchNormalized(source, remainingTargets[i], threshold);
      if (matchResult.matched && matchResult.type === "exact") {
        matchedIndex = i;
        exactMatches.push({ source: source.track, target: remainingTargets[i].track });
        break;
      }
    }

    if (matchedIndex !== -1) {
      remainingTargets.splice(matchedIndex, 1);
      continue;
    }

    let bestProbableIndex = -1;
    let bestScore = 0;

    for (let i = 0; i < remainingTargets.length; i++) {
      const matchResult = isMatchNormalized(source, remainingTargets[i], threshold);
      if (matchResult.matched && matchResult.type === "probable" && matchResult.score > bestScore) {
        bestScore = matchResult.score;
        bestProbableIndex = i;
      }
    }

    if (bestProbableIndex !== -1) {
      probableMatches.push({
        source: source.track,
        target: remainingTargets[bestProbableIndex].track,
        score: bestScore,
      });
      remainingTargets.splice(bestProbableIndex, 1);
      continue;
    }

    missing.push(source.track);
  }

  const extra = remainingTargets.map((t) => t.track);

  return {
    exactMatches,
    probableMatches,
    missing,
    extra,
  };
}

export async function compareTracksAsync(
  sourceTracks: ExportTrack[],
  targetTracks: ExportTrack[],
  onProgress?: (processed: number, total: number) => void,
  threshold = 0.80
): Promise<ComparisonResult> {
  const exactMatches: ComparisonResult["exactMatches"] = [];
  const probableMatches: ComparisonResult["probableMatches"] = [];
  const missing: ExportTrack[] = [];

  // Pre-normalizar al inicio de forma ultra-rápida
  const normSource = sourceTracks.map(toNormalized);
  const remainingTargets = targetTracks.map(toNormalized);

  const total = normSource.length;
  const chunkSize = 40; // Podemos subir el tamaño de lote a 40 ya que ahora la comparación es sumamente veloz

  for (let i = 0; i < total; i += chunkSize) {
    const chunk = normSource.slice(i, i + chunkSize);

    for (const source of chunk) {
      let matchedIndex = -1;
      for (let j = 0; j < remainingTargets.length; j++) {
        const matchResult = isMatchNormalized(source, remainingTargets[j], threshold);
        if (matchResult.matched && matchResult.type === "exact") {
          matchedIndex = j;
          exactMatches.push({ source: source.track, target: remainingTargets[j].track });
          break;
        }
      }

      if (matchedIndex !== -1) {
        remainingTargets.splice(matchedIndex, 1);
        continue;
      }

      let bestProbableIndex = -1;
      let bestScore = 0;

      for (let j = 0; j < remainingTargets.length; j++) {
        const matchResult = isMatchNormalized(source, remainingTargets[j], threshold);
        if (matchResult.matched && matchResult.type === "probable" && matchResult.score > bestScore) {
          bestScore = matchResult.score;
          bestProbableIndex = j;
        }
      }

      if (bestProbableIndex !== -1) {
        probableMatches.push({
          source: source.track,
          target: remainingTargets[bestProbableIndex].track,
          score: bestScore,
        });
        remainingTargets.splice(bestProbableIndex, 1);
        continue;
      }

      missing.push(source.track);
    }

    if (onProgress) {
      onProgress(Math.min(i + chunkSize, total), total);
    }

    // Ceder el control al event loop
    await new Promise((resolve) => setTimeout(resolve, 0));
  }

  const extra = remainingTargets.map((t) => t.track);

  return {
    exactMatches,
    probableMatches,
    missing,
    extra,
  };
}
