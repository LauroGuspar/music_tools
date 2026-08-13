import { useState } from "react";
import type { AuthStatus } from "../features/auth/useSpotifyAuth";
import { SettingsIcon, ListIcon } from "./Icons";

type TopBarProps = {
  authErrorMessage: string | null;
  isAuthenticated: boolean;
  isConfigured: boolean;
  onLogin: () => Promise<void>;
  onLogout: () => void;
  status: AuthStatus;
  onOpenSettings: () => void;
};

function TopBar(props: TopBarProps) {
  const [isMenuOpen, setIsMenuOpen] = useState(false);

  const buttonLabel = props.isAuthenticated
    ? "Conectado: Spotify"
    : props.status === "authorizing"
      ? "Conectando: Spotify"
      : "Iniciar sesión";

  return (
    <header className="top-bar">
      <div className="brand" style={{ display: "flex", alignItems: "center", gap: "10px" }}>
        <div
          className="brand-icon"
          style={{
            width: 32,
            height: 32,
            borderRadius: 10,
            background: "var(--accent-bg)",
            border: "1px solid var(--border-active)",
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            color: "var(--accent)",
          }}
        >
          <ListIcon style={{ width: 18, height: 18 }} />
        </div>
        <strong style={{ fontSize: "1.1rem", fontWeight: 800, color: "var(--text-primary)" }}>Music Tools</strong>
      </div>

      <div style={{ display: "flex", alignItems: "center", gap: "12px" }}>
        <button
          className="secondary-button settings-trigger-btn"
          onClick={props.onOpenSettings}
          style={{ padding: "8px 16px", borderRadius: "999px", display: "flex", alignItems: "center", gap: "6px", fontSize: "0.85rem" }}
          title="Configuración de la aplicación"
          type="button"
        >
          <SettingsIcon style={{ width: 16, height: 16 }} />
          <span>Configuración</span>
        </button>

        <div className="session-menu-shell">
        <button
          aria-expanded={isMenuOpen}
          className="session-menu-trigger"
          onClick={() => setIsMenuOpen((current) => !current)}
          type="button"
        >
          <span>{buttonLabel}</span>
          <span className="session-menu-caret">▾</span>
        </button>

        {isMenuOpen ? (
          <div className="session-menu-panel">
            {props.isAuthenticated ? (
              <>
                <p className="session-menu-caption">Spotify activo</p>
                <button
                  className="session-menu-item"
                  onClick={() => {
                    setIsMenuOpen(false);
                    void props.onLogin();
                  }}
                  type="button"
                >
                  Volver a conectar Spotify
                </button>
                <button
                  className="session-menu-item"
                  onClick={() => {
                    setIsMenuOpen(false);
                    props.onLogout();
                  }}
                  type="button"
                >
                  Cerrar sesión local
                </button>
              </>
            ) : (
              <>
                <p className="session-menu-caption">Proveedor disponible</p>
                <button
                  className="session-menu-item"
                  disabled={!props.isConfigured || props.status === "authorizing"}
                  onClick={() => {
                    setIsMenuOpen(false);
                    void props.onLogin();
                  }}
                  type="button"
                >
                  Spotify
                </button>
              </>
            )}

            {!props.isConfigured ? (
              <p className="session-menu-hint">
                Configura <code>VITE_SPOTIFY_CLIENT_ID</code> y registra <code>http://127.0.0.1:8000/callback</code>.
              </p>
            ) : null}

            {props.authErrorMessage ? <p className="session-menu-error">{props.authErrorMessage}</p> : null}
          </div>
        ) : null}
      </div>
      </div>
    </header>
  );
}

export default TopBar;
