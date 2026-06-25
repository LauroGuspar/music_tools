import { describe, expect, it } from "vitest";
import { exportJson } from "./exportJson";
import { exportTxt } from "./exportTxt";

const tracks = [
  { title: "One More Time", artists: ["Daft Punk"] },
  { title: "Get Lucky", artists: ["Daft Punk", "Pharrell Williams"] },
];

describe("exportJson", () => {
  it("serializes export tracks as pretty JSON", () => {
    expect(exportJson(tracks)).toBe(`[
  {
    "title": "One More Time",
    "artists": [
      "Daft Punk"
    ]
  },
  {
    "title": "Get Lucky",
    "artists": [
      "Daft Punk",
      "Pharrell Williams"
    ]
  }
]`);
  });
});

describe("exportTxt", () => {
  it("renders one line per track with joined artists", () => {
    expect(exportTxt(tracks)).toBe(
      "One More Time - Daft Punk\nGet Lucky - Daft Punk, Pharrell Williams",
    );
  });
});
