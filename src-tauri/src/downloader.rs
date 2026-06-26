use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use serde::Serialize;
use tauri::{AppHandle, Manager, Emitter};
use lofty::prelude::*;
use lofty::read_from_path;
use lofty::tag::{Tag, TagType};
use lofty::config::WriteOptions;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct DownloadProgress {
    track_key: String,
    percentage: f32,
    speed: String,
    eta: String,
    status: String, // "waiting", "searching", "downloading", "converting", "completed", "failed"
}

fn get_ytdlp_filename() -> &'static str {
    if cfg!(target_os = "windows") {
        "yt-dlp.exe"
    } else if cfg!(target_os = "macos") {
        "yt-dlp_macos"
    } else {
        "yt-dlp"
    }
}

fn get_ytdlp_url() -> &'static str {
    if cfg!(target_os = "windows") {
        "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp.exe"
    } else if cfg!(target_os = "macos") {
        "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp_macos"
    } else {
        "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp"
    }
}

fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            other => other,
        })
        .collect()
}

pub fn write_mp3_tags(file_path: &str, title: &str, artists: &[String]) -> Result<(), String> {
    let path = Path::new(file_path);
    
    // Lofty leerá el archivo de música
    let mut tagged_file = read_from_path(path)
        .map_err(|e| format!("Error al leer metadatos del archivo para etiquetado: {e}"))?;
        
    let tag = if let Some(tag) = tagged_file.primary_tag_mut() {
        tag
    } else if let Some(tag) = tagged_file.first_tag_mut() {
        tag
    } else {
        let tag_type = match path.extension().and_then(|ext| ext.to_str()).map(|ext| ext.to_lowercase()).as_deref() {
            Some("mp3") => TagType::Id3v2,
            Some("flac") => TagType::VorbisComments,
            Some("m4a") => TagType::Mp4Ilst,
            _ => TagType::Id3v2,
        };
        tagged_file.insert_tag(Tag::new(tag_type));
        tagged_file.primary_tag_mut().ok_or_else(|| "No se pudo crear etiqueta de audio".to_string())?
    };
    
    tag.set_title(title.to_string());
    tag.set_artist(artists.join(", "));
    
    tagged_file.save_to_path(path, WriteOptions::default())
        .map_err(|e| format!("Error al guardar etiquetas en el archivo: {e}"))?;
        
    Ok(())
}

#[tauri::command]
pub async fn check_ytdlp_installed(app: AppHandle) -> Result<bool, String> {
    let local_dir = app.path().app_local_data_dir()
        .map_err(|e| format!("No se pudo obtener directorio de datos: {e}"))?;
    let path = local_dir.join(get_ytdlp_filename());
    Ok(path.exists())
}

#[tauri::command]
pub async fn check_ffmpeg_installed() -> Result<bool, String> {
    let output = Command::new("ffmpeg")
        .arg("-version")
        .output();
    Ok(output.is_ok())
}

#[tauri::command]
pub async fn ensure_ytdlp_downloaded(app: AppHandle) -> Result<String, String> {
    let local_dir = app.path().app_local_data_dir()
        .map_err(|e| format!("No se pudo obtener directorio de datos: {e}"))?;
    
    if !local_dir.exists() {
        fs::create_dir_all(&local_dir)
            .map_err(|e| format!("Error al crear directorio de datos: {e}"))?;
    }
    
    let ytdlp_path = local_dir.join(get_ytdlp_filename());
    if ytdlp_path.exists() {
        return Ok(ytdlp_path.to_string_lossy().to_string());
    }
    
    let url = get_ytdlp_url();
    
    app.emit("ytdlp-download-status", "descargando").unwrap_or(());
    
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .build()
        .map_err(|e| format!("Error al crear cliente HTTP: {e}"))?;

    let response = client.get(url)
        .send()
        .await
        .map_err(|e| format!("Error al iniciar descarga de yt-dlp: {e}"))?;
        
    let content = response.bytes()
        .await
        .map_err(|e| format!("Error al leer bytes de descarga: {e}"))?;
        
    let mut file = File::create(&ytdlp_path)
        .map_err(|e| format!("Error al crear archivo yt-dlp en disco: {e}"))?;
        
    file.write_all(&content)
        .map_err(|e| format!("Error al escribir yt-dlp en disco: {e}"))?;
        
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&ytdlp_path)
            .map_err(|e| format!("Error al obtener metadatos: {e}"))?
            .permissions();
        perms.set_mode(0o755); // executable
        fs::set_permissions(&ytdlp_path, perms)
            .map_err(|e| format!("Error al asignar permisos de ejecución: {e}"))?;
    }
    
    app.emit("ytdlp-download-status", "completado").unwrap_or(());
    
    Ok(ytdlp_path.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn download_track(
    app: AppHandle,
    title: String,
    artists: Vec<String>,
    download_dir: String,
) -> Result<String, String> {
    let local_dir = app.path().app_local_data_dir()
        .map_err(|e| format!("No se pudo obtener directorio de datos: {e}"))?;
    let ytdlp_path = local_dir.join(get_ytdlp_filename());
    
    if !ytdlp_path.exists() {
        return Err("El motor de descarga (yt-dlp) no está instalado. Ejecuta la instalación primero.".to_string());
    }
    
    let clean_title = sanitize_filename(&title);
    let clean_artists = sanitize_filename(&artists.join(", "));
    
    let track_key = format!("{} - {}", artists.join(", "), title);
    let final_mp3_path = format!("{}/{} - {}.mp3", download_dir, clean_artists, clean_title);
    
    // Si ya existe la canción descargada, la omitimos.
    if Path::new(&final_mp3_path).exists() {
        app.emit("download-progress", DownloadProgress {
            track_key: track_key.clone(),
            percentage: 100.0,
            speed: "".to_string(),
            eta: "".to_string(),
            status: "completed".to_string(),
        }).unwrap_or(());
        return Ok("skipped_existing".to_string());
    }
    
    // Notificamos que iniciamos la búsqueda
    app.emit("download-progress", DownloadProgress {
        track_key: track_key.clone(),
        percentage: 0.0,
        speed: "".to_string(),
        eta: "".to_string(),
        status: "searching".to_string(),
    }).unwrap_or(());
    
    let query = format!("ytsearch1:{} - {}", artists.join(" "), title);
    let output_file_template = format!("{}/{} - {}.%(ext)s", download_dir, clean_artists, clean_title);
    
    let mut child = Command::new(&ytdlp_path)
        .arg("--extract-audio")
        .arg("--audio-format")
        .arg("mp3")
        .arg("--audio-quality")
        .arg("0")
        .arg("--newline")
        .arg(&query)
        .arg("-o")
        .arg(&output_file_template)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("Error al iniciar proceso de descarga: {e}"))?;
        
    let stdout = child.stdout.take().ok_or("No se pudo capturar la salida de descarga")?;
    let reader = BufReader::new(stdout);
    
    for line in reader.lines() {
        if let Ok(line_str) = line {
            if line_str.starts_with("[download]") {
                if let Some(pct_idx) = line_str.find('%') {
                    let pct_str = &line_str[10..pct_idx].trim();
                    if let Ok(pct) = pct_str.parse::<f32>() {
                        let mut speed = "N/A".to_string();
                        let mut eta = "N/A".to_string();
                        if let Some(at_idx) = line_str.find(" at ") {
                            if let Some(eta_idx) = line_str.find(" ETA ") {
                                speed = line_str[at_idx + 4..eta_idx].trim().to_string();
                                eta = line_str[eta_idx + 5..].trim().to_string();
                            }
                        }
                        
                        app.emit("download-progress", DownloadProgress {
                            track_key: track_key.clone(),
                            percentage: pct,
                            speed,
                            eta,
                            status: "downloading".to_string(),
                        }).unwrap_or(());
                    }
                }
            } else if line_str.contains("[ExtractAudio]") {
                app.emit("download-progress", DownloadProgress {
                    track_key: track_key.clone(),
                    percentage: 100.0,
                    speed: "".to_string(),
                    eta: "".to_string(),
                    status: "converting".to_string(),
                }).unwrap_or(());
            }
        }
    }
    
    let exit_status = child.wait().map_err(|e| format!("Error al esperar que termine la descarga: {e}"))?;
    if !exit_status.success() {
        app.emit("download-progress", DownloadProgress {
            track_key: track_key.clone(),
            percentage: 0.0,
            speed: "".to_string(),
            eta: "".to_string(),
            status: "failed".to_string(),
        }).unwrap_or(());
        return Err("Error en la ejecución de yt-dlp".to_string());
    }
    
    // Inyectar tags ID3
    if Path::new(&final_mp3_path).exists() {
        let _ = write_mp3_tags(&final_mp3_path, &title, &artists);
    }
    
    app.emit("download-progress", DownloadProgress {
        track_key: track_key.clone(),
        percentage: 100.0,
        speed: "".to_string(),
        eta: "".to_string(),
        status: "completed".to_string(),
    }).unwrap_or(());
    
    Ok("success".to_string())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EmbedTrack {
    title: String,
    artists: Vec<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EmbedPlaylistDetails {
    id: String,
    name: String,
    tracks: Vec<EmbedTrack>,
}

#[tauri::command]
pub async fn fetch_public_playlist_embed(playlist_id: String) -> Result<EmbedPlaylistDetails, String> {
    let url = format!("https://open.spotify.com/embed/playlist/{}", playlist_id);
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .build()
        .map_err(|e| format!("Error al inicializar cliente HTTP: {e}"))?;
        
    let html = client.get(&url)
        .send()
        .await
        .map_err(|e| format!("Error al conectar con la página pública de Spotify: {e}"))?
        .text()
        .await
        .map_err(|e| format!("Error al descargar el contenido de la playlist: {e}"))?;
        
    let start_tag = "<script id=\"__NEXT_DATA__\" type=\"application/json\">";
    let end_tag = "</script>";
    
    let start_idx = html.find(start_tag)
        .ok_or_else(|| "No se encontró el script de datos público en Spotify. Asegúrate de que la playlist sea pública.".to_string())? 
        + start_tag.len();
        
    let end_idx = html[start_idx..].find(end_tag)
        .ok_or_else(|| "Estructura de página pública de Spotify no reconocida.".to_string())? 
        + start_idx;
        
    let json_str = &html[start_idx..end_idx];
    let val: serde_json::Value = serde_json::from_str(json_str)
        .map_err(|e| format!("Error de decodificación JSON de Spotify: {e}"))?;
        
    let entity = val.pointer("/props/pageProps/state/data/entity")
        .ok_or_else(|| "Estructura de playlist pública no encontrada en los datos de la página.".to_string())?;
        
    let name = entity.get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("Playlist Pública")
        .to_string();
        
    let mut tracks = Vec::new();
    
    if let Some(track_list) = entity.get("trackList").and_then(|v| v.as_array()) {
        for item in track_list {
            let title = item.get("title").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let subtitle = item.get("subtitle").and_then(|v| v.as_str()).unwrap_or("");
            
            // Separar artistas por coma
            let artists: Vec<String> = subtitle
                .split(", ")
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
                
            if !title.is_empty() && !artists.is_empty() {
                tracks.push(EmbedTrack { title, artists });
            }
        }
    }
    
    Ok(EmbedPlaylistDetails {
        id: playlist_id,
        name,
        tracks,
    })
}
