pub mod compare;
pub mod downloads;
pub mod settings;
pub mod staging;
pub mod utilities;
pub mod workspace;

pub use compare::{CompareSubTab, CompareView};
pub use downloads::DownloadsView;
pub use settings::SettingsView;
pub use staging::{format_duration_human, format_duration_ms, StagingView};
pub use utilities::{format_file_size, UtilitiesView, UtilitySubTab};
pub use workspace::WorkspaceView;
