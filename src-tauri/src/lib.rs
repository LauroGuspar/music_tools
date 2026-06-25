use serde::Serialize;
use std::{
    fs,
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex},
    thread,
};
use tauri::{AppHandle, State};
use url::Url;

const SPOTIFY_AUTH_CALLBACK_PORT: u16 = 8000;

#[derive(Clone, Default)]
struct SpotifyAuthCallbackState(Arc<Mutex<Option<SpotifyAuthCallback>>>);

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
struct SpotifyAuthCallback {
    code: Option<String>,
    error: Option<String>,
    state: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct WriteExportedFilePayload {
    content: String,
    path: String,
}

#[tauri::command]
fn start_spotify_auth_callback(state: State<'_, SpotifyAuthCallbackState>) -> Result<String, String> {
    state
        .0
        .lock()
        .map_err(|_| "failed to lock auth callback state".to_string())?
        .take();

    let listener = TcpListener::bind(("127.0.0.1", SPOTIFY_AUTH_CALLBACK_PORT))
        .map_err(|error| format!("failed to start local auth listener: {error}"))?;

    let callback_state = state.inner().clone();

    thread::spawn(move || {
        if let Ok((stream, _)) = listener.accept() {
            handle_auth_callback_connection(stream, callback_state);
        }
    });

    Ok(format!(
        "http://127.0.0.1:{SPOTIFY_AUTH_CALLBACK_PORT}/callback"
    ))
}

#[tauri::command]
fn take_spotify_auth_callback(
    state: State<'_, SpotifyAuthCallbackState>,
) -> Result<Option<SpotifyAuthCallback>, String> {
    state
        .0
        .lock()
        .map(|mut callback| callback.take())
        .map_err(|_| "failed to lock auth callback state".to_string())
}

#[tauri::command]
fn write_exported_file(_app: AppHandle, payload: WriteExportedFilePayload) -> Result<(), String> {
    fs::write(payload.path, payload.content)
        .map_err(|error| format!("failed to write exported file: {error}"))
}

fn handle_auth_callback_connection(stream: TcpStream, state: SpotifyAuthCallbackState) {
    let callback = read_callback_from_stream(&stream).unwrap_or_else(|_| SpotifyAuthCallback {
        code: None,
        error: Some("invalid_request".to_string()),
        state: None,
    });

    if let Ok(mut auth_callback) = state.0.lock() {
        *auth_callback = Some(callback.clone());
    }

    let _ = write_callback_response(stream, callback.error.is_none());
}

fn read_callback_from_stream(stream: &TcpStream) -> Result<SpotifyAuthCallback, String> {
    let mut reader = BufReader::new(stream);
    let mut request_line = String::new();

    reader
        .read_line(&mut request_line)
        .map_err(|error| format!("failed to read auth callback request: {error}"))?;

    parse_callback_from_request_line(request_line.trim())
}

fn parse_callback_from_request_line(request_line: &str) -> Result<SpotifyAuthCallback, String> {
    let path_and_query = request_line
        .split_whitespace()
        .nth(1)
        .ok_or_else(|| "missing request target".to_string())?;

    let url = Url::parse(&format!("http://127.0.0.1{path_and_query}"))
        .map_err(|error| format!("invalid callback target: {error}"))?;

    if url.path() != "/callback" {
        return Err("unexpected callback path".to_string());
    }

    let mut code = None;
    let mut error = None;
    let mut state = None;

    for (key, value) in url.query_pairs() {
        match key.as_ref() {
            "code" => code = Some(value.into_owned()),
            "error" => error = Some(value.into_owned()),
            "state" => state = Some(value.into_owned()),
            _ => {}
        }
    }

    Ok(SpotifyAuthCallback { code, error, state })
}

fn write_callback_response(mut stream: TcpStream, success: bool) -> Result<(), String> {
    let (status, title, message) = if success {
        (
            "200 OK",
            "Spotify login completed",
            "You can return to Spotify Tools and continue.",
        )
    } else {
        (
            "400 Bad Request",
            "Spotify login failed",
            "Return to Spotify Tools and try the login flow again.",
        )
    };

    let body = format!(
        "<!doctype html><html><head><meta charset=\"utf-8\"><title>{title}</title></head><body><h1>{title}</h1><p>{message}</p></body></html>"
    );

    stream
        .write_all(
            format!(
                "HTTP/1.1 {status}\r\ncontent-type: text/html; charset=utf-8\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
                body.len()
            )
            .as_bytes(),
        )
        .map_err(|error| format!("failed to write auth callback response: {error}"))
}

#[cfg(test)]
mod tests {
    use super::parse_callback_from_request_line;

    #[test]
    fn parses_callback_with_code_and_state() {
        let callback = parse_callback_from_request_line(
            "GET /callback?code=abc123&state=state-123 HTTP/1.1",
        )
        .expect("callback should parse");

        assert_eq!(callback.code.as_deref(), Some("abc123"));
        assert_eq!(callback.error, None);
        assert_eq!(callback.state.as_deref(), Some("state-123"));
    }

    #[test]
    fn parses_callback_error_response() {
        let callback = parse_callback_from_request_line(
            "GET /callback?error=access_denied&state=state-123 HTTP/1.1",
        )
        .expect("callback should parse");

        assert_eq!(callback.code, None);
        assert_eq!(callback.error.as_deref(), Some("access_denied"));
        assert_eq!(callback.state.as_deref(), Some("state-123"));
    }

    #[test]
    fn rejects_unexpected_callback_path() {
        let error = parse_callback_from_request_line("GET /wrong?code=abc HTTP/1.1")
            .expect_err("callback path should be rejected");

        assert_eq!(error, "unexpected callback path");
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(SpotifyAuthCallbackState::default())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            write_exported_file,
            start_spotify_auth_callback,
            take_spotify_auth_callback
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
