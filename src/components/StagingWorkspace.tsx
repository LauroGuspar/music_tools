import { useState, useMemo, useRef, useEffect } from "react";
import type { ExportFormat } from "../features/export/saveExportedFile";
import { CloseIcon, WarningIcon, ChevronRightIcon, ChevronDownIcon } from "./Icons";
import { analyzeStagingDuplicates } from "../utils/duplicateDetector";

export interface StagingTrack {
  id: string;
  title: string;
  artistOrChannel: string;
  thumbnailUrl?: string;
  duration?: string;
  videoUrl?: string;
  albumName?: string;
  releaseDate?: string;
}

export interface StagingBlockData {
  id: string;
  title: string;
  source: "youtube" | "spotify" | "search";
  tracks: StagingTrack[];
}

type StagingWorkspaceProps = {
  blocks: StagingBlockData[];
  onRemoveBlock: (blockId: string) => void;
  onClearAllBlocks: () => void;
  onSendToDownloads: (selectedTracks: StagingTrack[], sourceName: string) => void;
  onExportTracks: (selectedTracks: StagingTrack[], format: ExportFormat, name: string) => void;
  onDeduplicateBlocks?: () => void;
  onUpdateBlocks?: (newBlocks: StagingBlockData[]) => void;
  onRemoveTrack?: (blockId: string, trackId: string) => void;
  isOpen?: boolean;
  onClose?: () => void;
};

export function StagingWorkspace({
  blocks,
  onRemoveBlock,
  onClearAllBlocks,
  onSendToDownloads,
  onExportTracks,
  onDeduplicateBlocks,
  onRemoveTrack,
  isOpen = false,
  onClose,
}: StagingWorkspaceProps) {
  const [showOnlyDuplicates, setShowOnlyDuplicates] = useState(false);

  // Analyze duplicates in current staging blocks
  const duplicateAnalysis = useMemo(() => {
    return analyzeStagingDuplicates(blocks);
  }, [blocks]);

  // State for selected track IDs across all blocks (Map blockId -> Set of trackIds)
  const [selectedMap, setSelectedMap] = useState<Record<string, Set<string>>>(() => {
    const initial: Record<string, Set<string>> = {};
    blocks.forEach((b) => {
      initial[b.id] = new Set(b.tracks.map((t) => t.id));
    });
    return initial;
  });

  // State for collapsed blocks
  const [collapsedMap, setCollapsedMap] = useState<Record<string, boolean>>({});

  // Sync selectedMap when blocks change (add new blocks with all tracks selected)
  useEffect(() => {
    setSelectedMap((prev) => {
      const next = { ...prev };
      blocks.forEach((b) => {
        if (!next[b.id]) {
          next[b.id] = new Set(b.tracks.map((t) => t.id));
        }
      });
      // Clean removed blocks
      Object.keys(next).forEach((bId) => {
        if (!blocks.some((b) => b.id === bId)) {
          delete next[bId];
        }
      });
      return next;
    });
  }, [blocks]);

  // Export format selection
  const [exportFormat, setExportFormat] = useState<ExportFormat>("json");

  // Drawer search & confirm dialog states
  const [drawerSearchQuery, setDrawerSearchQuery] = useState("");
  const [confirmModal, setConfirmModal] = useState<{
    type: "unselect_all" | "clear_all";
    title: string;
    message: string;
  } | null>(null);

  // Modal Detail State
  const [isDetailOpen, setIsDetailOpen] = useState(false);
  const [detailSearch, setDetailSearch] = useState("");
  const detailDialogRef = useRef<HTMLDialogElement>(null);

  useEffect(() => {
    const dialog = detailDialogRef.current;
    if (!dialog) return;
    if (isDetailOpen) {
      dialog.showModal();
    } else {
      dialog.close();
    }
  }, [isDetailOpen]);

  // Calculate totals
  const totalTracksCount = useMemo(() => {
    return blocks.reduce((acc, b) => acc + b.tracks.length, 0);
  }, [blocks]);

  const totalSelectedCount = useMemo(() => {
    let count = 0;
    blocks.forEach((b) => {
      const set = selectedMap[b.id];
      if (set) {
        count += set.size;
      }
    });
    return count;
  }, [blocks, selectedMap]);

  // Block master checkbox helpers
  const isBlockAllSelected = (block: StagingBlockData) => {
    const set = selectedMap[block.id];
    if (!set || block.tracks.length === 0) return false;
    return block.tracks.every((t) => set.has(t.id));
  };

  const toggleBlockAll = (block: StagingBlockData) => {
    const isAll = isBlockAllSelected(block);
    setSelectedMap((prev) => {
      const nextSet = new Set<string>();
      if (!isAll) {
        block.tracks.forEach((t) => nextSet.add(t.id));
      }
      return { ...prev, [block.id]: nextSet };
    });
  };

  const toggleTrackInBlock = (blockId: string, trackId: string) => {
    setSelectedMap((prev) => {
      const set = new Set(prev[blockId] || []);
      if (set.has(trackId)) {
        set.delete(trackId);
      } else {
        set.add(trackId);
      }
      return { ...prev, [blockId]: set };
    });
  };

  // Global select/deselect all
  const toggleGlobalAll = () => {
    const allSelected = totalSelectedCount === totalTracksCount && totalTracksCount > 0;
    setSelectedMap((_) => {
      const next: Record<string, Set<string>> = {};
      blocks.forEach((b) => {
        if (allSelected) {
          next[b.id] = new Set();
        } else {
          next[b.id] = new Set(b.tracks.map((t) => t.id));
        }
      });
      return next;
    });
  };

  const toggleCollapse = (blockId: string) => {
    setCollapsedMap((prev) => ({ ...prev, [blockId]: !prev[blockId] }));
  };

  // Helper to gather all selected StagingTracks
  const getSelectedTracks = (): StagingTrack[] => {
    const result: StagingTrack[] = [];
    blocks.forEach((b) => {
      const set = selectedMap[b.id];
      if (set) {
        b.tracks.forEach((t) => {
          if (set.has(t.id)) {
            result.push(t);
          }
        });
      }
    });
    return result;
  };

  const handleSendToDownloadsClick = () => {
    const selected = getSelectedTracks();
    if (selected.length === 0) return;
    const name = blocks.length === 1 ? blocks[0].title : `Mesa de Trabajo (${blocks.length} fuentes)`;
    onSendToDownloads(selected, name);
  };

  const handleExportClick = () => {
    const selected = getSelectedTracks();
    if (selected.length === 0) return;
    const name = blocks.length === 1 ? blocks[0].title : `Mesa_de_Trabajo_${blocks.length}_fuentes`;
    onExportTracks(selected, exportFormat, name);
  };



  if (blocks.length === 0 && !isOpen) return null;

  return (
    <>
      {/* PANEL LATERAL FLOTANTE (SIDE DRAWER DE LA MESA DE TRABAJO) */}
      <aside className={isOpen ? "drawer-panel is-open" : "drawer-panel"}>
        <div className="drawer-header">
          <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
            <CloseIcon style={{ width: 18, height: 18, color: "var(--accent)", transform: "rotate(45deg)" }} />
            <strong style={{ fontSize: "1rem" }}>Mesa de Trabajo</strong>
            <span className="badge-count" style={{ fontSize: "0.75rem" }}>
              {totalSelectedCount} de {totalTracksCount}
            </span>
          </div>
          {onClose && (
            <button
              type="button"
              className="icon-close-btn"
              onClick={onClose}
              title="Cerrar panel lateral"
              style={{ background: "transparent", border: 0, color: "var(--text-muted)", cursor: "pointer" }}
            >
              <CloseIcon style={{ width: 18, height: 18 }} />
            </button>
          )}
        </div>

        {/* CONTROLES Y BUSCADOR DEL DRAWER */}
        <div className="drawer-controls-bar">
          <div className="search-input-box">
            <input
              type="text"
              placeholder="Buscar en la mesa de trabajo..."
              value={drawerSearchQuery}
              onChange={(e) => setDrawerSearchQuery(e.target.value)}
            />
          </div>

          <div style={{ display: "flex", gap: "6px" }}>
            <button
              type="button"
              className="secondary-button"
              style={{ flex: 1, padding: "6px 10px", fontSize: "0.78rem", justifyContent: "center" }}
              onClick={() =>
                setConfirmModal({
                  type: "unselect_all",
                  title: "¿Desmarcar todas las canciones?",
                  message: "Esta acción desmarcará todas las canciones en la Mesa de Trabajo.",
                })
              }
            >
              Desmarcar Todos
            </button>
            <button
              type="button"
              className="secondary-button"
              style={{ flex: 1, padding: "6px 10px", fontSize: "0.78rem", justifyContent: "center", color: "#eb5757", borderColor: "rgba(235,87,87,0.3)" }}
              onClick={() =>
                setConfirmModal({
                  type: "clear_all",
                  title: "¿Vaciar la Mesa de Trabajo?",
                  message: "Esta acción eliminará todos los bloques de la Mesa de Trabajo.",
                })
              }
            >
              Vaciar Mesa
            </button>
          </div>

          <button
            type="button"
            className="secondary-button"
            style={{ width: "100%", justifyContent: "center", padding: "6px 10px", fontSize: "0.8rem" }}
            onClick={() => setIsDetailOpen(true)}
          >
            <span>Ver Detalle Completo (Todas las columnas)</span>
          </button>
        </div>

        {/* BLOQUES Y CANCIONES DENTRO DEL DRAWER */}
        <div className="drawer-body" style={{ padding: "16px", overflowY: "auto", flex: 1, display: "flex", flexDirection: "column", gap: "12px" }}>
          {blocks.map((block) => {
            const selectedSet = selectedMap[block.id] || new Set();
            const isCollapsed = Boolean(collapsedMap[block.id]);
            const isAllSelected = isBlockAllSelected(block);

            const q = drawerSearchQuery.toLowerCase().trim();
            const displayedTracks = q
              ? block.tracks.filter(
                  (t) => t.title.toLowerCase().includes(q) || t.artistOrChannel.toLowerCase().includes(q)
                )
              : block.tracks;

            if (q && displayedTracks.length === 0) return null;

            return (
              <div key={block.id} className="drawer-block" style={{ border: "1px solid var(--border-color)", borderRadius: "12px", background: "var(--bg-card)", overflow: "hidden" }}>
                <div className="drawer-block-header" style={{ padding: "10px 14px", background: "var(--bg-surface)", display: "flex", alignItems: "center", justifyContent: "space-between", fontWeight: 600, fontSize: "0.88rem" }}>
                  <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
                    <input
                      type="checkbox"
                      checked={isAllSelected}
                      onChange={() => toggleBlockAll(block)}
                      title="Seleccionar todo el bloque"
                    />
                    <button
                      type="button"
                      onClick={() => toggleCollapse(block.id)}
                      style={{ border: 0, background: "transparent", color: "inherit", cursor: "pointer", padding: 0, display: "flex", alignItems: "center" }}
                    >
                      {isCollapsed ? <ChevronRightIcon style={{ width: 14, height: 14 }} /> : <ChevronDownIcon style={{ width: 14, height: 14 }} />}
                    </button>
                    <span style={{ fontSize: "0.85rem" }}>{block.title}</span>
                  </div>

                  <div style={{ display: "flex", alignItems: "center", gap: "6px" }}>
                    <span style={{ fontSize: "0.75rem", color: "var(--text-muted)" }}>{block.tracks.length} items</span>
                    <button
                      type="button"
                      className="icon-close-btn"
                      title="Eliminar bloque completo"
                      onClick={() => onRemoveBlock(block.id)}
                      style={{ border: 0, background: "transparent", color: "var(--text-muted)", cursor: "pointer", padding: "2px" }}
                    >
                      <CloseIcon style={{ width: 14, height: 14 }} />
                    </button>
                  </div>
                </div>

                {!isCollapsed && (
                  <div className="drawer-block-content" style={{ padding: "8px 10px", display: "flex", flexDirection: "column", gap: "6px" }}>
                    {displayedTracks.map((track) => {
                      const isChecked = selectedSet.has(track.id);
                      return (
                        <div key={track.id} className="drawer-track-row" style={{ display: "flex", alignItems: "center", gap: "10px", padding: "6px 8px", borderRadius: "8px", fontSize: "0.84rem" }}>
                          <input
                            type="checkbox"
                            checked={isChecked}
                            onChange={() => toggleTrackInBlock(block.id, track.id)}
                          />
                          <div style={{ flex: 1, overflow: "hidden" }}>
                            <strong style={{ display: "block", textOverflow: "ellipsis", overflow: "hidden", whiteSpace: "nowrap", opacity: isChecked ? 1 : 0.5 }}>
                              {track.title}
                            </strong>
                            <span style={{ fontSize: "0.75rem", color: "var(--text-secondary)", opacity: isChecked ? 1 : 0.5 }}>
                              {track.artistOrChannel}
                            </span>
                          </div>
                          <button
                            type="button"
                            className="icon-close-btn"
                            title="Eliminar canción"
                            onClick={() => onRemoveTrack ? onRemoveTrack(block.id, track.id) : null}
                            style={{ border: 0, background: "transparent", color: "var(--text-muted)", cursor: "pointer", padding: "2px" }}
                          >
                            <CloseIcon style={{ width: 12, height: 12 }} />
                          </button>
                        </div>
                      );
                    })}
                  </div>
                )}
              </div>
            );
          })}
        </div>

        <div className="drawer-footer" style={{ padding: "14px 20px", borderTop: "1px solid var(--border-color)", background: "var(--bg-surface)", display: "flex", flexDirection: "column", gap: "10px" }}>
          {onDeduplicateBlocks && (
            <button type="button" className="secondary-button" style={{ width: "100%", justifyContent: "center" }} onClick={onDeduplicateBlocks}>
              🧹 Limpiar Duplicados
            </button>
          )}
          <button type="button" className="primary-button" style={{ width: "100%", justifyContent: "center" }} disabled={totalSelectedCount === 0} onClick={handleSendToDownloadsClick}>
            📥 Descargar Selección ({totalSelectedCount})
          </button>
        </div>
      </aside>

      {/* POPUP DE CONFIRMACIÓN DE ACCIONES */}
      {confirmModal && (
        <dialog className="expanded-dialog" open style={{ zIndex: 350 }}>
          <div className="dialog-content" style={{ width: "min(90vw, 420px)", padding: "24px" }}>
            <h3 style={{ margin: "0 0 12px 0", fontSize: "1.1rem" }}>{confirmModal.title}</h3>
            <p style={{ margin: "0 0 20px 0", fontSize: "0.88rem", color: "var(--text-secondary)" }}>
              {confirmModal.message}
            </p>
            <div style={{ display: "flex", justifyContent: "flex-end", gap: "10px" }}>
              <button type="button" className="secondary-button" onClick={() => setConfirmModal(null)}>
                Cancelar
              </button>
              <button
                type="button"
                className="primary-button"
                style={{ background: "var(--danger, #eb5757)", color: "#fff" }}
                onClick={() => {
                  if (confirmModal.type === "unselect_all") {
                    toggleGlobalAll();
                  } else if (confirmModal.type === "clear_all") {
                    onClearAllBlocks();
                  }
                  setConfirmModal(null);
                }}
              >
                Confirmar
              </button>
            </div>
          </div>
        </dialog>
      )}

      {/* Warning banner when duplicates are detected */}
      {duplicateAnalysis.duplicateCount > 0 && (
        <div
          style={{
            backgroundColor: "rgba(255, 171, 0, 0.12)",
            border: "1px solid rgba(255, 171, 0, 0.3)",
            borderRadius: "8px",
            padding: "0.75rem 1rem",
            marginBottom: "1.25rem",
            display: "flex",
            alignItems: "center",
            justifyContent: "space-between",
            flexWrap: "wrap",
            gap: "0.75rem",
          }}
        >
          <div style={{ display: "flex", alignItems: "center", gap: "0.5rem" }}>
            <WarningIcon />
            <span style={{ fontSize: "0.9rem", color: "#ffc107" }}>
              <strong>{duplicateAnalysis.duplicateCount} canciones duplicadas</strong> detectadas entre los bloques de la Mesa de Trabajo.
            </span>
          </div>
          <div style={{ display: "flex", gap: "0.75rem", alignItems: "center", flexWrap: "wrap" }}>
            <label style={{ fontSize: "0.85rem", cursor: "pointer", display: "inline-flex", alignItems: "center", gap: "4px" }}>
              <input
                type="checkbox"
                checked={showOnlyDuplicates}
                onChange={(e) => setShowOnlyDuplicates(e.target.checked)}
              />
              Ver solo duplicadas
            </label>
            {onDeduplicateBlocks && (
              <button
                type="button"
                className="secondary-button"
                onClick={onDeduplicateBlocks}
                style={{ fontSize: "0.8rem", padding: "4px 10px", borderColor: "rgba(255, 171, 0, 0.4)", color: "#ffc107" }}
              >
                Limpiar duplicados (conservar 1ª copia)
              </button>
            )}
          </div>
        </div>
      )}

      {/* Lista de Bloques (Accordion Groups) */}
      <div style={{ display: "flex", flexDirection: "column", gap: "1rem", marginBottom: "1.5rem" }}>
        {blocks.map((block) => {
          const selectedSet = selectedMap[block.id] || new Set();
          const isCollapsed = Boolean(collapsedMap[block.id]);
          const isAllSelected = isBlockAllSelected(block);
          
          const displayedTracks = showOnlyDuplicates
            ? block.tracks.filter((t) => duplicateAnalysis.duplicateTrackIds.has(t.id))
            : block.tracks;

          if (showOnlyDuplicates && displayedTracks.length === 0) {
            return null;
          }

          const hasThumbnails = displayedTracks.some((t) => Boolean(t.thumbnailUrl));
          const hasDurations = displayedTracks.some((t) => Boolean(t.duration && t.duration !== "-"));

          return (
            <div
              key={block.id}
              className="staging-block-card"
              style={{
                border: "1px solid rgba(255, 255, 255, 0.1)",
                borderRadius: "8px",
                backgroundColor: "rgba(255, 255, 255, 0.02)",
                overflow: "hidden",
              }}
            >
              {/* Header de Bloque */}
              <div
                style={{
                  display: "flex",
                  alignItems: "center",
                  justifyContent: "space-between",
                  padding: "0.75rem 1rem",
                  backgroundColor: "rgba(255, 255, 255, 0.04)",
                  borderBottom: isCollapsed ? "none" : "1px solid rgba(255, 255, 255, 0.08)",
                  gap: "0.5rem",
                }}
              >
                <div style={{ display: "flex", alignItems: "center", gap: "0.75rem", flex: 1, minWidth: 0 }}>
                  <input
                    type="checkbox"
                    checked={isAllSelected}
                    onChange={() => toggleBlockAll(block)}
                    title="Seleccionar/Desmarcar todo el bloque"
                  />
                  <button
                    type="button"
                    onClick={() => toggleCollapse(block.id)}
                    style={{
                      background: "transparent",
                      border: "none",
                      color: "inherit",
                      cursor: "pointer",
                      display: "inline-flex",
                      alignItems: "center",
                      padding: 0,
                    }}
                  >
                    {isCollapsed ? <ChevronRightIcon style={{ width: 14, height: 14 }} /> : <ChevronDownIcon style={{ width: 14, height: 14 }} />}
                  </button>
                  <div style={{ overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
                    <span
                      style={{
                        fontSize: "0.75rem",
                        fontWeight: 600,
                        padding: "2px 6px",
                        borderRadius: "4px",
                        marginRight: "8px",
                        backgroundColor: block.source === "youtube" ? "rgba(255,0,0,0.15)" : "rgba(29,185,84,0.15)",
                        color: block.source === "youtube" ? "#FF4D4D" : "#1DB954",
                      }}
                    >
                      {block.source.toUpperCase()}
                    </span>
                    <strong style={{ fontSize: "0.95rem" }}>{block.title}</strong>
                    <span style={{ fontSize: "0.8rem", opacity: 0.7, marginLeft: "8px" }}>
                      ({selectedSet.size} de {block.tracks.length} seleccionadas)
                    </span>
                  </div>
                </div>

                <button
                  type="button"
                  className="secondary-button"
                  onClick={() => onRemoveBlock(block.id)}
                  style={{ fontSize: "0.75rem", padding: "4px 8px" }}
                >
                  Eliminar bloque
                </button>
              </div>

              {/* Contenido Plegable (Tabla de canciones) */}
              {!isCollapsed && (
                <div className="table-wrapper" style={{ maxHeight: "320px", overflowY: "auto" }}>
                  <table className="preview-table">
                    <thead>
                      <tr>
                        <th style={{ width: "40px", textAlign: "center" }}>
                          <input
                            type="checkbox"
                            checked={isAllSelected}
                            onChange={() => toggleBlockAll(block)}
                          />
                        </th>
                        {hasThumbnails && <th style={{ width: "60px" }}>PORTADA</th>}
                        <th>TÍTULO</th>
                        <th>CANAL / ARTISTA</th>
                        {hasDurations && <th style={{ width: "85px", textAlign: "right" }}>DURACIÓN</th>}
                      </tr>
                    </thead>
                    <tbody>
                      {displayedTracks.map((track) => {
                        const isChecked = selectedSet.has(track.id);
                        const isDuplicate = duplicateAnalysis.duplicateTrackIds.has(track.id);
                        const dupInfos = duplicateAnalysis.duplicateMap.get(track.id);
                        const dupTooltip = dupInfos
                          ? `Duplicada. Coincide con:\n${dupInfos.map((d) => `• ${d.trackTitle} (${d.blockTitle})`).join("\n")}`
                          : "Duplicada";

                        return (
                          <tr
                            key={track.id}
                            onClick={() => toggleTrackInBlock(block.id, track.id)}
                            style={{ cursor: "pointer", backgroundColor: isChecked ? "rgba(255,255,255,0.03)" : undefined }}
                          >
                            <td style={{ textAlign: "center" }} onClick={(e) => e.stopPropagation()}>
                              <input
                                type="checkbox"
                                checked={isChecked}
                                onChange={() => toggleTrackInBlock(block.id, track.id)}
                              />
                            </td>
                            {hasThumbnails && (
                              <td>
                                {track.thumbnailUrl ? (
                                  <img
                                    src={track.thumbnailUrl}
                                    alt={track.title}
                                    style={{ width: 44, height: 32, objectFit: "cover", borderRadius: 4 }}
                                  />
                                ) : (
                                  <div style={{ width: 44, height: 32, background: "rgba(255,255,255,0.05)", borderRadius: 4 }} />
                                )}
                              </td>
                            )}
                            <td>
                              <strong style={{ opacity: isChecked ? 1 : 0.5 }}>{track.title}</strong>
                              {isDuplicate && (
                                <span
                                  title={dupTooltip}
                                  style={{
                                    fontSize: "0.7rem",
                                    backgroundColor: "rgba(255, 171, 0, 0.2)",
                                    color: "#ffc107",
                                    padding: "2px 6px",
                                    borderRadius: "4px",
                                    marginLeft: "8px",
                                    fontWeight: "normal",
                                    display: "inline-flex",
                                    alignItems: "center",
                                    gap: "3px",
                                  }}
                                >
                                  <WarningIcon style={{ width: 12, height: 12 }} /> Duplicada
                                </span>
                              )}
                            </td>
                            <td style={{ opacity: isChecked ? 1 : 0.5 }}>{track.artistOrChannel}</td>
                            {hasDurations && (
                              <td style={{ textAlign: "right", opacity: isChecked ? 1 : 0.5 }}>
                                {track.duration || "-"}
                              </td>
                            )}
                          </tr>
                        );
                      })}
                    </tbody>
                  </table>
                </div>
              )}
            </div>
          );
        })}
      </div>

      {/* Panel Unificado de Acciones */}
      <div
        style={{
          display: "flex",
          justifyContent: "space-between",
          alignItems: "center",
          flexWrap: "wrap",
          gap: "1rem",
          padding: "1rem",
          backgroundColor: "rgba(255, 255, 255, 0.03)",
          borderRadius: "8px",
          border: "1px solid rgba(255, 255, 255, 0.08)",
        }}
      >
        <div style={{ display: "flex", alignItems: "center", gap: "0.75rem" }}>
          <span style={{ fontSize: "0.85rem", fontWeight: 600, opacity: 0.8 }}>Formato:</span>
          <label style={{ display: "inline-flex", alignItems: "center", gap: "4px", fontSize: "0.85rem", cursor: "pointer" }}>
            <input
              type="radio"
              name="exportFormatStaging"
              value="json"
              checked={exportFormat === "json"}
              onChange={() => setExportFormat("json")}
            />
            JSON
          </label>
          <label style={{ display: "inline-flex", alignItems: "center", gap: "4px", fontSize: "0.85rem", cursor: "pointer" }}>
            <input
              type="radio"
              name="exportFormatStaging"
              value="txt"
              checked={exportFormat === "txt"}
              onChange={() => setExportFormat("txt")}
            />
            TXT
          </label>
        </div>

        <div style={{ display: "flex", gap: "0.5rem", flexWrap: "wrap" }}>
          <button
            type="button"
            className="secondary-button"
            disabled={totalSelectedCount === 0}
            onClick={handleExportClick}
          >
            Exportar {totalSelectedCount} seleccionadas
          </button>
          <button
            type="button"
            className="primary-button"
            disabled={totalSelectedCount === 0}
            onClick={handleSendToDownloadsClick}
          >
            Enviar {totalSelectedCount} seleccionadas a la cola de descargas
          </button>
        </div>
      </div>

      {/* Modal Dialog "Ver Detalle General" */}
      <dialog
        ref={detailDialogRef}
        className="expanded-dialog"
        onClose={() => setIsDetailOpen(false)}
        onClick={(e) => {
          if (e.target === detailDialogRef.current) {
            setIsDetailOpen(false);
          }
        }}
      >
        <div className="dialog-content">
          <div className="dialog-header">
            <div>
              <span className="section-label">Detalle General</span>
              <h3>Mesa de Trabajo ({blocks.length} fuentes)</h3>
              <p className="workspace-copy" style={{ margin: 0 }}>
                {totalSelectedCount} de {totalTracksCount} canciones seleccionadas
              </p>
            </div>
            <button
              type="button"
              className="dialog-close-btn"
              onClick={() => setIsDetailOpen(false)}
              aria-label="Cerrar"
            >
              <CloseIcon />
            </button>
          </div>

          <div className="dialog-search-row">
            <input
              type="search"
              placeholder="Buscar canción, artista o playlist en toda la mesa..."
              value={detailSearch}
              onChange={(e) => setDetailSearch(e.target.value)}
              className="playlist-input dialog-search-input"
              style={{ flex: 1 }}
              autoFocus
            />
            <button
              type="button"
              className="secondary-button"
              onClick={toggleGlobalAll}
            >
              {totalSelectedCount === totalTracksCount ? "Desmarcar todas" : "Marcar todas"}
            </button>
          </div>

          <div className="dialog-table-shell" style={{ maxHeight: "60vh", overflowY: "auto", display: "flex", flexDirection: "column", gap: "1rem", paddingRight: "4px" }}>
            {blocks.map((block) => {
              const q = detailSearch.toLowerCase().trim();
              const filteredTracksInBlock = q
                ? block.tracks.filter(
                    (t) =>
                      t.title.toLowerCase().includes(q) ||
                      t.artistOrChannel.toLowerCase().includes(q) ||
                      block.title.toLowerCase().includes(q)
                  )
                : block.tracks;

              if (filteredTracksInBlock.length === 0) return null;

              const selectedSet = selectedMap[block.id] || new Set();
              const isCollapsed = Boolean(collapsedMap[block.id]);
              const isAllSelected = isBlockAllSelected(block);
              const hasThumbnails = filteredTracksInBlock.some((t) => Boolean(t.thumbnailUrl));
              const hasDurations = filteredTracksInBlock.some((t) => Boolean(t.duration && t.duration !== "-"));

              return (
                <div
                  key={`modal-${block.id}`}
                  className="staging-block-card"
                  style={{
                    border: "1px solid rgba(255, 255, 255, 0.1)",
                    borderRadius: "8px",
                    backgroundColor: "rgba(255, 255, 255, 0.02)",
                    overflow: "hidden",
                  }}
                >
                  <div
                    style={{
                      display: "flex",
                      alignItems: "center",
                      justifyContent: "space-between",
                      padding: "0.75rem 1rem",
                      backgroundColor: "rgba(255, 255, 255, 0.04)",
                      borderBottom: isCollapsed ? "none" : "1px solid rgba(255, 255, 255, 0.08)",
                      gap: "0.5rem",
                    }}
                  >
                    <div style={{ display: "flex", alignItems: "center", gap: "0.75rem", flex: 1, minWidth: 0 }}>
                      <input
                        type="checkbox"
                        checked={isAllSelected}
                        onChange={() => toggleBlockAll(block)}
                        title="Seleccionar/Desmarcar todo el bloque"
                      />
                      <button
                        type="button"
                        onClick={() => toggleCollapse(block.id)}
                        style={{
                          background: "transparent",
                          border: "none",
                          color: "inherit",
                          cursor: "pointer",
                          display: "inline-flex",
                          alignItems: "center",
                          padding: 0,
                        }}
                      >
                        {isCollapsed ? <ChevronRightIcon style={{ width: 14, height: 14 }} /> : <ChevronDownIcon style={{ width: 14, height: 14 }} />}
                      </button>
                      <div style={{ overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
                        <span
                          style={{
                            fontSize: "0.75rem",
                            fontWeight: 600,
                            padding: "2px 6px",
                            borderRadius: "4px",
                            marginRight: "8px",
                            backgroundColor: block.source === "youtube" ? "rgba(255,0,0,0.15)" : "rgba(29,185,84,0.15)",
                            color: block.source === "youtube" ? "#FF4D4D" : "#1DB954",
                          }}
                        >
                          {block.source.toUpperCase()}
                        </span>
                        <strong style={{ fontSize: "0.95rem" }}>{block.title}</strong>
                        <span style={{ fontSize: "0.8rem", opacity: 0.7, marginLeft: "8px" }}>
                          ({selectedSet.size} de {block.tracks.length} seleccionadas)
                        </span>
                      </div>
                    </div>

                    <button
                      type="button"
                      className="secondary-button"
                      onClick={() => onRemoveBlock(block.id)}
                      style={{ fontSize: "0.75rem", padding: "4px 8px" }}
                    >
                      Eliminar bloque
                    </button>
                  </div>

                  {!isCollapsed && (
                    <div className="table-wrapper" style={{ maxHeight: "280px", overflowY: "auto" }}>
                      <table className="preview-table">
                        <thead>
                          <tr>
                            <th style={{ width: "40px", textAlign: "center" }}>
                              <input
                                type="checkbox"
                                checked={isAllSelected}
                                onChange={() => toggleBlockAll(block)}
                              />
                            </th>
                            {hasThumbnails && <th style={{ width: "60px" }}>PORTADA</th>}
                            <th>TÍTULO</th>
                            <th>CANAL / ARTISTA</th>
                            {hasDurations && <th style={{ width: "85px", textAlign: "right" }}>DURACIÓN</th>}
                          </tr>
                        </thead>
                        <tbody>
                          {filteredTracksInBlock.map((track) => {
                            const isChecked = selectedSet.has(track.id);
                            return (
                              <tr
                                key={track.id}
                                onClick={() => toggleTrackInBlock(block.id, track.id)}
                                style={{ cursor: "pointer", backgroundColor: isChecked ? "rgba(255,255,255,0.03)" : undefined }}
                              >
                                <td style={{ textAlign: "center" }} onClick={(e) => e.stopPropagation()}>
                                  <input
                                    type="checkbox"
                                    checked={isChecked}
                                    onChange={() => toggleTrackInBlock(block.id, track.id)}
                                  />
                                </td>
                                {hasThumbnails && (
                                  <td>
                                    {track.thumbnailUrl ? (
                                      <img
                                        src={track.thumbnailUrl}
                                        alt={track.title}
                                        style={{ width: 44, height: 32, objectFit: "cover", borderRadius: 4 }}
                                      />
                                    ) : (
                                      <div style={{ width: 44, height: 32, background: "rgba(255,255,255,0.05)", borderRadius: 4 }} />
                                    )}
                                  </td>
                                )}
                                <td>
                                  <strong style={{ opacity: isChecked ? 1 : 0.5 }}>{track.title}</strong>
                                </td>
                                <td style={{ opacity: isChecked ? 1 : 0.5 }}>{track.artistOrChannel}</td>
                                {hasDurations && (
                                  <td style={{ textAlign: "right", opacity: isChecked ? 1 : 0.5 }}>
                                    {track.duration || "-"}
                                  </td>
                                )}
                              </tr>
                            );
                          })}
                        </tbody>
                      </table>
                    </div>
                  )}
                </div>
              );
            })}
          </div>

          <div style={{ marginTop: "1rem", display: "flex", justifyContent: "flex-end", gap: "0.5rem" }}>
            <button type="button" className="secondary-button" onClick={() => setIsDetailOpen(false)}>
              Cerrar
            </button>
            <button
              type="button"
              className="primary-button"
              disabled={totalSelectedCount === 0}
              onClick={() => {
                setIsDetailOpen(false);
                handleSendToDownloadsClick();
              }}
            >
              Enviar {totalSelectedCount} seleccionadas a descargas
            </button>
          </div>
        </div>
      </dialog>
    </>
  );
}
