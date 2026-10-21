use std::path::{Path, PathBuf};
use std::process::Command;

/// Opens a native folder selection dialog.
/// Uses `rfd` first (supporting Wayland and X11 via XDG Desktop Portal / GTK),
/// and falls back to `zenity` on Linux if needed.
pub fn pick_folder(title: &str, initial_path: Option<&Path>) -> Option<PathBuf> {
    let mut dialog = rfd::FileDialog::new().set_title(title);
    if let Some(path) = initial_path {
        if path.exists() {
            dialog = dialog.set_directory(path);
        }
    }

    if let Some(folder) = dialog.pick_folder() {
        return Some(folder);
    }

    // Fallback to zenity on Linux if rfd returns None
    #[cfg(target_os = "linux")]
    {
        let mut cmd = Command::new("zenity");
        cmd.args(["--file-selection", "--directory", &format!("--title={title}")]);
        if let Some(path) = initial_path {
            if path.exists() {
                cmd.arg(format!("--filename={}/", path.display()));
            }
        }
        if let Ok(output) = cmd.output() {
            if output.status.success() {
                let chosen = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !chosen.is_empty() {
                    let p = PathBuf::from(chosen);
                    if p.is_dir() {
                        return Some(p);
                    }
                }
            }
        }
    }

    None
}

/// Helper function to expand `~` in file paths.
pub fn expand_tilde(path: &str) -> PathBuf {
    let trimmed = path.trim();
    if let Some(rest) = trimmed.strip_prefix("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(rest);
        }
    } else if trimmed == "~" {
        if let Some(home) = dirs::home_dir() {
            return home;
        }
    }
    PathBuf::from(trimmed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_tilde() {
        let expanded = expand_tilde("~/Music");
        assert!(!expanded.to_string_lossy().starts_with('~'));

        let normal = expand_tilde("/tmp/test");
        assert_eq!(normal, PathBuf::from("/tmp/test"));
    }
}
