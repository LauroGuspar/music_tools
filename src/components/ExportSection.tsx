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
        <label className="format-field">
          <span>Formato</span>
          <select
            className="format-select"
            disabled={props.disabled || props.isExporting}
            onChange={(event) => props.onChangeFormat(event.currentTarget.value as ExportFormat)}
            value={props.format}
          >
            <option value="json">JSON</option>
            <option value="txt">TXT</option>
          </select>
        </label>

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
