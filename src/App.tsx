import AuthSection from "./components/AuthSection";
import { useSpotifyAuth } from "./features/auth/useSpotifyAuth";
import "./App.css";

function App() {
  const spotifyAuth = useSpotifyAuth();

  return (
    <main className="app-shell">
      <section className="hero-card">
        <p className="eyebrow">Spotify Tools</p>
        <h1>Exporta playlists de Spotify a JSON o TXT</h1>
        <p className="lead">
          La base ya soporta el inicio de sesion con Spotify en escritorio usando
          PKCE y callback local. El siguiente bloque conectara lectura de playlists
          y exportacion.
        </p>

        <div className="status-grid" aria-label="Estado inicial del proyecto">
          <article>
            <span className="status-label">Shell</span>
            <strong>Tauri v2</strong>
          </article>
          <article>
            <span className="status-label">Frontend</span>
            <strong>React + TypeScript</strong>
          </article>
          <article>
            <span className="status-label">Proximo modulo</span>
            <strong>Playlist Loader</strong>
          </article>
        </div>

        <AuthSection
          errorMessage={spotifyAuth.errorMessage}
          isAuthenticated={spotifyAuth.isAuthenticated}
          isConfigured={spotifyAuth.isConfigured}
          onLogin={spotifyAuth.login}
          onLogout={spotifyAuth.logout}
          status={spotifyAuth.status}
        />
      </section>
    </main>
  );
}

export default App;
