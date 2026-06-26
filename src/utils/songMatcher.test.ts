import { describe, expect, it } from "vitest";
import { normalizeTitle, normalizeArtist, calculateSimilarity, isMatch } from "./songMatcher";

describe("Song Matcher Heuristics", () => {
  describe("normalizeTitle", () => {
    it("converts to lowercase and trims whitespace", () => {
      expect(normalizeTitle("  Hello World  ")).toBe("hello world");
    });

    it("removes diacritics / accents", () => {
      expect(normalizeTitle("Canción del Adiós")).toBe("cancion del adios");
      expect(normalizeTitle("München")).toBe("munchen");
    });

    it("removes punctuation", () => {
      expect(normalizeTitle("Hello, World! (Testing...)")).toBe("hello world testing");
      expect(normalizeTitle("A.B.C.")).toBe("abc");
    });

    it("removes common suffix keywords (Remaster, Live, Deluxe, etc.)", () => {
      // Dash suffixes
      expect(normalizeTitle("Let It Be - Remastered 2009")).toBe("let it be");
      expect(normalizeTitle("Creep - Live At Wembley")).toBe("creep");
      expect(normalizeTitle("Purple Rain - Deluxe Edition")).toBe("purple rain");
      expect(normalizeTitle("Acoustic - Bonus Track")).toBe("acoustic"); // "bonus track" removed, "acoustic" is start word
      
      // Parenthesis and bracket suffixes
      expect(normalizeTitle("Under Pressure (2011 Remaster)")).toBe("under pressure");
      expect(normalizeTitle("Radiohead (Live)")).toBe("radiohead");
      expect(normalizeTitle("Song [Acoustic Version]")).toBe("song");
      expect(normalizeTitle("Music (Radio Edit)")).toBe("music");
      expect(normalizeTitle("Track (Mono Mix)")).toBe("track");
      expect(normalizeTitle("Beat [Club Remix]")).toBe("beat");
      expect(normalizeTitle("Interlude (Instrumental)")).toBe("interlude");
    });

    it("normalizes multiple whitespaces", () => {
      expect(normalizeTitle("hello   world  test")).toBe("hello world test");
    });
  });

  describe("normalizeArtist", () => {
    it("normalizes artist names correctly", () => {
      expect(normalizeArtist("  Béyonce  ")).toBe("beyonce");
      expect(normalizeArtist("M83.")).toBe("m83");
    });
  });

  describe("calculateSimilarity (Dice Coefficient / Levenshtein)", () => {
    it("returns 1 for exact match", () => {
      expect(calculateSimilarity("hello", "hello")).toBe(1);
    });

    it("returns 0 for completely different strings", () => {
      expect(calculateSimilarity("abc", "xyz")).toBe(0);
    });

    it("returns high similarity for minor typos", () => {
      expect(calculateSimilarity("smells like teen spirit", "smells like ten spirit")).toBeGreaterThan(0.85);
    });
  });

  describe("isMatch", () => {
    it("returns exact match when normalized title and artists match", () => {
      const result = isMatch(
        { title: "Let It Be - Remastered 2009", artists: ["The Beatles"] },
        { title: "Let It Be", artists: ["The Beatles"] }
      );
      expect(result.matched).toBe(true);
      expect(result.type).toBe("exact");
    });

    it("returns exact match when at least one artist matches (intersection)", () => {
      const result = isMatch(
        { title: "Under Pressure", artists: ["Queen", "David Bowie"] },
        { title: "Under Pressure (Remastered)", artists: ["Queen"] }
      );
      expect(result.matched).toBe(true);
      expect(result.type).toBe("exact");
    });

    it("returns probable match for minor typos in title with matching artist", () => {
      const result = isMatch(
        { title: "Smells Like Teen Spirit", artists: ["Nirvana"] },
        { title: "Smells Like Ten Spirit", artists: ["Nirvana"] }
      );
      expect(result.matched).toBe(true);
      expect(result.type).toBe("probable");
      expect(result.score).toBeGreaterThan(0.85);
    });

    it("returns no match if artists are completely different even if titles match", () => {
      const result = isMatch(
        { title: "Creep", artists: ["Radiohead"] },
        { title: "Creep", artists: ["TLC"] }
      );
      expect(result.matched).toBe(false);
      expect(result.type).toBe("none");
    });

    it("returns probable match if title matches exactly but artists are missing from one side", () => {
      const result = isMatch(
        { title: "Comfortably Numb", artists: ["Pink Floyd"] },
        { title: "Comfortably Numb", artists: [] }
      );
      expect(result.matched).toBe(true);
      expect(result.type).toBe("probable");
      expect(result.score).toBe(1.0); // Exact title similarity, but artist is missing
    });
  });
});
