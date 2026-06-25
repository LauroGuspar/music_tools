import { describe, expect, it, vi } from "vitest";
import {
  type SpotifyAuthFlowDependencies,
  completeSpotifyAuthorization,
  startSpotifyAuthorization,
  waitForSpotifyAuthCallback,
} from "./spotifyAuthFlow";
import {
  createMemoryStorage,
  getSpotifyAccessToken,
  getSpotifyRefreshToken,
  loadSpotifyAuthSession,
} from "./authStorage";

function createDependencies(overrides: Partial<SpotifyAuthFlowDependencies> = {}): SpotifyAuthFlowDependencies {
  return {
    clientId: "client-123",
    storage: createMemoryStorage(),
    createRedirectUri: vi.fn().mockResolvedValue("http://127.0.0.1:8123/callback"),
    openExternalUrl: vi.fn().mockResolvedValue(undefined),
    generateVerifier: vi.fn().mockReturnValue("verifier-123"),
    generateChallenge: vi.fn().mockResolvedValue("challenge-123"),
    createState: vi.fn().mockReturnValue("state-123"),
    exchangeCodeForToken: vi.fn().mockResolvedValue({
      accessToken: "access-123",
      refreshToken: "refresh-123",
    }),
    ...overrides,
  };
}

describe("spotifyAuthFlow", () => {
  it("starts authorization by storing session and opening Spotify auth url", async () => {
    const dependencies = createDependencies();

    await startSpotifyAuthorization(dependencies);

    expect(loadSpotifyAuthSession(dependencies.storage)).toEqual({
      codeVerifier: "verifier-123",
      redirectUri: "http://127.0.0.1:8123/callback",
      state: "state-123",
    });
    expect(dependencies.openExternalUrl).toHaveBeenCalledTimes(1);
    const openedUrl = new URL((dependencies.openExternalUrl as ReturnType<typeof vi.fn>).mock.calls[0][0]);
    expect(openedUrl.searchParams.get("client_id")).toBe("client-123");
    expect(openedUrl.searchParams.get("code_challenge")).toBe("challenge-123");
    expect(openedUrl.searchParams.get("state")).toBe("state-123");
  });

  it("completes authorization by validating state and storing tokens", async () => {
    const dependencies = createDependencies();

    await startSpotifyAuthorization(dependencies);

    await completeSpotifyAuthorization(
      { code: "code-123", error: null, state: "state-123" },
      dependencies,
    );

    expect(dependencies.exchangeCodeForToken).toHaveBeenCalledWith({
      clientId: "client-123",
      code: "code-123",
      codeVerifier: "verifier-123",
      redirectUri: "http://127.0.0.1:8123/callback",
    });
    expect(getSpotifyAccessToken(dependencies.storage)).toBe("access-123");
    expect(getSpotifyRefreshToken(dependencies.storage)).toBe("refresh-123");
    expect(loadSpotifyAuthSession(dependencies.storage)).toBeNull();
  });

  it("rejects callback state mismatches", async () => {
    const dependencies = createDependencies();

    await startSpotifyAuthorization(dependencies);

    await expect(
      completeSpotifyAuthorization({ code: "code-123", error: null, state: "wrong-state" }, dependencies),
    ).rejects.toThrow("Spotify auth state mismatch");
  });

  it("waits until the callback reader returns a result", async () => {
    const readCallback = vi
      .fn()
      .mockResolvedValueOnce(null)
      .mockResolvedValueOnce(null)
      .mockResolvedValueOnce({ code: "code-123", error: null, state: "state-123" });
    const sleep = vi.fn().mockResolvedValue(undefined);

    await expect(
      waitForSpotifyAuthCallback({
        readCallback,
        sleep,
        timeoutMs: 2_000,
        pollIntervalMs: 50,
      }),
    ).resolves.toEqual({ code: "code-123", error: null, state: "state-123" });

    expect(readCallback).toHaveBeenCalledTimes(3);
    expect(sleep).toHaveBeenCalledTimes(2);
  });

  it("throws when the callback polling times out", async () => {
    const readCallback = vi.fn().mockResolvedValue(null);
    const sleep = vi.fn().mockResolvedValue(undefined);

    await expect(
      waitForSpotifyAuthCallback({
        readCallback,
        sleep,
        timeoutMs: 120,
        pollIntervalMs: 60,
      }),
    ).rejects.toThrow("Spotify login timed out");
  });
});
