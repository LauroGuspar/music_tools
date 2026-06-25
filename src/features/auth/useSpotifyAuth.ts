import { invoke } from "@tauri-apps/api/core";
import { openUrl } from "@tauri-apps/plugin-opener";
import { useState } from "react";
import {
  clearSpotifyAccessToken,
  clearSpotifyAuthSession,
  clearSpotifyRefreshToken,
  getSpotifyAccessToken,
  type StorageLike,
} from "./authStorage";
import {
  completeSpotifyAuthorization,
  startSpotifyAuthorization,
  waitForSpotifyAuthCallback,
  type SpotifyAuthCallback,
} from "./spotifyAuthFlow";

type AuthStatus = "idle" | "authorizing" | "authenticated" | "error";

type UseSpotifyAuthResult = {
  accessToken: string | null;
  errorMessage: string | null;
  isConfigured: boolean;
  isAuthenticated: boolean;
  login: () => Promise<void>;
  logout: () => void;
  status: AuthStatus;
};

const spotifyClientId = import.meta.env.VITE_SPOTIFY_CLIENT_ID?.trim() ?? "";

export function useSpotifyAuth(storage: StorageLike = window.localStorage): UseSpotifyAuthResult {
  const initialToken = getSpotifyAccessToken(storage);
  const [accessToken, setAccessToken] = useState<string | null>(initialToken);
  const [status, setStatus] = useState<AuthStatus>(initialToken ? "authenticated" : "idle");
  const [errorMessage, setErrorMessage] = useState<string | null>(null);

  async function login(): Promise<void> {
    if (!spotifyClientId) {
      setStatus("error");
      setErrorMessage("Configura VITE_SPOTIFY_CLIENT_ID antes de iniciar sesion.");
      return;
    }

    setStatus("authorizing");
    setErrorMessage(null);

    try {
      await startSpotifyAuthorization({
        clientId: spotifyClientId,
        storage,
        createRedirectUri: () => invoke<string>("start_spotify_auth_callback"),
        openExternalUrl: (url) => openUrl(url),
      });

      const callback = await waitForSpotifyAuthCallback({
        readCallback: () => invoke<SpotifyAuthCallback | null>("take_spotify_auth_callback"),
      });

      await completeSpotifyAuthorization(callback, {
        clientId: spotifyClientId,
        storage,
        createRedirectUri: () => invoke<string>("start_spotify_auth_callback"),
        openExternalUrl: (url) => openUrl(url),
      });

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
    setAccessToken(null);
    setErrorMessage(null);
    setStatus("idle");
  }

  return {
    accessToken,
    errorMessage,
    isConfigured: spotifyClientId.length > 0,
    isAuthenticated: accessToken !== null,
    login,
    logout,
    status,
  };
}

export type { AuthStatus, UseSpotifyAuthResult };
