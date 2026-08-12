# Plan de Implementación: Descargas de YouTube (Canciones y Playlists)

Este documento detalla la arquitectura, características y el flujo de desarrollo planeado para integrar la descarga de música (.mp3) y playlists completas desde YouTube en **Music Tools**.

---

## 🎯 Objetivo Principal

Permitir a los usuarios descargar audio en formato `.mp3` de alta calidad a partir de:
1. **URLs individuales de YouTube** (videos/canciones).
2. **URLs de Playlists completas de YouTube**.
3. **Búsqueda directa integrada** por nombre de canción o artista dentro de la aplicación.

---

## 🧱 Fases de Desarrollo

### 1. Backend en Rust (`src-tauri/src/downloader.rs` y `lib.rs`)
- **Extractor de Playlists de YouTube (`fetch_youtube_playlist`):**
  - Crear un comando IPC que ejecute `yt-dlp --dump-single-json --flat-playlist <URL_PLAYLIST>`.
  - Parsear la respuesta JSON para retornar un struct `EmbedPlaylistDetails` o `YoutubePlaylistDetails` con:
    - Nombre de la playlist.
    - Lista de canciones (ID de video, título, uploader/canal, duración, URL del thumbnail).
- **Soporte de Descarga Directa por URL (`download_youtube_url`):**
  - Optimizar el motor de descarga `download_track` para aceptar tanto términos de búsqueda (`ytsearch1:...`) como URLs directas de YouTube (`https://www.youtube.com/watch?v=...` o `https://youtu.be/...`).
  - Extraer automáticamente la miniatura del video en alta resolución (Thumbnail) para inyectarla como carátula de álbum del archivo `.mp3` mediante la librería `lofty`.
- **Conversión y Metadatos ID3:**
  - Garantizar que la conversión con `ffmpeg` genere metadatos ID3v2 limpios (Título, Artista, Álbum, Año, Carátula).

### 2. Frontend en React (`src/components/DownloadsPanel.tsx`)
- **Selector de Entrada Inteligente (Input Unificado):**
  - Crear un campo de entrada que detecte automáticamente el tipo de enlace ingresado:
    - 🔗 **Enlace de Canción de YouTube:** Carga la vista previa del video individual con opción de descarga instantánea.
    - 📋 **Enlace de Playlist de YouTube:** Extrae todos los videos de la lista y los añade a la cola de descargas.
    - 🟢 **Enlace/ID de Playlist de Spotify:** Mantiene la compatibilidad existente para convertir y buscar la playlist en YouTube.
    - 🔍 **Texto Libre:** Ejecuta una búsqueda directa en YouTube y muestra los resultados.
- **Vista Previa de Playlist de YouTube:**
  - Al pegar un enlace de playlist de YouTube, mostrar una tarjeta resumen con:
    - Título de la playlist y número total de canciones.
    - Lista con casillas de verificación (*checkboxes*) para permitir al usuario seleccionar/desmarcar qué canciones desea descargar.
    - Botón principal: **"Añadir X canciones a la cola de descarga"**.
- **Buscador Integrado con Descarga en 1-Clic:**
  - Tarjetas visuales para resultados de búsqueda con la carátula, duración, nombre del canal y botones **"Escuchar previa"** y **"Descargar MP3"**.

### 3. Cola de Descargas y Experiencia de Usuario
- **Gestión de Descargas Concurrentes:**
  - Reutilizar y mejorar el sistema de cola concurrente (3 descargas simultáneas).
  - Notificaciones de estado en tiempo real vía eventos de Tauri: *Esperando*, *Buscando*, *Descargando (con % y velocidad en MB/s)*, *Convirtiendo a MP3* y *Completado*.
- **Control de Duplicados y Almacenamiento:**
  - Respetar las opciones de configuración: **Omitir si ya existe en disco** o **Sobrescribir**.
- **Reproducción Inmediata:**
  - Una vez finalizada la descarga de cualquier canción, mostrar el botón **"Escuchar"** que activa el reproductor de audio interno de la aplicación.

---

## 📋 Pasos de Ejecución Propuestos

1. **Paso 1:** Implementación de funciones backend en Rust (`downloader.rs`).
2. **Paso 2:** Creación de componentes React para la detección de URLs y la vista previa de listas de YouTube.
3. **Paso 3:** Conexión con la cola de descargas y verificación de etiquetas ID3.
4. **Paso 4:** Pruebas de descarga con videos individuales y playlists públicas de YouTube.
