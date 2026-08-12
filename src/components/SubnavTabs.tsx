type AppTab = "export" | "compare" | "utilities" | "downloads";

type SubnavTabsProps = {
  activeTab: AppTab;
  onChange: (tab: AppTab) => void;
};

const TABS: Array<{ id: AppTab; label: string }> = [
  { id: "export", label: "Explorar y Exportar" },
  { id: "compare", label: "Comparar playlist" },
  { id: "utilities", label: "Utilidades locales" },
  { id: "downloads", label: "Cola de descargas" },
];

function SubnavTabs(props: SubnavTabsProps) {
  return (
    <nav aria-label="Módulos" className="subnav-tabs">
      {TABS.map((tab) => (
        <button
          aria-pressed={props.activeTab === tab.id}
          className={props.activeTab === tab.id ? "subnav-tab is-active" : "subnav-tab"}
          key={tab.id}
          onClick={() => props.onChange(tab.id)}
          type="button"
        >
          {tab.label}
        </button>
      ))}
    </nav>
  );
}

export default SubnavTabs;
export type { AppTab };
