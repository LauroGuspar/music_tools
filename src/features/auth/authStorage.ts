type StorageLike = Pick<Storage, "getItem" | "setItem" | "removeItem">;

type SpotifyAuthSession = {
  codeVerifier: string;
  state: string;
  redirectUri: string;
};

const AUTH_SESSION_KEY = "spotify-tools.auth.session";
const ACCESS_TOKEN_KEY = "spotify-tools.auth.access-token";
const REFRESH_TOKEN_KEY = "spotify-tools.auth.refresh-token";

export function createMemoryStorage(): StorageLike {
  const store = new Map<string, string>();

  return {
    getItem(key) {
      return store.get(key) ?? null;
    },
    setItem(key, value) {
      store.set(key, value);
    },
    removeItem(key) {
      store.delete(key);
    },
  };
}

export function saveSpotifyAuthSession(storage: StorageLike, session: SpotifyAuthSession): void {
  storage.setItem(AUTH_SESSION_KEY, JSON.stringify(session));
}

export function loadSpotifyAuthSession(storage: StorageLike): SpotifyAuthSession | null {
  const rawValue = storage.getItem(AUTH_SESSION_KEY);

  if (!rawValue) {
    return null;
  }

  return JSON.parse(rawValue) as SpotifyAuthSession;
}

export function clearSpotifyAuthSession(storage: StorageLike): void {
  storage.removeItem(AUTH_SESSION_KEY);
}

export function saveSpotifyAccessToken(storage: StorageLike, token: string): void {
  storage.setItem(ACCESS_TOKEN_KEY, token);
}

export function getSpotifyAccessToken(storage: StorageLike): string | null {
  return storage.getItem(ACCESS_TOKEN_KEY);
}

export function clearSpotifyAccessToken(storage: StorageLike): void {
  storage.removeItem(ACCESS_TOKEN_KEY);
}

export function saveSpotifyRefreshToken(storage: StorageLike, token: string): void {
  storage.setItem(REFRESH_TOKEN_KEY, token);
}

export function getSpotifyRefreshToken(storage: StorageLike): string | null {
  return storage.getItem(REFRESH_TOKEN_KEY);
}

export function clearSpotifyRefreshToken(storage: StorageLike): void {
  storage.removeItem(REFRESH_TOKEN_KEY);
}

const EXPIRES_AT_KEY = "spotify-tools.auth.expires-at";

export function saveSpotifyExpiresAt(storage: StorageLike, expiresInSeconds: number): void {
  const expiresAt = Date.now() + expiresInSeconds * 1000;
  storage.setItem(EXPIRES_AT_KEY, String(expiresAt));
}

export function getSpotifyExpiresAt(storage: StorageLike): number {
  const val = storage.getItem(EXPIRES_AT_KEY);
  return val ? Number(val) : 0;
}

export function clearSpotifyExpiresAt(storage: StorageLike): void {
  storage.removeItem(EXPIRES_AT_KEY);
}

export type { SpotifyAuthSession, StorageLike };

