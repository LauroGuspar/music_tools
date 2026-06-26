import { describe, expect, it } from "vitest";
import { compareTracks, compareTracksAsync } from "./compareEngine";

describe("compareEngine", () => {
  it("compares two track lists and categorizes them correctly", () => {
    const source = [
      { title: "Yesterday", artists: ["The Beatles"] },
      { title: "Creep - Live", artists: ["Radiohead"] }, // should match exactly after stripping "- Live"
      { title: "Smells Like Teen Spirit", artists: ["Nirvana"] }, // should fuzzy match
      { title: "Missing Track", artists: ["Artist A"] }, // missing
    ];

    const target = [
      { title: "Yesterday", artists: ["The Beatles"] }, // exact
      { title: "Creep", artists: ["Radiohead"] }, // exact after stripping
      { title: "Smells Like Ten Spirit", artists: ["Nirvana"] }, // probable (fuzzy match)
      { title: "Extra Track", artists: ["Artist B"] }, // extra
    ];

    const results = compareTracks(source, target);

    expect(results.exactMatches).toHaveLength(2);
    expect(results.exactMatches[0].source.title).toBe("Yesterday");
    expect(results.exactMatches[1].source.title).toBe("Creep - Live");

    expect(results.probableMatches).toHaveLength(1);
    expect(results.probableMatches[0].source.title).toBe("Smells Like Teen Spirit");
    expect(results.probableMatches[0].target.title).toBe("Smells Like Ten Spirit");

    expect(results.missing).toHaveLength(1);
    expect(results.missing[0].title).toBe("Missing Track");

    expect(results.extra).toHaveLength(1);
    expect(results.extra[0].title).toBe("Extra Track");
  });

  it("compares two track lists asynchronously with progress callbacks", async () => {
    const source = [
      { title: "Yesterday", artists: ["The Beatles"] },
      { title: "Missing Track", artists: ["Artist A"] },
    ];
    const target = [
      { title: "Yesterday", artists: ["The Beatles"] },
    ];

    const progressCalls: Array<{ processed: number; total: number }> = [];
    const results = await compareTracksAsync(source, target, (processed, total) => {
      progressCalls.push({ processed, total });
    });

    expect(results.exactMatches).toHaveLength(1);
    expect(results.missing).toHaveLength(1);
    expect(progressCalls.length).toBeGreaterThan(0);
    expect(progressCalls[0].total).toBe(2);
  });
});
