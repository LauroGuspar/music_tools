const KEYWORDS_PATTERN = "\\b(?:remaster(?:ed)?|live|deluxe|edit(?:ed)?|version|mix|remix|bonus|mono|stereo|instrumental|acoustic)\\b";

export function normalizeTitle(title: string): string {
  let clean = title.toLowerCase().trim();

  // Normalize diacritics / accents
  clean = clean.normalize("NFD").replace(/[\u0300-\u036f]/g, "");

  // 1. Remove bracketed / parenthesized content containing buzzwords
  const bracketRegex = new RegExp(`[(\\[][^)\\]]*?${KEYWORDS_PATTERN}[^)\\]]*?[)\\]]`, "gi");
  clean = clean.replace(bracketRegex, "");

  // 2. Remove dash/hyphen suffixes containing buzzwords
  const suffixRegex = new RegExp(`(?:[-–—/\\\\:])\\s*(?:\\d{4}\\s+)?${KEYWORDS_PATTERN}.*`, "gi");
  clean = clean.replace(suffixRegex, "");

  // Remove apostrophes and dots to keep letters contiguous
  clean = clean.replace(/['’]/g, "");
  clean = clean.replace(/\./g, "");

  // Remove other punctuation (everything except letters, numbers, and spaces)
  clean = clean.replace(/[^a-z0-9\s]/g, " ");

  // Normalize spaces
  clean = clean.replace(/\s+/g, " ").trim();

  return clean;
}

export function normalizeArtist(artist: string): string {
  let clean = artist.toLowerCase().trim();
  clean = clean.normalize("NFD").replace(/[\u0300-\u036f]/g, "");
  clean = clean.replace(/[^a-z0-9\s]/g, " ");
  clean = clean.replace(/\s+/g, " ").trim();
  return clean;
}

export function calculateSimilarity(str1: string, str2: string): number {
  const s1 = str1.replace(/\s+/g, "");
  const s2 = str2.replace(/\s+/g, "");

  if (s1 === s2) return 1;
  if (s1.length < 2 || s2.length < 2) return 0;

  const getBigrams = (str: string) => {
    const bigrams = new Set<string>();
    for (let i = 0; i < str.length - 1; i++) {
      bigrams.add(str.substring(i, i + 2));
    }
    return bigrams;
  };

  const b1 = getBigrams(s1);
  const b2 = getBigrams(s2);

  let intersection = 0;
  for (const bigram of b1) {
    if (b2.has(bigram)) {
      intersection++;
    }
  }

  return (2 * intersection) / (b1.size + b2.size);
}

export type MatchResult = {
  matched: boolean;
  score: number;
  type: "exact" | "probable" | "none";
};

export function isMatch(
  trackA: { title: string; artists: string[] },
  trackB: { title: string; artists: string[] },
  threshold = 0.80
): MatchResult {
  const normTitleA = normalizeTitle(trackA.title);
  const normTitleB = normalizeTitle(trackB.title);

  const normArtistsA = trackA.artists.map(normalizeArtist);
  const normArtistsB = trackB.artists.map(normalizeArtist);

  const hasArtistIntersection = normArtistsA.some((a) => normArtistsB.includes(a));
  const hasAnyArtistMissing = normArtistsA.length === 0 || normArtistsB.length === 0;

  const isExactTitle = normTitleA === normTitleB;

  if (isExactTitle && hasArtistIntersection) {
    return { matched: true, score: 1.0, type: "exact" };
  }

  const titleSimilarity = calculateSimilarity(normTitleA, normTitleB);

  if (isExactTitle) {
    if (hasAnyArtistMissing) {
      return { matched: true, score: 1.0, type: "probable" };
    }
    return { matched: false, score: 0.0, type: "none" };
  }

  if (titleSimilarity >= threshold && (hasArtistIntersection || hasAnyArtistMissing)) {
    return { matched: true, score: titleSimilarity, type: "probable" };
  }

  return { matched: false, score: titleSimilarity, type: "none" };
}

export type NormalizedMatchTrack = {
  normTitle: string;
  normArtists: string[];
};

export function isMatchNormalized(
  trackA: NormalizedMatchTrack,
  trackB: NormalizedMatchTrack,
  threshold = 0.80
): MatchResult {
  const hasArtistIntersection = trackA.normArtists.some((a) => trackB.normArtists.includes(a));
  const hasAnyArtistMissing = trackA.normArtists.length === 0 || trackB.normArtists.length === 0;

  const isExactTitle = trackA.normTitle === trackB.normTitle;

  if (isExactTitle && hasArtistIntersection) {
    return { matched: true, score: 1.0, type: "exact" };
  }

  const titleSimilarity = calculateSimilarity(trackA.normTitle, trackB.normTitle);

  if (isExactTitle) {
    if (hasAnyArtistMissing) {
      return { matched: true, score: 1.0, type: "probable" };
    }
    return { matched: false, score: 0.0, type: "none" };
  }

  if (titleSimilarity >= threshold && (hasArtistIntersection || hasAnyArtistMissing)) {
    return { matched: true, score: titleSimilarity, type: "probable" };
  }

  return { matched: false, score: titleSimilarity, type: "none" };
}
