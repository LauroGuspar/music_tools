type PlaylistInputSectionProps = {
  disabled: boolean;
  inputValue: string;
  isLoading: boolean;
  onChange: (value: string) => void;
  onSubmit: () => Promise<void>;
};

function PlaylistInputSection(props: PlaylistInputSectionProps) {
  return (
    <section className="workspace-panel" aria-label="Cargar playlist">
      <div>
        <p className="section-label">Playlist</p>
        <h2>Pega la URL o el ID de tu playlist</h2>
      </div>

      <div className="playlist-input-row">
        <input
          className="playlist-input"
          disabled={props.disabled || props.isLoading}
          onChange={(event) => props.onChange(event.currentTarget.value)}
          placeholder="https://open.spotify.com/playlist/... o 37i9dQZF..."
          type="text"
          value={props.inputValue}
        />
        <button
          className="primary-button"
          disabled={props.disabled || props.isLoading || props.inputValue.trim().length === 0}
          onClick={() => {
            void props.onSubmit();
          }}
          type="button"
        >
          {props.isLoading ? "Cargando..." : "Cargar playlist"}
        </button>
      </div>
    </section>
  );
}

export default PlaylistInputSection;
