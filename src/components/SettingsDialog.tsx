import { useState, useEffect, useRef } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { loadSettings, saveSettings, applyTheme, type AppSettings, type ThemePreset, type ThemeMode } from "../utils/settings";
import { SettingsIcon, CloseIcon, PaletteIcon, FolderIcon, KeyIcon, CpuIcon, MoonIcon, SunIcon, MonitorIcon } from "./Icons";

type SettingsDialogProps = {
  isOpen: boolean;
  onClose: () => void;
  onSettingsSaved?: () => void;
};

type SettingsTab = "appearance" | "directories" | "integrations" | "algorithm";

type ThemePresetOption = {
  id: ThemePreset;
  name: string;
  category: "dark" | "light";
  swatches: [string, string, string]; // bg, accent, card
};

const THEME_PRESETS: ThemePresetOption[] = [
  { id: "default-dark", name: "Default Dark", category: "dark", swatches: ["#07130d", "#79e5a7", "#102319"] },
  { id: "default-light", name: "Default Light", category: "light", swatches: ["#ffffff", "#059669", "#f3f4f6"] },
  { id: "catppuccin-mocha", name: "Catppuccin Mocha", category: "dark", swatches: ["#1e1e2e", "#cba6f7", "#313244"] },
  { id: "catppuccin-latte", name: "Catppuccin Latte", category: "light", swatches: ["#eff1f5", "#8839ef", "#dce0e8"] },
  { id: "dracula", name: "Dracula", category: "dark", swatches: ["#282a36", "#ff79c6", "#44475a"] },
  { id: "one-dark", name: "One Dark", category: "dark", swatches: ["#282c34", "#61afef", "#353b45"] },
  { id: "nord", name: "Nord", category: "dark", swatches: ["#2e3440", "#88c0d0", "#434c5e"] },
];

function SettingsDialog({ isOpen, onClose, onSettingsSaved }: SettingsDialogProps) {
  const dialogRef = useRef<HTMLDialogElement>(null);
  const [settings, setSettings] = useState<AppSettings>(loadSettings());
  const [activeTab, setActiveTab] = useState<SettingsTab>("appearance");
  const [saveNotice, setSaveNotice] = useState<string | null>(null);

  // Cargar configuraciones al abrir el diálogo
  useEffect(() => {
    if (isOpen) {
      const current = loadSettings();
      setSettings(current);
      applyTheme(current);
      setSaveNotice(null);
      if (dialogRef.current && !dialogRef.current.open) {
        dialogRef.current.showModal();
      }
    } else {
      if (dialogRef.current && dialogRef.current.open) {
        dialogRef.current.close();
      }
    }
  }, [isOpen]);

  const handleCancel = (e: React.SyntheticEvent) => {
    e.preventDefault();
    // Revert to saved settings on cancel
    applyTheme(loadSettings());
    onClose();
  };

  const updateSetting = <K extends keyof AppSettings>(key: K, value: AppSettings[K]) => {
    setSettings((prev) => {
      const next = { ...prev, [key]: value };
      if (key === "themePreset" || key === "themeMode") {
        applyTheme(next);
      }
      return next;
    });
  };

  async function selectDirectory(key: keyof AppSettings, title: string) {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title,
      });
      if (typeof selected === "string" && selected.trim().length > 0) {
        updateSetting(key, selected.trim());
      } else if (Array.isArray(selected) && selected.length > 0 && typeof selected[0] === "string") {
        updateSetting(key, selected[0].trim());
      }
    } catch (err) {
      console.error("Error al seleccionar carpeta:", err);
    }
  }

  function handleSave(e?: React.FormEvent) {
    if (e) {
      e.preventDefault();
    }
    saveSettings(settings);
    setSaveNotice("¡Configuración guardada correctamente!");
    if (onSettingsSaved) {
      onSettingsSaved();
    }
    setTimeout(() => {
      onClose();
    }, 350);
  }

  return (
    <dialog
      className="expanded-dialog settings-dialog"
      onCancel={handleCancel}
      onClick={(e) => {
        if (e.target === dialogRef.current) {
          applyTheme(loadSettings());
          onClose();
        }
      }}
      ref={dialogRef}
    >
      <div className="dialog-content settings-dialog-content">
        <div className="dialog-header">
          <div style={{ display: "flex", alignItems: "center", gap: "10px" }}>
            <SettingsIcon style={{ fontSize: "1.3rem", color: "var(--accent)" }} />
            <h3>Configuración</h3>
          </div>
          <button
            className="dialog-close-btn"
            onClick={() => {
              applyTheme(loadSettings());
              onClose();
            }}
            type="button"
            aria-label="Cerrar"
          >
            <CloseIcon />
          </button>
        </div>

        <form onSubmit={handleSave} className="settings-form">
          <div className="settings-layout">
            {/* BARRA LATERAL DE NAVEGACIÓN */}
            <nav aria-label="Categorías de Configuración" className="settings-sidebar">
              <button
                type="button"
                className={`settings-tab-btn ${activeTab === "appearance" ? "is-active" : ""}`}
                onClick={() => setActiveTab("appearance")}
              >
                <PaletteIcon style={{ fontSize: "1.1rem" }} /> Apariencia
              </button>
              <button
                type="button"
                className={`settings-tab-btn ${activeTab === "directories" ? "is-active" : ""}`}
                onClick={() => setActiveTab("directories")}
              >
                <FolderIcon style={{ fontSize: "1.1rem" }} /> Directorios
              </button>
              <button
                type="button"
                className={`settings-tab-btn ${activeTab === "integrations" ? "is-active" : ""}`}
                onClick={() => setActiveTab("integrations")}
              >
                <KeyIcon style={{ fontSize: "1.1rem" }} /> Integraciones
              </button>
              <button
                type="button"
                className={`settings-tab-btn ${activeTab === "algorithm" ? "is-active" : ""}`}
                onClick={() => setActiveTab("algorithm")}
              >
                <CpuIcon style={{ fontSize: "1.1rem" }} /> Algoritmo
              </button>
            </nav>

            {/* CONTENIDO DE LA PESTAÑA */}
            <div className="settings-tab-pane">
              {saveNotice ? (
                <div style={{ background: "var(--accent-bg)", border: "1px solid var(--accent)", color: "var(--accent)", padding: "10px 14px", borderRadius: "10px", fontSize: "0.88rem" }}>
                  {saveNotice}
                </div>
              ) : null}

              {/* PESTAÑA 1: APARIENCIA */}
              {activeTab === "appearance" && (
                <div style={{ display: "flex", flexDirection: "column", gap: "20px" }}>
                  <div>
                    <h4 style={{ margin: "0 0 6px 0", fontSize: "0.95rem", color: "var(--text-primary)" }}>
                      Modo de Esquema de Color
                    </h4>
                    <p style={{ margin: "0 0 10px 0", fontSize: "0.8rem", color: "var(--text-secondary)" }}>
                      Elige si prefieres un esquema de tema claro, oscuro o sincronizado con el sistema.
                    </p>

                    <div className="mode-selector">
                      {(["dark", "light", "system"] as ThemeMode[]).map((mode) => (
                        <button
                          key={mode}
                          type="button"
                          className={`mode-btn ${settings.themeMode === mode ? "is-active" : ""}`}
                          onClick={() => updateSetting("themeMode", mode)}
                        >
                          {mode === "dark" && <><MoonIcon style={{ fontSize: "1.05rem" }} /> Oscuro</>}
                          {mode === "light" && <><SunIcon style={{ fontSize: "1.05rem" }} /> Claro</>}
                          {mode === "system" && <><MonitorIcon style={{ fontSize: "1.05rem" }} /> Sistema</>}
                        </button>
                      ))}
                    </div>
                  </div>

                  <div>
                    <h4 style={{ margin: "0 0 6px 0", fontSize: "0.95rem", color: "var(--text-primary)" }}>
                      Temas de la Aplicación
                    </h4>
                    <p style={{ margin: "0 0 10px 0", fontSize: "0.8rem", color: "var(--text-secondary)" }}>
                      Selecciona un estilo visual. Las muestras reflejan el fondo, el color de acento y las tarjetas.
                    </p>

                    <div className="theme-presets-grid">
                      {THEME_PRESETS.map((preset) => {
                        const isActive = settings.themePreset === preset.id;
                        return (
                          <button
                            type="button"
                            key={preset.id}
                            className={`theme-card ${isActive ? "is-active" : ""}`}
                            onClick={() => updateSetting("themePreset", preset.id)}
                          >
                            <div className="theme-card-title">{preset.name}</div>
                            <div className="theme-card-sub">
                              {preset.category === "dark" ? "Tema Oscuro" : "Tema Claro"}
                            </div>
                            <div className="theme-swatches">
                              {preset.swatches.map((color, i) => (
                                <span
                                  key={i}
                                  className="swatch-circle"
                                  style={{ backgroundColor: color }}
                                />
                              ))}
                            </div>
                          </button>
                        );
                      })}
                    </div>
                  </div>
                </div>
              )}

              {/* PESTAÑA 2: DIRECTORIOS */}
              {activeTab === "directories" && (
                <div style={{ display: "flex", flexDirection: "column", gap: "18px" }}>
                  <h4 style={{ margin: 0, fontSize: "0.95rem", color: "var(--accent)", textTransform: "uppercase", letterSpacing: "0.05em" }}>
                    Rutas de Archivos
                  </h4>

                  <div className="settings-field">
                    <label style={{ fontSize: "0.85rem", color: "var(--text-secondary)", display: "block", marginBottom: "6px" }}>
                      Carpeta de música local
                    </label>
                    <div style={{ display: "flex", gap: "10px" }}>
                      <input
                        className="playlist-input"
                        onChange={(e) => updateSetting("defaultLocalMusicFolder", e.target.value)}
                        placeholder="Ruta local..."
                        type="text"
                        value={settings.defaultLocalMusicFolder}
                      />
                      <button
                        className="secondary-button"
                        onClick={() => selectDirectory("defaultLocalMusicFolder", "Carpeta de música local")}
                        style={{ padding: "10px 16px", fontSize: "0.85rem" }}
                        type="button"
                      >
                        Buscar
                      </button>
                    </div>
                  </div>

                  <div className="settings-field">
                    <label style={{ fontSize: "0.85rem", color: "var(--text-secondary)", display: "block", marginBottom: "6px" }}>
                      Carpeta para exportar playlists (JSON / TXT)
                    </label>
                    <div style={{ display: "flex", gap: "10px" }}>
                      <input
                        className="playlist-input"
                        onChange={(e) => updateSetting("defaultPlaylistExportFolder", e.target.value)}
                        placeholder="Ruta de exportación..."
                        type="text"
                        value={settings.defaultPlaylistExportFolder}
                      />
                      <button
                        className="secondary-button"
                        onClick={() => selectDirectory("defaultPlaylistExportFolder", "Carpeta de exportación de playlists")}
                        style={{ padding: "10px 16px", fontSize: "0.85rem" }}
                        type="button"
                      >
                        Buscar
                      </button>
                    </div>
                  </div>

                  <div className="settings-field">
                    <label style={{ fontSize: "0.85rem", color: "var(--text-secondary)", display: "block", marginBottom: "6px" }}>
                      Carpeta para descargas de audio (.mp3)
                    </label>
                    <div style={{ display: "flex", gap: "10px" }}>
                      <input
                        className="playlist-input"
                        onChange={(e) => updateSetting("defaultSongDownloadFolder", e.target.value)}
                        placeholder="Ruta de descargas MP3..."
                        type="text"
                        value={settings.defaultSongDownloadFolder}
                      />
                      <button
                        className="secondary-button"
                        onClick={() => selectDirectory("defaultSongDownloadFolder", "Carpeta para descargar canciones")}
                        style={{ padding: "10px 16px", fontSize: "0.85rem" }}
                        type="button"
                      >
                        Buscar
                      </button>
                    </div>
                  </div>
                </div>
              )}

              {/* PESTAÑA 3: INTEGRACIONES */}
              {activeTab === "integrations" && (
                <div style={{ display: "flex", flexDirection: "column", gap: "18px" }}>
                  <h4 style={{ margin: 0, fontSize: "0.95rem", color: "var(--accent)", textTransform: "uppercase", letterSpacing: "0.05em" }}>
                    Spotify & Motor de Búsqueda
                  </h4>

                  <div className="settings-field">
                    <label style={{ fontSize: "0.85rem", color: "var(--text-secondary)", display: "block", marginBottom: "6px" }}>
                      Spotify Client ID (Personalizado)
                    </label>
                    <input
                      className="playlist-input"
                      onChange={(e) => updateSetting("spotifyClientId", e.target.value)}
                      placeholder="Usar Client ID predeterminado..."
                      type="text"
                      value={settings.spotifyClientId || ""}
                    />
                    <p style={{ margin: "6px 0 0 0", fontSize: "0.76rem", color: "var(--text-muted)", lineHeight: "1.3" }}>
                      Opcional. Permite conectar tu propia clave de desarrollador en la API Web de Spotify.
                    </p>
                  </div>

                  <div className="settings-field" style={{ marginTop: "10px" }}>
                    <label style={{ display: "flex", alignItems: "center", gap: "10px", cursor: "pointer", fontSize: "0.88rem", color: "var(--text-primary)" }}>
                      <input
                        type="checkbox"
                        checked={settings.manualYoutubeSelection || false}
                        onChange={(e) => updateSetting("manualYoutubeSelection", e.target.checked)}
                        style={{ width: "18px", height: "18px", accentColor: "var(--accent)", cursor: "pointer" }}
                      />
                      <span>Elegir vídeo manualmente en YouTube antes de descargar</span>
                    </label>
                    <p style={{ margin: "6px 0 0 28px", fontSize: "0.76rem", color: "var(--text-muted)", lineHeight: "1.3" }}>
                      Muestra un diálogo de confirmación con 5 opciones de vídeo para seleccionar la versión preferida.
                    </p>
                  </div>
                </div>
              )}

              {/* PESTAÑA 4: ALGORITMO */}
              {activeTab === "algorithm" && (
                <div style={{ display: "flex", flexDirection: "column", gap: "18px" }}>
                  <h4 style={{ margin: 0, fontSize: "0.95rem", color: "var(--accent)", textTransform: "uppercase", letterSpacing: "0.05em" }}>
                    Algoritmo de Comparación Difusa
                  </h4>

                  <div className="settings-field">
                    <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: "8px" }}>
                      <label style={{ fontSize: "0.85rem", color: "var(--text-secondary)" }}>
                        Tolerancia de Coincidencia (Fuzzy Match Threshold)
                      </label>
                      <span style={{ fontSize: "0.9rem", color: "var(--accent)", fontWeight: "700" }}>
                        {Math.round(settings.fuzzyThreshold * 100)}%
                      </span>
                    </div>
                    <div style={{ display: "flex", alignItems: "center", gap: "14px" }}>
                      <input
                        max="1.0"
                        min="0.5"
                        onChange={(e) => updateSetting("fuzzyThreshold", parseFloat(e.target.value))}
                        step="0.05"
                        style={{ flex: 1, accentColor: "var(--accent)", cursor: "pointer", height: "6px" }}
                        type="range"
                        value={settings.fuzzyThreshold}
                      />
                    </div>
                    <p style={{ margin: "10px 0 0 0", fontSize: "0.78rem", color: "var(--text-muted)", lineHeight: "1.4" }}>
                      Controla la precisión requerida al comparar nombres de pistas y artistas locales contra playlists remotas para detectar coincidencias parciales.
                    </p>
                  </div>
                </div>
              )}
            </div>
          </div>

          {/* PIE DE DIÁLOGO */}
          <div className="dialog-footer">
            <button
              className="secondary-button"
              onClick={() => {
                applyTheme(loadSettings());
                onClose();
              }}
              type="button"
            >
              Cancelar
            </button>
            <button className="primary-button" type="submit">
              Guardar Cambios
            </button>
          </div>
        </form>
      </div>
    </dialog>
  );
}

export default SettingsDialog;
