import type { ExportFormat } from "../features/export/saveExportedFile";

type ExportSectionProps = {
  disabled: boolean;
  format: ExportFormat;
  isExporting: boolean;
  onChangeFormat: (format: ExportFormat) => void;
  onExport: () => Promise<void>;
};

function ExportSection(props: ExportSectionProps) {
  return (
    <section className="workspace-panel" aria-label="Exportar playlist">
      <div>
        <p className="section-label">Exportacion</p>
        <h2>Guarda la playlist en JSON o TXT</h2>
      </div>

      <div className="export-row">
        <div className="format-field">
          <span>Formato</span>
          <div className="segmented-control" role="tablist" aria-label="Formato de exportación">
            {(["json", "txt"] as ExportFormat[]).map((format) => (
              <button
                aria-selected={props.format === format}
                className={props.format === format ? "segment-button is-active" : "segment-button"}
                disabled={props.disabled || props.isExporting}
                key={format}
                onClick={() => props.onChangeFormat(format)}
                role="tab"
                type="button"
              >
                {format.toUpperCase()}
              </button>
            ))}
          </div>
        </div>

        <button
          className="primary-button"
          disabled={props.disabled || props.isExporting}
          onClick={() => {
            void props.onExport();
          }}
          type="button"
        >
          {props.isExporting ? "Guardando..." : "Exportar archivo"}
        </button>
      </div>
    </section>
  );
}

export default ExportSection;
