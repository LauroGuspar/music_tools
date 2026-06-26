import type { ExportTrack } from "../../types/export-track";
import { isMatch } from "../../utils/songMatcher";

export type ComparisonResult = {
  exactMatches: Array<{ source: ExportTrack; target: ExportTrack }>;
  probableMatches: Array<{ source: ExportTrack; target: ExportTrack; score: number }>;
  missing: ExportTrack[];
  extra: ExportTrack[];
};

export function compareTracks(
  sourceTracks: ExportTrack[],
  targetTracks: ExportTrack[]
): ComparisonResult {
  const exactMatches: ComparisonResult["exactMatches"] = [];
  const probableMatches: ComparisonResult["probableMatches"] = [];
  const missing: ExportTrack[] = [];

  const remainingTargets = [...targetTracks];

  for (const source of sourceTracks) {
    let matchedIndex = -1;
    for (let i = 0; i < remainingTargets.length; i++) {
      const matchResult = isMatch(source, remainingTargets[i]);
      if (matchResult.matched && matchResult.type === "exact") {
        matchedIndex = i;
        exactMatches.push({ source, target: remainingTargets[i] });
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
      const matchResult = isMatch(source, remainingTargets[i]);
      if (matchResult.matched && matchResult.type === "probable" && matchResult.score > bestScore) {
        bestScore = matchResult.score;
        bestProbableIndex = i;
      }
    }

    if (bestProbableIndex !== -1) {
      probableMatches.push({
        source,
        target: remainingTargets[bestProbableIndex],
        score: bestScore,
      });
      remainingTargets.splice(bestProbableIndex, 1);
      continue;
    }

    missing.push(source);
  }

  const extra = remainingTargets;

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
  onProgress?: (processed: number, total: number) => void
): Promise<ComparisonResult> {
  const exactMatches: ComparisonResult["exactMatches"] = [];
  const probableMatches: ComparisonResult["probableMatches"] = [];
  const missing: ExportTrack[] = [];

  const remainingTargets = [...targetTracks];
  const total = sourceTracks.length;
  const chunkSize = 20;

  for (let i = 0; i < total; i += chunkSize) {
    const chunk = sourceTracks.slice(i, i + chunkSize);

    for (const source of chunk) {
      let matchedIndex = -1;
      for (let j = 0; j < remainingTargets.length; j++) {
        const matchResult = isMatch(source, remainingTargets[j]);
        if (matchResult.matched && matchResult.type === "exact") {
          matchedIndex = j;
          exactMatches.push({ source, target: remainingTargets[j] });
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
        const matchResult = isMatch(source, remainingTargets[j]);
        if (matchResult.matched && matchResult.type === "probable" && matchResult.score > bestScore) {
          bestScore = matchResult.score;
          bestProbableIndex = j;
        }
      }

      if (bestProbableIndex !== -1) {
        probableMatches.push({
          source,
          target: remainingTargets[bestProbableIndex],
          score: bestScore,
        });
        remainingTargets.splice(bestProbableIndex, 1);
        continue;
      }

      missing.push(source);
    }

    if (onProgress) {
      onProgress(Math.min(i + chunkSize, total), total);
    }

    // Permitir al navegador procesar el renderizado
    await new Promise((resolve) => setTimeout(resolve, 0));
  }

  const extra = remainingTargets;

  return {
    exactMatches,
    probableMatches,
    missing,
    extra,
  };
}
