import { invoke } from "@tauri-apps/api/core";
import { openUrl } from "@tauri-apps/plugin-opener";
import { useState, useEffect } from "react";
import {
  clearSpotifyAccessToken,
  clearSpotifyAuthSession,
  clearSpotifyRefreshToken,
  clearSpotifyExpiresAt,
  getSpotifyAccessToken,
  getSpotifyRefreshToken,
  getSpotifyExpiresAt,
  saveSpotifyAccessToken,
  saveSpotifyRefreshToken,
  saveSpotifyExpiresAt,
  saveSpotifyAuthClientId,
  getSpotifyAuthClientId,
  clearSpotifyAuthClientId,
  type StorageLike,
} from "./authStorage";
import {
  completeSpotifyAuthorization,
  startSpotifyAuthorization,
  waitForSpotifyAuthCallback,
  type SpotifyAuthCallback,
} from "./spotifyAuthFlow";
import { refreshSpotifyToken } from "./spotifyAuth";
import { loadSettings } from "../../utils/settings";

type AuthStatus = "idle" | "authorizing" | "authenticated" | "error";

type UseSpotifyAuthResult = {
  accessToken: string | null;
  errorMessage: string | null;
  isConfigured: boolean;
  isAuthenticated: boolean;
  login: () => Promise<void>;
  logout: () => void;
  status: AuthStatus;
  ensureValidToken: () => Promise<string | null>;
};

function getActiveClientId(): string {
  try {
    const settings = loadSettings();
    if (settings.spotifyClientId && settings.spotifyClientId.trim().length > 0) {
      return settings.spotifyClientId.trim();
    }
  } catch (err) {
    console.error("Error reading dynamic Spotify Client ID:", err);
  }
  return import.meta.env.VITE_SPOTIFY_CLIENT_ID?.trim() ?? "";
}

export function useSpotifyAuth(
  storage: StorageLike = window.localStorage,
  settingsVersion?: number
): UseSpotifyAuthResult {
  const initialToken = getSpotifyAccessToken(storage);
  const [accessToken, setAccessToken] = useState<string | null>(initialToken);
  const [status, setStatus] = useState<AuthStatus>(initialToken ? "authenticated" : "idle");
  const [errorMessage, setErrorMessage] = useState<string | null>(null);

  async function refreshSession(): Promise<string | null> {
    const refreshToken = getSpotifyRefreshToken(storage);
    if (!refreshToken) {
      logout();
      return null;
    }

    const clientId = getActiveClientId();
    try {
      const tokens = await refreshSpotifyToken(clientId, refreshToken);
      saveSpotifyAccessToken(storage, tokens.accessToken);
      if (tokens.refreshToken) {
        saveSpotifyRefreshToken(storage, tokens.refreshToken);
      }
      saveSpotifyExpiresAt(storage, tokens.expiresIn);
      saveSpotifyAuthClientId(storage, clientId);
      setAccessToken(tokens.accessToken);
      setStatus("authenticated");
      return tokens.accessToken;
    } catch (error) {
      console.error("Failed to refresh Spotify token:", error);
      logout();
      return null;
    }
  }

  async function ensureValidToken(): Promise<string | null> {
    const clientId = getActiveClientId();
    if (!clientId) {
      return null;
    }

    const token = getSpotifyAccessToken(storage);
    const refreshToken = getSpotifyRefreshToken(storage);

    if (!token && refreshToken) {
      return refreshSession();
    }

    if (token && refreshToken) {
      const expiresAt = getSpotifyExpiresAt(storage);
      // Refresh if expired or expiring within 5 minutes
      if (Date.now() > expiresAt - 5 * 60 * 1000) {
        return refreshSession();
      }
    }

    return token;
  }

  useEffect(() => {
    const sessionClientId = getSpotifyAuthClientId(storage);
    const activeClientId = getActiveClientId();
    
    // Si cambia el Client ID configurado, invalidamos la sesión anterior automáticamente
    if (sessionClientId && activeClientId && sessionClientId !== activeClientId) {
      logout();
      return;
    }

    async function checkExpiration() {
      const token = getSpotifyAccessToken(storage);
      const refreshToken = getSpotifyRefreshToken(storage);
      if (!token && refreshToken) {
        await refreshSession();
        return;
      }
      if (token && refreshToken) {
        const expiresAt = getSpotifyExpiresAt(storage);
        if (Date.now() > expiresAt - 5 * 60 * 1000) {
          await refreshSession();
        }
      }
    }
    void checkExpiration();
  }, [storage, settingsVersion]);

  async function login(): Promise<void> {
    const clientId = getActiveClientId();
    if (!clientId) {
      setStatus("error");
      setErrorMessage("Configura un Spotify Client ID en Ajustes (o VITE_SPOTIFY_CLIENT_ID) antes de iniciar sesión.");
      return;
    }

    setStatus("authorizing");
    setErrorMessage(null);

    try {
      await startSpotifyAuthorization({
        clientId,
        storage,
        createRedirectUri: () => invoke<string>("start_spotify_auth_callback"),
        openExternalUrl: (url) => openUrl(url),
      });

      const callback = await waitForSpotifyAuthCallback({
        readCallback: () => invoke<SpotifyAuthCallback | null>("take_spotify_auth_callback"),
      });

      await completeSpotifyAuthorization(callback, {
        clientId,
        storage,
        createRedirectUri: () => invoke<string>("start_spotify_auth_callback"),
        openExternalUrl: (url) => openUrl(url),
      });

      saveSpotifyAuthClientId(storage, clientId);

      const nextToken = getSpotifyAccessToken(storage);
      setAccessToken(nextToken);
      setStatus(nextToken ? "authenticated" : "idle");
    } catch (error) {
      const message = error instanceof Error ? error.message : "Spotify login failed";
      setStatus("error");
      setErrorMessage(message);
    }
  }

  function logout(): void {
    clearSpotifyAuthSession(storage);
    clearSpotifyAccessToken(storage);
    clearSpotifyRefreshToken(storage);
    clearSpotifyExpiresAt(storage);
    clearSpotifyAuthClientId(storage);
    setAccessToken(null);
    setErrorMessage(null);
    setStatus("idle");
  }

  return {
    accessToken,
    errorMessage,
    isConfigured: getActiveClientId().length > 0,
    isAuthenticated: accessToken !== null,
    login,
    logout,
    status,
    ensureValidToken,
  };
}

export type { AuthStatus, UseSpotifyAuthResult };

