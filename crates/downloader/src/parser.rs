use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ProgressState {
    Downloading,
    Converting,
    Error(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProgressUpdate {
    pub percent: f32,
    pub speed: Option<String>,
    pub eta: Option<String>,
    pub state: ProgressState,
}

impl ProgressUpdate {
    pub fn is_downloading(&self) -> bool {
        matches!(self.state, ProgressState::Downloading)
    }

    pub fn is_converting(&self) -> bool {
        matches!(self.state, ProgressState::Converting)
    }

    pub fn is_error(&self) -> bool {
        matches!(self.state, ProgressState::Error(_))
    }

    pub fn error_message(&self) -> Option<&str> {
        match &self.state {
            ProgressState::Error(msg) => Some(msg.as_str()),
            _ => None,
        }
    }
}

/// Replaces invalid filename characters (`/`, `\`, `:`, `*`, `?`, `"`, `<`, `>`, `|`) with `_`.
pub fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            other => other,
        })
        .collect()
}

/// Parses a line emitted by `yt-dlp` to extract download progress, audio extraction state, or errors.
pub fn parse_ytdlp_line(line: &str) -> Option<ProgressUpdate> {
    let trimmed = line.trim();

    // 1. Error detection
    if let Some(err_idx) = trimmed.find("ERROR:") {
        let msg = trimmed[err_idx + 6..].trim();
        let error_msg = if msg.is_empty() {
            trimmed.to_string()
        } else {
            msg.to_string()
        };
        return Some(ProgressUpdate {
            percent: 0.0,
            speed: None,
            eta: None,
            state: ProgressState::Error(error_msg),
        });
    }

    // 2. Conversion / audio extraction detection
    if trimmed.contains("[ExtractAudio]") {
        return Some(ProgressUpdate {
            percent: 100.0,
            speed: None,
            eta: None,
            state: ProgressState::Converting,
        });
    }

    // 3. Download progress detection: line starts with `[download]`
    if trimmed.starts_with("[download]") {
        let after_download = &trimmed[10..];
        if let Some(pct_idx) = after_download.find('%') {
            let before_pct = &after_download[..pct_idx];
            if let Some(num_str) = before_pct.split_whitespace().last() {
                if let Ok(percent) = num_str.parse::<f32>() {
                    let remainder = &after_download[pct_idx + 1..];
                    let mut speed = None;
                    let mut eta = None;

                    if let Some(at_idx) = remainder.find(" at ") {
                        let after_at = remainder[at_idx + 4..].trim_start();
                        if let Some(eta_idx) = after_at.find(" ETA ") {
                            let speed_candidate = after_at[..eta_idx].trim();
                            if !speed_candidate.is_empty()
                                && speed_candidate != "Unknown speed"
                                && speed_candidate != "N/A"
                            {
                                speed = Some(speed_candidate.to_string());
                            }

                            let eta_candidate = after_at[eta_idx + 5..].trim();
                            let clean_eta = if let Some(paren_idx) = eta_candidate.find('(') {
                                eta_candidate[..paren_idx].trim()
                            } else {
                                eta_candidate.split_whitespace().next().unwrap_or(eta_candidate)
                            };

                            if !clean_eta.is_empty()
                                && clean_eta != "Unknown"
                                && clean_eta != "N/A"
                            {
                                eta = Some(clean_eta.to_string());
                            }
                        } else {
                            // Format: "... at SPEED"
                            let token = after_at.split_whitespace().next().unwrap_or("");
                            if !token.is_empty() && token != "Unknown" && token != "N/A" {
                                speed = Some(token.to_string());
                            }
                        }
                    } else if let Some(eta_idx) = remainder.find(" ETA ") {
                        let eta_candidate = remainder[eta_idx + 5..].trim();
                        let clean_eta = if let Some(paren_idx) = eta_candidate.find('(') {
                            eta_candidate[..paren_idx].trim()
                        } else {
                            eta_candidate.split_whitespace().next().unwrap_or(eta_candidate)
                        };

                        if !clean_eta.is_empty() && clean_eta != "Unknown" && clean_eta != "N/A" {
                            eta = Some(clean_eta.to_string());
                        }
                    }

                    return Some(ProgressUpdate {
                        percent,
                        speed,
                        eta,
                        state: ProgressState::Downloading,
                    });
                }
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(
            sanitize_filename("AC/DC: Back in Black *Remastered*?"),
            "AC_DC_ Back in Black _Remastered__"
        );
        assert_eq!(
            sanitize_filename("Track <with> \"quotes\" | pipes & symbols \\ / ? * :"),
            "Track _with_ _quotes_ _ pipes & symbols _ _ _ _ _"
        );
        assert_eq!(sanitize_filename("Clean Song Name.mp3"), "Clean Song Name.mp3");
    }

    #[test]
    fn test_parse_ytdlp_download_percentage_speed_eta() {
        let line = "[download]  45.5% of   10.23MiB at  5.12MiB/s ETA 00:02";
        let parsed = parse_ytdlp_line(line).expect("should parse progress");
        assert_eq!(parsed.percent, 45.5);
        assert_eq!(parsed.speed.as_deref(), Some("5.12MiB/s"));
        assert_eq!(parsed.eta.as_deref(), Some("00:02"));
        assert!(parsed.is_downloading());
    }

    #[test]
    fn test_parse_ytdlp_download_with_fragments() {
        let line = "[download]  12.3% of ~  20.00MiB at  1.50MiB/s ETA 00:15 (frag 2/10)";
        let parsed = parse_ytdlp_line(line).expect("should parse progress with frags");
        assert_eq!(parsed.percent, 12.3);
        assert_eq!(parsed.speed.as_deref(), Some("1.50MiB/s"));
        assert_eq!(parsed.eta.as_deref(), Some("00:15"));
        assert!(parsed.is_downloading());
    }

    #[test]
    fn test_parse_ytdlp_download_completed_line() {
        let line = "[download] 100% of 10.23MiB in 00:02 at 4.50MiB/s";
        let parsed = parse_ytdlp_line(line).expect("should parse 100%");
        assert_eq!(parsed.percent, 100.0);
        assert_eq!(parsed.speed.as_deref(), Some("4.50MiB/s"));
        assert_eq!(parsed.eta, None);
        assert!(parsed.is_downloading());
    }

    #[test]
    fn test_parse_ytdlp_extract_audio() {
        let line = "[ExtractAudio] Destination: /music/Pink Floyd - Time.mp3";
        let parsed = parse_ytdlp_line(line).expect("should parse extract audio");
        assert_eq!(parsed.percent, 100.0);
        assert!(parsed.is_converting());
    }

    #[test]
    fn test_parse_ytdlp_error() {
        let line = "ERROR: [youtube] 12345: Video unavailable. This video is private.";
        let parsed = parse_ytdlp_line(line).expect("should parse error");
        assert!(parsed.is_error());
        assert_eq!(parsed.percent, 0.0);
        assert!(
            parsed
                .error_message()
                .unwrap()
                .contains("Video unavailable. This video is private.")
        );
    }

    #[test]
    fn test_parse_ytdlp_ignore_unrelated_lines() {
        assert_eq!(parse_ytdlp_line("[youtube] Extracting URL: https://..."), None);
        assert_eq!(
            parse_ytdlp_line("[download] Destination: Pink Floyd - Time.webm"),
            None
        );
        assert_eq!(parse_ytdlp_line(""), None);
        assert_eq!(parse_ytdlp_line("Deleting original file /music/Pink Floyd - Time.webm"), None);
    }
}
