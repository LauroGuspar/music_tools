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
