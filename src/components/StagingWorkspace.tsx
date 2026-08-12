import { useState, useMemo, useRef, useEffect } from "react";
import type { ExportFormat } from "../features/export/saveExportedFile";
import { CloseIcon, WarningIcon } from "./Icons";
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
};

export function StagingWorkspace({
  blocks,
  onRemoveBlock,
  onClearAllBlocks,
  onSendToDownloads,
  onExportTracks,
  onDeduplicateBlocks,
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



  if (blocks.length === 0) return null;

  return (
    <section className="workspace-panel staging-workspace-panel" style={{ marginTop: "1.5rem" }}>
      {/* Encabezado Superior de la Mesa de Trabajo */}
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", flexWrap: "wrap", gap: "0.75rem", marginBottom: "1rem" }}>
        <div>
          <p className="section-label">Mesa de Trabajo Unificada</p>
          <h2 style={{ margin: "2px 0" }}>Playlist Unificada</h2>
          <p className="workspace-copy" style={{ margin: 0 }}>
            {totalSelectedCount} de {totalTracksCount} canciones seleccionadas ({blocks.length} fuente{blocks.length === 1 ? "" : "s"})
          </p>
        </div>

        <div style={{ display: "flex", gap: "0.5rem", flexWrap: "wrap" }}>
          <button
            type="button"
            className="secondary-button"
            onClick={() => setIsDetailOpen(true)}
          >
            Ver Detalle General
          </button>
          <button
            type="button"
            className="secondary-button"
            onClick={toggleGlobalAll}
          >
            {totalSelectedCount === totalTracksCount ? "Desmarcar todas" : "Marcar todas"}
          </button>
          <button
            type="button"
            className="secondary-button"
            onClick={onClearAllBlocks}
            style={{ color: "#eb5757", borderColor: "rgba(235,87,87,0.3)" }}
          >
            Vaciar Mesa de Trabajo
          </button>
        </div>
      </div>

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
                      fontWeight: "bold",
                      fontSize: "0.85rem",
                      padding: 0,
                    }}
                  >
                    {isCollapsed ? "▶" : "▼"}
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
                                  ⚠️ Duplicada
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
                          fontWeight: "bold",
                          fontSize: "0.85rem",
                          padding: 0,
                        }}
                      >
                        {isCollapsed ? "▶" : "▼"}
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
    </section>
  );
}
