import { describe, expect, it } from "vitest";
import {
  buildSpotifyAuthorizeUrl,
  generateCodeChallenge,
  generateCodeVerifier,
  PLAYLIST_READ_SCOPES,
} from "./spotifyAuth";
import {
  clearSpotifyAccessToken,
  createMemoryStorage,
  getSpotifyAccessToken,
  saveSpotifyAccessToken,
  saveSpotifyAuthSession,
  loadSpotifyAuthSession,
  clearSpotifyAuthSession,
} from "./authStorage";

describe("spotifyAuth", () => {
  it("generates a verifier with requested length", () => {
    const verifier = generateCodeVerifier(64);

    expect(verifier).toHaveLength(64);
    expect(verifier).toMatch(/^[A-Za-z0-9]+$/);
  });

  it("creates the expected PKCE code challenge", async () => {
    await expect(generateCodeChallenge("dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk")).resolves.toBe(
      "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM",
    );
  });

  it("builds a spotify authorization url with PKCE and playlist scopes", () => {
    const url = buildSpotifyAuthorizeUrl({
      clientId: "client-123",
      redirectUri: "http://127.0.0.1:8000/callback",
      codeChallenge: "challenge-abc",
      state: "state-xyz",
    });

    expect(url.origin + url.pathname).toBe("https://accounts.spotify.com/authorize");
    expect(url.searchParams.get("client_id")).toBe("client-123");
    expect(url.searchParams.get("response_type")).toBe("code");
    expect(url.searchParams.get("redirect_uri")).toBe("http://127.0.0.1:8000/callback");
    expect(url.searchParams.get("code_challenge_method")).toBe("S256");
    expect(url.searchParams.get("code_challenge")).toBe("challenge-abc");
    expect(url.searchParams.get("state")).toBe("state-xyz");
    expect(url.searchParams.get("scope")).toBe(PLAYLIST_READ_SCOPES.join(" "));
  });
});

describe("authStorage", () => {
  it("stores and retrieves the spotify auth session", () => {
    const storage = createMemoryStorage();

    saveSpotifyAuthSession(storage, {
      codeVerifier: "verifier-123",
      state: "state-123",
      redirectUri: "http://127.0.0.1:8000/callback",
    });

    expect(loadSpotifyAuthSession(storage)).toEqual({
      codeVerifier: "verifier-123",
      state: "state-123",
      redirectUri: "http://127.0.0.1:8000/callback",
    });

    clearSpotifyAuthSession(storage);
    expect(loadSpotifyAuthSession(storage)).toBeNull();
  });

  it("stores and clears the spotify access token", () => {
    const storage = createMemoryStorage();

    saveSpotifyAccessToken(storage, "token-123");
    expect(getSpotifyAccessToken(storage)).toBe("token-123");

    clearSpotifyAccessToken(storage);
    expect(getSpotifyAccessToken(storage)).toBeNull();
  });
});
