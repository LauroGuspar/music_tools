import "./App.css";

function App() {
  return (
    <main className="app-shell">
      <section className="hero-card">
        <p className="eyebrow">Spotify Tools</p>
        <h1>Exporta playlists de Spotify a JSON o TXT</h1>
        <p className="lead">
          Base inicial de la V1 con Tauri, React y TypeScript. El siguiente
          paso sera conectar autenticacion, lectura de playlists y exportacion.
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
            <strong>Spotify Auth</strong>
          </article>
        </div>
      </section>
    </main>
  );
}

export default App;
