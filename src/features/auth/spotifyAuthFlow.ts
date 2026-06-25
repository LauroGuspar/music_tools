import {
  clearSpotifyAuthSession,
  saveSpotifyAccessToken,
  saveSpotifyAuthSession,
  saveSpotifyRefreshToken,
  type StorageLike,
  loadSpotifyAuthSession,
} from "./authStorage";
import {
  buildSpotifyAuthorizeUrl,
  exchangeSpotifyCodeForToken,
  generateCodeChallenge,
  generateCodeVerifier,
} from "./spotifyAuth";

type SpotifyAuthCallback = {
  code: string | null;
  error: string | null;
  state: string | null;
};

type SpotifyTokenResult = {
  accessToken: string;
  refreshToken: string | null;
};

type SpotifyAuthFlowDependencies = {
  clientId: string;
  storage: StorageLike;
  createRedirectUri: () => Promise<string>;
  openExternalUrl: (url: string) => Promise<void>;
  generateVerifier?: (length?: number) => string;
  generateChallenge?: (codeVerifier: string) => Promise<string>;
  createState?: () => string;
  exchangeCodeForToken?: (input: {
    clientId: string;
    code: string;
    codeVerifier: string;
    redirectUri: string;
  }) => Promise<SpotifyTokenResult>;
};

type WaitForSpotifyAuthCallbackOptions = {
  readCallback: () => Promise<SpotifyAuthCallback | null>;
  sleep?: (ms: number) => Promise<void>;
  timeoutMs?: number;
  pollIntervalMs?: number;
};

const DEFAULT_TIMEOUT_MS = 120_000;
const DEFAULT_POLL_INTERVAL_MS = 400;

export async function startSpotifyAuthorization(
  dependencies: SpotifyAuthFlowDependencies,
): Promise<void> {
  const generateVerifier = dependencies.generateVerifier ?? generateCodeVerifier;
  const generateChallenge = dependencies.generateChallenge ?? generateCodeChallenge;
  const createState = dependencies.createState ?? (() => crypto.randomUUID());

  const redirectUri = await dependencies.createRedirectUri();
  const codeVerifier = generateVerifier();
  const codeChallenge = await generateChallenge(codeVerifier);
  const state = createState();

  saveSpotifyAuthSession(dependencies.storage, {
    codeVerifier,
    redirectUri,
    state,
  });

  const authorizationUrl = buildSpotifyAuthorizeUrl({
    clientId: dependencies.clientId,
    redirectUri,
    codeChallenge,
    state,
  });

  await dependencies.openExternalUrl(authorizationUrl.toString());
}

export async function completeSpotifyAuthorization(
  callback: SpotifyAuthCallback,
  dependencies: SpotifyAuthFlowDependencies,
): Promise<void> {
  if (callback.error) {
    throw new Error(`Spotify login failed: ${callback.error}`);
  }

  if (!callback.code) {
    throw new Error("Spotify login did not return an authorization code");
  }

  const session = loadSpotifyAuthSession(dependencies.storage);

  if (!session) {
    throw new Error("Missing Spotify auth session");
  }

  if (callback.state !== session.state) {
    throw new Error("Spotify auth state mismatch");
  }

  const exchangeCodeForToken = dependencies.exchangeCodeForToken ?? exchangeSpotifyCodeForToken;
  const tokens = await exchangeCodeForToken({
    clientId: dependencies.clientId,
    code: callback.code,
    codeVerifier: session.codeVerifier,
    redirectUri: session.redirectUri,
  });

  saveSpotifyAccessToken(dependencies.storage, tokens.accessToken);

  if (tokens.refreshToken) {
    saveSpotifyRefreshToken(dependencies.storage, tokens.refreshToken);
  }

  clearSpotifyAuthSession(dependencies.storage);
}

export async function waitForSpotifyAuthCallback(
  options: WaitForSpotifyAuthCallbackOptions,
): Promise<SpotifyAuthCallback> {
  const sleep = options.sleep ?? defaultSleep;
  const timeoutMs = options.timeoutMs ?? DEFAULT_TIMEOUT_MS;
  const pollIntervalMs = options.pollIntervalMs ?? DEFAULT_POLL_INTERVAL_MS;
  const start = Date.now();

  while (Date.now() - start < timeoutMs) {
    const callback = await options.readCallback();

    if (callback) {
      return callback;
    }

    await sleep(pollIntervalMs);
  }

  throw new Error("Spotify login timed out");
}

async function defaultSleep(ms: number): Promise<void> {
  await new Promise((resolve) => window.setTimeout(resolve, ms));
}

export type { SpotifyAuthCallback, SpotifyAuthFlowDependencies, SpotifyTokenResult };
