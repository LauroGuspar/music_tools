const PKCE_CHARSET = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";

const SPOTIFY_AUTHORIZE_URL = "https://accounts.spotify.com/authorize";

const PLAYLIST_READ_SCOPES = [
  "playlist-read-private",
  "playlist-read-collaborative",
] as const;

type BuildSpotifyAuthorizeUrlOptions = {
  clientId: string;
  redirectUri: string;
  codeChallenge: string;
  state: string;
};

type ExchangeSpotifyCodeForTokenInput = {
  clientId: string;
  code: string;
  codeVerifier: string;
  redirectUri: string;
};

type SpotifyTokenResponse = {
  access_token?: string;
  refresh_token?: string;
  expires_in?: number;
  error?: string;
  error_description?: string;
};

export function generateCodeVerifier(length = 128): string {
  const randomValues = crypto.getRandomValues(new Uint8Array(length));

  return Array.from(randomValues, (value) => PKCE_CHARSET[value % PKCE_CHARSET.length]).join("");
}

export async function generateCodeChallenge(codeVerifier: string): Promise<string> {
  const encodedVerifier = new TextEncoder().encode(codeVerifier);
  const digest = await crypto.subtle.digest("SHA-256", encodedVerifier);

  return bytesToBase64Url(new Uint8Array(digest));
}

export function buildSpotifyAuthorizeUrl(options: BuildSpotifyAuthorizeUrlOptions): URL {
  const url = new URL(SPOTIFY_AUTHORIZE_URL);

  url.search = new URLSearchParams({
    client_id: options.clientId,
    response_type: "code",
    redirect_uri: options.redirectUri,
    scope: PLAYLIST_READ_SCOPES.join(" "),
    code_challenge_method: "S256",
    code_challenge: options.codeChallenge,
    state: options.state,
  }).toString();

  return url;
}

export async function exchangeSpotifyCodeForToken(
  input: ExchangeSpotifyCodeForTokenInput,
): Promise<{ accessToken: string; refreshToken: string | null; expiresIn: number }> {
  const response = await fetch("https://accounts.spotify.com/api/token", {
    method: "POST",
    headers: {
      "Content-Type": "application/x-www-form-urlencoded",
    },
    body: new URLSearchParams({
      client_id: input.clientId,
      grant_type: "authorization_code",
      code: input.code,
      redirect_uri: input.redirectUri,
      code_verifier: input.codeVerifier,
    }),
  });

  const payload = (await response.json()) as SpotifyTokenResponse;

  if (!response.ok || !payload.access_token) {
    throw new Error(payload.error_description ?? payload.error ?? "Failed to exchange Spotify code");
  }

  return {
    accessToken: payload.access_token,
    refreshToken: payload.refresh_token ?? null,
    expiresIn: payload.expires_in ?? 3600,
  };
}

export async function refreshSpotifyToken(
  clientId: string,
  refreshToken: string,
): Promise<{ accessToken: string; refreshToken: string | null; expiresIn: number }> {
  const response = await fetch("https://accounts.spotify.com/api/token", {
    method: "POST",
    headers: {
      "Content-Type": "application/x-www-form-urlencoded",
    },
    body: new URLSearchParams({
      grant_type: "refresh_token",
      refresh_token: refreshToken,
      client_id: clientId,
    }),
  });

  const payload = (await response.json()) as SpotifyTokenResponse;

  if (!response.ok || !payload.access_token) {
    throw new Error(payload.error_description ?? payload.error ?? "Failed to refresh Spotify token");
  }

  return {
    accessToken: payload.access_token,
    refreshToken: payload.refresh_token ?? null,
    expiresIn: payload.expires_in ?? 3600,
  };
}

function bytesToBase64Url(bytes: Uint8Array): string {
  const base64 = btoa(String.fromCharCode(...bytes));

  return base64.replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/g, "");
}

export { PLAYLIST_READ_SCOPES };
export type { BuildSpotifyAuthorizeUrlOptions, ExchangeSpotifyCodeForTokenInput };

