use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use serde::Serialize;
use tauri::{AppHandle, Manager, Emitter};
use lofty::prelude::*;
use lofty::read_from_path;
use lofty::tag::{Tag, TagType};
use lofty::picture::{Picture, PictureType, MimeType};
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

pub fn write_mp3_tags(
    file_path: &str,
    title: &str,
    artists: &[String],
    album: Option<&str>,
    cover_data: Option<(&[u8], &str)>,
    year: Option<u32>,
) -> Result<(), String> {
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
    
    if let Some(alb) = album {
        if !alb.trim().is_empty() {
            tag.set_album(alb.to_string());
        }
    }

    if let Some(y) = year {
        tag.set_year(y);
    }

    if let Some((bytes, mime)) = cover_data {
        let mime_type = match mime {
            "image/png" => MimeType::Png,
            _ => MimeType::Jpeg,
        };
        let pic = Picture::new_unchecked(
            PictureType::CoverFront,
            Some(mime_type),
            None,
            bytes.to_vec(),
        );
        tag.push_picture(pic);
    }

    tagged_file.save_to_path(path, WriteOptions::default())
        .map_err(|e| format!("Error al guardar etiquetas en el archivo: {e}"))?;
        
    Ok(())
}

pub fn get_effective_ytdlp_command(app: &AppHandle) -> Result<Command, String> {
    let local_dir = app.path().app_local_data_dir()
        .map_err(|e| format!("No se pudo obtener directorio de datos: {e}"))?;
    let path = local_dir.join(get_ytdlp_filename());
    if path.exists() {
        return Ok(Command::new(path));
    }
    let sys_cmd = get_ytdlp_filename();
    if Command::new(sys_cmd).arg("--version").output().is_ok() {
        return Ok(Command::new(sys_cmd));
    }
    Err("El motor de descarga (yt-dlp) no está instalado. Ejecuta la instalación primero.".to_string())
}

#[tauri::command]
pub async fn save_track_tags(
    file_path: String,
    title: String,
    artists: Vec<String>,
    album: Option<String>,
    year: Option<u32>,
) -> Result<(), String> {
    write_mp3_tags(&file_path, &title, &artists, album.as_deref(), None, year)
}

#[tauri::command]
pub async fn check_ytdlp_installed(app: AppHandle) -> Result<bool, String> {
    Ok(get_effective_ytdlp_command(&app).is_ok())
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
    let sys_cmd = get_ytdlp_filename();
    if Command::new(sys_cmd).arg("--version").output().is_ok() {
        return Ok(sys_cmd.to_string());
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
    album_name: Option<String>,
    cover_url: Option<String>,
    release_year: Option<String>,
    custom_url: Option<String>,
) -> Result<String, String> {
    let mut cmd = match get_effective_ytdlp_command(&app) {
        Ok(c) => c,
        Err(e) => return Err(e),
    };
    
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
    
    let query = if let Some(ref url) = custom_url {
        if !url.trim().is_empty() {
            url.trim().to_string()
        } else {
            format!("ytsearch1:{} - {}", artists.join(" "), title)
        }
    } else {
        format!("ytsearch1:{} - {}", artists.join(" "), title)
    };
    let output_file_template = format!("{}/{} - {}.%(ext)s", download_dir, clean_artists, clean_title);
    
    let mut child = cmd
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
    
    // Inyectar tags ID3, álbum, año y carátula del álbum
    if Path::new(&final_mp3_path).exists() {
        let mut cover_bytes_vec: Option<(Vec<u8>, String)> = None;
        if let Some(ref url) = cover_url {
            if !url.trim().is_empty() {
                if let Ok(client) = reqwest::Client::builder()
                    .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
                    .build()
                {
                    if let Ok(resp) = client.get(url).send().await {
                        let content_type = resp.headers()
                            .get("content-type")
                            .and_then(|v| v.to_str().ok())
                            .unwrap_or("image/jpeg")
                            .to_string();
                        if let Ok(bytes) = resp.bytes().await {
                            cover_bytes_vec = Some((bytes.to_vec(), content_type));
                        }
                    }
                }
            }
        }

        let parsed_year = release_year.as_deref().and_then(|y| y.parse::<u32>().ok());
        let cover_ref = cover_bytes_vec.as_ref().map(|(b, m)| (b.as_slice(), m.as_str()));

        let _ = write_mp3_tags(
            &final_mp3_path,
            &title,
            &artists,
            album_name.as_deref(),
            cover_ref,
            parsed_year,
        );
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

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct YoutubeSearchResult {
    pub id: String,
    pub title: String,
    pub channel: String,
    pub duration: String,
    pub thumbnail_url: String,
    pub video_url: String,
}

#[tauri::command]
pub async fn search_youtube(
    app: AppHandle,
    query: String,
) -> Result<Vec<YoutubeSearchResult>, String> {
    let search_target = format!("ytsearch5:{}", query);
    let mut cmd = match get_effective_ytdlp_command(&app) {
        Ok(c) => c,
        Err(e) => return Err(e),
    };
    let output = cmd
        .arg(&search_target)
        .arg("--dump-single-json")
        .arg("--flat-playlist")
        .output()
        .map_err(|e| format!("Error al ejecutar la búsqueda en YouTube: {e}"))?;

    if !output.status.success() {
        return Err("No se encontraron resultados en YouTube.".to_string());
    }

    let json_str = String::from_utf8_lossy(&output.stdout);
    let val: serde_json::Value = serde_json::from_str(&json_str)
        .map_err(|e| format!("Error al interpretar la respuesta de YouTube: {e}"))?;

    let mut results = Vec::new();
    if let Some(entries) = val.get("entries").and_then(|v| v.as_array()) {
        for entry in entries {
            let id = entry.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let title = entry.get("title").and_then(|v| v.as_str()).unwrap_or("Sin título").to_string();
            let channel = entry.get("channel")
                .or_else(|| entry.get("uploader"))
                .and_then(|v| v.as_str())
                .unwrap_or("Desconocido")
                .to_string();

            let duration_secs = entry.get("duration").and_then(|v| v.as_f64()).unwrap_or(0.0) as u64;
            let mins = duration_secs / 60;
            let secs = duration_secs % 60;
            let duration = format!("{}:{:02}", mins, secs);

            let video_url = entry.get("url")
                .or_else(|| entry.get("webpage_url"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| format!("https://www.youtube.com/watch?v={}", id));

            let thumbnail_url = entry.get("thumbnails")
                .and_then(|v| v.as_array())
                .and_then(|arr| arr.last())
                .and_then(|thumb| thumb.get("url"))
                .and_then(|v| v.as_str())
                .or_else(|| entry.get("thumbnail").and_then(|v| v.as_str()))
                .unwrap_or("")
                .to_string();

            if !id.is_empty() {
                results.push(YoutubeSearchResult {
                    id,
                    title,
                    channel,
                    duration,
                    thumbnail_url,
                    video_url,
                });
            }
        }
    }

    Ok(results)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct YoutubePlaylistDetails {
    pub id: String,
    pub title: String,
    pub tracks: Vec<YoutubeSearchResult>,
}

#[tauri::command]
pub async fn fetch_youtube_playlist(
    app: AppHandle,
    url_or_id: String,
) -> Result<YoutubePlaylistDetails, String> {
    let target = if url_or_id.starts_with("http://") || url_or_id.starts_with("https://") {
        url_or_id.clone()
    } else {
        format!("https://www.youtube.com/playlist?list={}", url_or_id)
    };

    let mut cmd = match get_effective_ytdlp_command(&app) {
        Ok(c) => c,
        Err(e) => return Err(e),
    };

    let output = cmd
        .arg(&target)
        .arg("--dump-single-json")
        .arg("--flat-playlist")
        .output()
        .map_err(|e| format!("Error al consultar la playlist de YouTube: {e}"))?;

    if !output.status.success() {
        return Err("No se pudo obtener la información de la playlist de YouTube. Revisa la URL o si es privada.".to_string());
    }

    let json_str = String::from_utf8_lossy(&output.stdout);
    let val: serde_json::Value = serde_json::from_str(&json_str)
        .map_err(|e| format!("Error al procesar la respuesta de la playlist: {e}"))?;

    let playlist_id = val.get("id").and_then(|v| v.as_str()).unwrap_or(&url_or_id).to_string();
    let playlist_title = val.get("title")
        .or_else(|| val.get("playlist_title"))
        .and_then(|v| v.as_str())
        .unwrap_or("Playlist de YouTube")
        .to_string();

    let mut tracks = Vec::new();
    if let Some(entries) = val.get("entries").and_then(|v| v.as_array()) {
        for entry in entries {
            let id = entry.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let title = entry.get("title").and_then(|v| v.as_str()).unwrap_or("Sin título").to_string();
            let channel = entry.get("channel")
                .or_else(|| entry.get("uploader"))
                .and_then(|v| v.as_str())
                .unwrap_or("Desconocido")
                .to_string();

            let duration_secs = entry.get("duration").and_then(|v| v.as_f64()).unwrap_or(0.0) as u64;
            let mins = duration_secs / 60;
            let secs = duration_secs % 60;
            let duration = format!("{}:{:02}", mins, secs);

            let video_url = entry.get("url")
                .or_else(|| entry.get("webpage_url"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| format!("https://www.youtube.com/watch?v={}", id));

            let thumbnail_url = entry.get("thumbnails")
                .and_then(|v| v.as_array())
                .and_then(|arr| arr.last())
                .and_then(|thumb| thumb.get("url"))
                .and_then(|v| v.as_str())
                .or_else(|| entry.get("thumbnail").and_then(|v| v.as_str()))
                .unwrap_or("")
                .to_string();

            if !id.is_empty() {
                tracks.push(YoutubeSearchResult {
                    id,
                    title,
                    channel,
                    duration,
                    thumbnail_url,
                    video_url,
                });
            }
        }
    }

    Ok(YoutubePlaylistDetails {
        id: playlist_id,
        title: playlist_title,
        tracks,
    })
}

#[tauri::command]
pub async fn fetch_youtube_video_info(
    app: AppHandle,
    url_or_id: String,
) -> Result<YoutubeSearchResult, String> {
    let target = if url_or_id.starts_with("http://") || url_or_id.starts_with("https://") {
        url_or_id.clone()
    } else {
        format!("https://www.youtube.com/watch?v={}", url_or_id)
    };

    let mut cmd = match get_effective_ytdlp_command(&app) {
        Ok(c) => c,
        Err(e) => return Err(e),
    };

    let output = cmd
        .arg(&target)
        .arg("--dump-single-json")
        .arg("--no-playlist")
        .output()
        .map_err(|e| format!("Error al consultar el video de YouTube: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let clean_err = stderr.lines().last().unwrap_or("Verifica la URL o conexión.");
        return Err(format!("No se pudo obtener la información del video de YouTube: {}", clean_err));
    }

    let json_str = String::from_utf8_lossy(&output.stdout);
    let val: serde_json::Value = serde_json::from_str(&json_str)
        .map_err(|e| format!("Error al procesar los datos del video: {e}"))?;

    let id = val.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let title = val.get("title").and_then(|v| v.as_str()).unwrap_or("Sin título").to_string();
    let channel = val.get("channel")
        .or_else(|| val.get("uploader"))
        .and_then(|v| v.as_str())
        .unwrap_or("Desconocido")
        .to_string();

    let duration_secs = val.get("duration").and_then(|v| v.as_f64()).unwrap_or(0.0) as u64;
    let mins = duration_secs / 60;
    let secs = duration_secs % 60;
    let duration = format!("{}:{:02}", mins, secs);

    let video_url = val.get("webpage_url")
        .or_else(|| val.get("url"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| format!("https://www.youtube.com/watch?v={}", id));

    let thumbnail_url = val.get("thumbnails")
        .and_then(|v| v.as_array())
        .and_then(|arr| arr.last())
        .and_then(|thumb| thumb.get("url"))
        .and_then(|v| v.as_str())
        .or_else(|| val.get("thumbnail").and_then(|v| v.as_str()))
        .unwrap_or("")
        .to_string();

    Ok(YoutubeSearchResult {
        id,
        title,
        channel,
        duration,
        thumbnail_url,
        video_url,
    })
}

