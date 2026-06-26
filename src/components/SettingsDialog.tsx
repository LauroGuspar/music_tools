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

  // Cargar configuraciones actualizadas al abrir el diálogo
  useEffect(() => {
    if (isOpen) {
      setSettings(loadSettings());
      dialogRef.current?.showModal();
    } else {
      dialogRef.current?.close();
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
      if (typeof selected === "string") {
        setSettings((prev) => ({
          ...prev,
          [key]: selected,
        }));
      }
    } catch (err) {
      console.error("Error al seleccionar carpeta:", err);
    }
  }

  // Guardar configuración
  function handleSave() {
    saveSettings(settings);
    if (onSettingsSaved) {
      onSettingsSaved();
    }
    onClose();
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

        <div className="settings-body" style={{ padding: "20px 24px", display: "flex", flexDirection: "column", gap: "24px" }}>
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
                    className="playlist-input read-only-input"
                    placeholder="No configurada (se usará selección manual)"
                    readOnly
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
                    className="playlist-input read-only-input"
                    placeholder="Por defecto del sistema (Downloads)"
                    readOnly
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
                    className="playlist-input read-only-input"
                    placeholder="No configurada (por implementar)"
                    readOnly
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
                  onChange={(e) => setSettings(prev => ({ ...prev, fuzzyThreshold: parseFloat(e.target.value) }))}
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
        </div>

        {/* PIE DE DIÁLOGO */}
        <div className="dialog-footer" style={{ padding: "16px 24px", display: "flex", justifyContent: "flex-end", gap: "12px", borderTop: "1px solid rgba(255, 255, 255, 0.08)", background: "rgba(7, 19, 13, 0.4)" }}>
          <button className="secondary-button" onClick={onClose} type="button">
            Cancelar
          </button>
          <button className="primary-button" onClick={handleSave} type="button">
            Guardar Cambios
          </button>
        </div>
      </div>
    </dialog>
  );
}

export default SettingsDialog;
