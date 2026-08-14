import { SearchIcon, CompareIcon, FolderIcon, DownloadIcon } from "./Icons";

type AppTab = "export" | "compare" | "utilities" | "downloads";

type SubnavTabsProps = {
  activeTab: AppTab;
  onChange: (tab: AppTab) => void;
};

const TABS: Array<{ id: AppTab; label: string; icon: React.ReactNode }> = [
  { id: "export", label: "Explorar y Carga Smart", icon: <SearchIcon style={{ width: 16, height: 16 }} /> },
  { id: "compare", label: "Comparar Playlist", icon: <CompareIcon style={{ width: 16, height: 16 }} /> },
  { id: "utilities", label: "Utilidades Locales", icon: <FolderIcon style={{ width: 16, height: 16 }} /> },
  { id: "downloads", label: "Cola de Descargas", icon: <DownloadIcon style={{ width: 16, height: 16 }} /> },
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
          {tab.icon}
          <span>{tab.label}</span>
        </button>
      ))}
    </nav>
  );
}

export default SubnavTabs;
export type { AppTab };

