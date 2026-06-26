# Plan de Implementación de Futuras Mejoras

Este documento detalla el plan de acción para las próximas características y mejoras del ecosistema de herramientas de música (`spotify_tools`), ordenadas por prioridad de implementación (de mayor a menor importancia).

---

## 📅 Resumen de Etapas de Prioridad

1. **Etapa 1: Enriquecimiento de Metadatos (ID3 + Cover Art)** (Más importante)
2. **Etapa 2: Reproductor de Audio Integrado (Inline Preview)**
3. **Etapa 3: Editor de Metadatos de Archivos Locales (Tag Editor)**
4. **Etapa 4: Búsqueda Visual y Selección Manual de YouTube (Opcional)**

---

## 🛠️ Detalle de cada Etapa

### Etapa 1: Enriquecimiento de Metadatos (ID3 + Cover Art)
**Objetivo:** Al descargar canciones, no solo inyectar el Título y Artistas (que ya se hace de forma básica), sino también el Álbum, Año, Género y la **Carátula del álbum (Cover Art)** directamente dentro del archivo MP3/M4A.

#### 📝 Tareas a realizar:
1. **Frontend:**
   - Pasar el campo `albumName` y la URL de la carátula `albumImageUrl` desde la playlist de Spotify al backend durante el llamado a `download_track`.
2. **Backend (Rust):**
   - Modificar la firma del comando `download_track` para recibir `album_name: Option<String>` y `cover_url: Option<String>`.
   - Implementar la descarga de la imagen de la carátula usando `reqwest` a un buffer de bytes temporales.
   - En la función `write_mp3_tags` de `src-tauri/src/downloader.rs`, usar `lofty::picture::Picture` para incrustar los bytes de la imagen descargada en el tag con el tipo `PictureType::CoverFront`.
   - Guardar etiquetas de Álbum (`tag.set_album`) y Año (si está disponible en Spotify).

#### 💡 Beneficio:
Las canciones descargadas se verán profesionales con su arte oficial y detalles completos en cualquier reproductor (móvil, auto, reproductor de sistema).

---

### Etapa 2: Reproductor de Audio Integrado (Inline Preview)
**Objetivo:** Permitir escuchar vistas previas rápidas de las canciones directamente en la aplicación sin tener que abrir el reproductor externo del sistema operativo.

#### 📝 Tareas a realizar:
1. **Backend (Rust):**
   - Exponer un comando Tauri (e.g. `get_audio_stream` o usar el asset protocol de Tauri) para permitir que el frontend cargue archivos locales en un elemento `<audio>` de HTML5 de manera segura.
2. **Frontend:**
   - Crear un componente de barra flotante global de reproducción (`AudioPlayer.tsx`) con controles estilizados (Play, Pausa, barra de progreso de tiempo, control de volumen con sliders).
   - Reemplazar las llamadas de `openPath` del reproductor de sistema en los paneles de duplicados y archivos corruptos por una acción que cargue y reproduzca el archivo directamente en este reproductor flotante interno.

#### 💡 Beneficio:
Facilita la verificación rápida de archivos duplicados o corruptos de forma rápida y cómoda.

---

### Etapa 3: Editor de Metadatos de Archivos Locales (Tag Editor)
**Objetivo:** Permitir al usuario corregir manualmente el título, los artistas y el álbum de cualquier archivo local directamente en la herramienta de limpieza.

#### 📝 Tareas a realizar:
1. **Backend (Rust):**
   - Crear un comando Tauri `save_track_tags(path: String, title: String, artists: Vec<String>, album: String)` que use `lofty` para escribir y sobreescribir las etiquetas de metadatos de un archivo local especificado.
2. **Frontend:**
   - En la lista de archivos duplicados y dañados, agregar un botón de "Editar Metadatos".
   - Abrir un diálogo modal con un formulario limpio (inputs para Título, Artistas separados por coma, Álbum) que actualice el archivo llamando al comando de Rust.
   - Refrescar dinámicamente la vista del panel después de guardar.

#### 💡 Beneficio:
Da control total al usuario para organizar su biblioteca local que ya tiene canciones mal nombradas.

---

### Etapa 4: Búsqueda Visual y Selección Manual de YouTube (Opcional)
**Objetivo:** Darle la opción al usuario de elegir de qué video de YouTube descargar la canción si el resultado automático ("Recomendado") no es el correcto (ej. si descarga una versión en vivo o un remix).

#### 📝 Tareas a realizar:
1. **Backend (Rust):**
   - Crear un comando Tauri `search_youtube(query: String) -> Vec<YoutubeSearchResult>` que ejecute `yt-dlp` con `--dump-json` o use una consulta simple a una API para obtener títulos, canales, duraciones y URLs de los primeros 5 resultados de búsqueda en YouTube.
2. **Frontend:**
   - Agregar un selector en la configuración: *"Elegir canción antes de descargar (Manual)"* vs *"Descargar recomendados de forma automática"* (default).
   - Si se activa el modo manual, al iniciar una descarga se abrirá un panel modal con los 5 videos encontrados, mostrando su miniatura, título y duración para que el usuario escoja el correcto.

#### 💡 Beneficio:
Soluciona casos raros donde la selección automática no descarga la versión de estudio de la canción.
