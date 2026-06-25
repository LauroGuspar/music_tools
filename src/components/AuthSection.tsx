import type { AuthStatus } from "../features/auth/useSpotifyAuth";

type AuthSectionProps = {
  errorMessage: string | null;
  isAuthenticated: boolean;
  isConfigured: boolean;
  onLogin: () => Promise<void>;
  onLogout: () => void;
  status: AuthStatus;
};

const STATUS_LABELS: Record<AuthStatus, string> = {
  idle: "Listo para conectar Spotify",
  authorizing: "Esperando autorizacion en el navegador",
  authenticated: "Sesion iniciada con Spotify",
  error: "No se pudo completar el login",
};

function AuthSection(props: AuthSectionProps) {
  return (
    <section className="auth-panel" aria-label="Autenticacion de Spotify">
      <div>
        <p className="section-label">Spotify Auth</p>
        <h2>Conecta tu cuenta para leer playlists privadas</h2>
      </div>

      <p className="auth-status">{STATUS_LABELS[props.status]}</p>

      {!props.isConfigured ? (
        <p className="auth-hint">
          Falta configurar <code>VITE_SPOTIFY_CLIENT_ID</code>. Puedes copiar <code>.env.example</code>
          a <code>.env</code> y poner ahi tu Client ID.
        </p>
      ) : null}

      <p className="auth-hint">
        Redirect URI esperada en Spotify Developers: <code>http://127.0.0.1:8000/callback</code>
      </p>

      {props.errorMessage ? <p className="auth-error">{props.errorMessage}</p> : null}

      <div className="auth-actions">
        <button
          className="primary-button"
          disabled={!props.isConfigured || props.status === "authorizing"}
          onClick={() => {
            void props.onLogin();
          }}
          type="button"
        >
          {props.isAuthenticated ? "Volver a conectar" : "Iniciar sesion con Spotify"}
        </button>

        <button
          className="secondary-button"
          disabled={!props.isAuthenticated && props.status !== "error"}
          onClick={props.onLogout}
          type="button"
        >
          Cerrar sesion local
        </button>
      </div>
    </section>
  );
}

export default AuthSection;
