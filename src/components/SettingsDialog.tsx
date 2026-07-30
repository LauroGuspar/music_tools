import { useState, useEffect, useRef } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { loadSettings, saveSettings, type AppSettings } from "../utils/settings";
import { SettingsIcon, CloseIcon } from "./Icons";

type SettingsDialogProps = {
  isOpen: boolean;
  onClose: () => void;
  onSettingsSaved?: () => void;
};

function SettingsDialog({ isOpen, onClose, onSettingsSaved }: SettingsDialogProps) {
  const dialogRef = useRef<HTMLDialogElement>(null);
  const [settings, setSettings] = useState<AppSettings>(loadSettings());
  const [saveNotice, setSaveNotice] = useState<string | null>(null);

  // Cargar configuraciones actualizadas al abrir el diálogo
  useEffect(() => {
    if (isOpen) {
      setSettings(loadSettings());
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

  // Cerrar al pulsar Escape
  const handleCancel = (e: React.SyntheticEvent) => {
    e.preventDefault();
    onClose();
  };

  // Selector de carpeta genérico
  async function selectDirectory(key: keyof AppSettings, title: string) {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title,
      });
      if (typeof selected === "string" && selected.trim().length > 0) {
        setSettings((prev) => ({
          ...prev,
          [key]: selected.trim(),
        }));
      } else if (Array.isArray(selected) && selected.length > 0 && typeof selected[0] === "string") {
        setSettings((prev) => ({
          ...prev,
          [key]: selected[0].trim(),
        }));
      }
    } catch (err) {
      console.error("Error al seleccionar carpeta:", err);
    }
  }

  // Guardar configuración
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
    }, 400);
  }

  return (
    <dialog
      className="expanded-dialog settings-dialog"
      onCancel={handleCancel}
      onClick={(e) => {
        if (e.target === dialogRef.current) {
          onClose();
        }
      }}
      ref={dialogRef}
    >
      <div className="dialog-content settings-dialog-content">
        <div className="dialog-header">
          <div style={{ display: "flex", alignItems: "center", gap: "10px" }}>
            <SettingsIcon style={{ fontSize: "1.4rem", color: "#79e5a7" }} />
            <h3>Configuración de la Aplicación</h3>
          </div>
          <button className="dialog-close-btn" onClick={onClose} type="button" aria-label="Cerrar">
            <CloseIcon />
          </button>
        </div>

        <form onSubmit={handleSave} style={{ display: "flex", flexDirection: "column", flex: 1 }}>
          <div className="settings-body" style={{ padding: "20px 24px", display: "flex", flexDirection: "column", gap: "24px" }}>
            {saveNotice ? (
              <div style={{ background: "rgba(121, 229, 167, 0.15)", border: "1px solid #79e5a7", color: "#79e5a7", padding: "10px 14px", borderRadius: "8px", fontSize: "0.88rem" }}>
                {saveNotice}
              </div>
            ) : null}

            {/* SECCIÓN 1: DIRECTORIOS */}
            <div className="settings-section">
              <h4 style={{ margin: "0 0 14px 0", color: "#79e5a7", fontSize: "0.95rem", textTransform: "uppercase", letterSpacing: "0.05em" }}>
                Directorios por defecto
              </h4>
              
              <div style={{ display: "flex", flexDirection: "column", gap: "16px" }}>
                {/* Directorio de música */}
                <div className="settings-field">
                  <label style={{ fontSize: "0.85rem", color: "rgba(244, 247, 245, 0.72)", display: "block", marginBottom: "6px" }}>
                    Carpeta de música local
                  </label>
                  <div style={{ display: "flex", gap: "10px" }}>
                    <input
                      className="playlist-input"
                      onChange={(e) => setSettings((prev) => ({ ...prev, defaultLocalMusicFolder: e.target.value }))}
                      placeholder="Escribe la ruta o usa Buscar..."
                      type="text"
                      value={settings.defaultLocalMusicFolder}
                    />
                    <button
                      className="secondary-button"
                      onClick={() => selectDirectory("defaultLocalMusicFolder", "Selecciona carpeta de música local")}
                      style={{ padding: "10px 16px", fontSize: "0.85rem" }}
                      type="button"
                    >
                      Buscar
                    </button>
                  </div>
                </div>

                {/* Directorio de exportación de playlists */}
                <div className="settings-field">
                  <label style={{ fontSize: "0.85rem", color: "rgba(244, 247, 245, 0.72)", display: "block", marginBottom: "6px" }}>
                    Carpeta de descargas de playlists (JSON/TXT)
                  </label>
                  <div style={{ display: "flex", gap: "10px" }}>
                    <input
                      className="playlist-input"
                      onChange={(e) => setSettings((prev) => ({ ...prev, defaultPlaylistExportFolder: e.target.value }))}
                      placeholder="Escribe la ruta o usa Buscar..."
                      type="text"
                      value={settings.defaultPlaylistExportFolder}
                    />
                    <button
                      className="secondary-button"
                      onClick={() => selectDirectory("defaultPlaylistExportFolder", "Selecciona carpeta para exportar playlists")}
                      style={{ padding: "10px 16px", fontSize: "0.85rem" }}
                      type="button"
                    >
                      Buscar
                    </button>
                  </div>
                </div>

                {/* Directorio de descargas de canciones */}
                <div className="settings-field">
                  <label style={{ fontSize: "0.85rem", color: "rgba(244, 247, 245, 0.72)", display: "block", marginBottom: "6px" }}>
                    Carpeta de descargas de canciones (.mp3)
                  </label>
                  <div style={{ display: "flex", gap: "10px" }}>
                    <input
                      className="playlist-input"
                      onChange={(e) => setSettings((prev) => ({ ...prev, defaultSongDownloadFolder: e.target.value }))}
                      placeholder="Escribe la ruta o usa Buscar..."
                      type="text"
                      value={settings.defaultSongDownloadFolder}
                    />
                    <button
                      className="secondary-button"
                      onClick={() => selectDirectory("defaultSongDownloadFolder", "Selecciona carpeta para descargar canciones")}
                      style={{ padding: "10px 16px", fontSize: "0.85rem" }}
                      type="button"
                    >
                      Buscar
                    </button>
                  </div>
                </div>
              </div>
            </div>

            {/* SECCIÓN 2: AJUSTES DE COMPARACIÓN */}
            <div className="settings-section">
              <h4 style={{ margin: "0 0 14px 0", color: "#79e5a7", fontSize: "0.95rem", textTransform: "uppercase", letterSpacing: "0.05em" }}>
                Algoritmo de Comparación
              </h4>

              <div className="settings-field">
                <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: "8px" }}>
                  <label style={{ fontSize: "0.85rem", color: "rgba(244, 247, 245, 0.72)" }}>
                    Precisión de coincidencia difusa (Fuzzy Match Threshold)
                  </label>
                  <span style={{ fontSize: "0.9rem", color: "#79e5a7", fontWeight: "700" }}>
                    {Math.round(settings.fuzzyThreshold * 100)}%
                  </span>
                </div>
                <div style={{ display: "flex", alignItems: "center", gap: "14px" }}>
                  <input
                    max="1.0"
                    min="0.5"
                    onChange={(e) => setSettings((prev) => ({ ...prev, fuzzyThreshold: parseFloat(e.target.value) }))}
                    step="0.05"
                    style={{ flex: 1, accentColor: "#79e5a7", cursor: "pointer", height: "6px" }}
                    type="range"
                    value={settings.fuzzyThreshold}
                  />
                </div>
                <p style={{ margin: "6px 0 0 0", fontSize: "0.76rem", color: "rgba(244, 247, 245, 0.45)", lineHeight: "1.3" }}>
                  Ajusta qué tan similares deben ser los nombres de las canciones para considerarse coincidencias probables (por ejemplo, omitiendo diferencias leves de ortografía).
                </p>
              </div>
            </div>

            {/* SECCIÓN 3: API DE SPOTIFY */}
            <div className="settings-section">
              <h4 style={{ margin: "0 0 14px 0", color: "#79e5a7", fontSize: "0.95rem", textTransform: "uppercase", letterSpacing: "0.05em" }}>
                API de Spotify
              </h4>

              <div className="settings-field">
                <label style={{ fontSize: "0.85rem", color: "rgba(244, 247, 245, 0.72)", display: "block", marginBottom: "6px" }}>
                  Spotify Client ID (Personalizado)
                </label>
                <input
                  className="playlist-input"
                  onChange={(e) => setSettings((prev) => ({ ...prev, spotifyClientId: e.target.value }))}
                  placeholder="Usar Client ID por defecto..."
                  type="text"
                  value={settings.spotifyClientId || ""}
                />
                <p style={{ margin: "6px 0 0 0", fontSize: "0.76rem", color: "rgba(244, 247, 245, 0.45)", lineHeight: "1.3" }}>
                  Opcional. Si se deja en blanco, la aplicación usará el Client ID predeterminado de la compilación. Útil si ejecutas tu propia aplicación en el dashboard de Spotify Developers.
                </p>
              </div>
            </div>

            {/* SECCIÓN 4: MOTOR DE DESCARGA */}
            <div className="settings-section">
              <h4 style={{ margin: "0 0 14px 0", color: "#79e5a7", fontSize: "0.95rem", textTransform: "uppercase", letterSpacing: "0.05em" }}>
                Motor de Descarga (YouTube)
              </h4>

              <div className="settings-field">
                <label style={{ display: "flex", alignItems: "center", gap: "10px", cursor: "pointer", fontSize: "0.88rem", color: "rgba(244, 247, 245, 0.88)" }}>
                  <input
                    type="checkbox"
                    checked={settings.manualYoutubeSelection || false}
                    onChange={(e) => setSettings((prev) => ({ ...prev, manualYoutubeSelection: e.target.checked }))}
                    style={{ width: "18px", height: "18px", accentColor: "#79e5a7", cursor: "pointer" }}
                  />
                  <span>Elegir video manualmente antes de descargar (Selección Visual)</span>
                </label>
                <p style={{ margin: "6px 0 0 28px", fontSize: "0.76rem", color: "rgba(244, 247, 245, 0.45)", lineHeight: "1.3" }}>
                  Si está activado, antes de descargar una canción se mostrará un panel visual con los 5 mejores resultados de YouTube para que elijas la versión de estudio o tu preferida.
                </p>
              </div>
            </div>
          </div>

          {/* PIE DE DIÁLOGO */}
          <div className="dialog-footer" style={{ padding: "16px 24px", display: "flex", justifyContent: "flex-end", gap: "12px", borderTop: "1px solid rgba(255, 255, 255, 0.08)", background: "rgba(7, 19, 13, 0.4)" }}>
            <button className="secondary-button" onClick={onClose} type="button">
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
