# Plan de Implementación y Mejoras (Versión 2)

Este documento contiene la auditoría completa, el diagnóstico de fallos detectados y el plan de mejoras de arquitectura y usabilidad para la versión v2 de **Music Tools**.

---

## 🔍 Diagnóstico del Sistema y Auditoría de la v1

### 1. Verificación de Características Propuestas Anteriormente
Todas las etapas originalmente contempladas en el plan inicial (**Etapas 1, 2, 3 y 4**) han sido implementadas en el código base actual:
- **Etapa 1 (Incrustación de Metadatos e Imágenes ID3):** Implementada en el backend Rust (`downloader.rs` con `lofty` y `reqwest`) y frontend (`DownloadsPanel.tsx`).
- **Etapa 2 (Reproductor Interno de Audio):** Implementado en `AudioPlayer.tsx` y conectado a `LocalUtilitiesPanel.tsx`.
- **Etapa 3 (Editor de Etiquetas ID3):** Implementado en `save_track_tags` (Rust) y modal en `LocalUtilitiesPanel.tsx`.
- **Etapa 4 (Selección Manual en YouTube):** Implementado con `search_youtube` (yt-dlp) y modal interactivo en `DownloadsPanel.tsx`.

Debido a que todo el alcance original ha sido cubierto, el documento previo `Por_Implementar.md` ha sido eliminado y reemplazado por este plan **v2**.

---

## ⚠️ Diagnóstico de Problemas Detectados

### 1. Reproductor Interno de Audio (`AudioPlayer.tsx`) - **Prioridad Alta**
- **Problema Actual:** 
  Actualmente, el componente solicita los bytes binarios completos del archivo mediante `invoke("read_file_binary", { path })`. Rust lee todo el archivo a memoria, lo serializa a un array JSON de enteros y se lo envía a Javascript para construir un `Blob` y un `Blob URL`.
- **Impacto:**
  - Para un archivo MP3 o FLAC de 10-30 MB, la serialización de millones de enteros en JSON genera cadenas de ~40-120 MB. Esto provoca picos severos de uso de memoria, ralentización en la interfaz (UI freezing) y fallos de reproducción en archivos medianos/grandes.
  - Los `Blob URL` no admiten peticiones de rango HTTP (HTTP Range headers 206), por lo que el usuario no puede hacer seek/retroceso suave en pistas largas.
- **Solución Planificada (v2):**
  - Configurar las capacidades de Tauri 2 para habilitar `convertFileSrc` (protocolo `asset://`).
  - Sustituir la carga de bytes mediante IPC por `<audio src={convertFileSrc(filePath)}>`. Esto permite reproducción directa por streaming desde el sistema de archivos con cero sobrecarga de memoria IPC JSON y soporte nativo de Range requests.
  - Conectar el `AudioPlayer` a todos los paneles donde se listan canciones (`ComparisonPanel.tsx` para escuchar pistas duplicadas/diferentes y `DownloadsPanel.tsx` para vista previa post-descarga).

### 2. Autenticación de Spotify (Servidor OAuth Callback Local) - **Prioridad Media**
- **Problema Actual:** 
  `start_spotify_auth_callback` intenta hacer binding estático al puerto `8000` (`127.0.0.1:8000`). Si el puerto 8000 está en uso por otro servicio local, el login de Spotify falla con error de socket.
- **Solución Planificada (v2):**
  - Implementar un fallback de puertos en Rust (ej. intentar 8000, 8080, 8888 o solicitar puerto dinámico asignado por el S.O. `0`) y pasar el puerto real obtenido en la URI de callback a la API de Spotify PKCE.

### 3. Detección y Gestión de Motor `yt-dlp` - **Prioridad Media**
- **Problema Actual:**
  El sistema descarga `yt-dlp` al directorio local de la app si no lo encuentra allí. Sin embargo, si el usuario ya tiene `yt-dlp` instalado globalmente en su sistema (en `$PATH`), no se aprovecha.
- **Solución Planificada (v2):**
  - Verificar primero si `yt-dlp` responde globalmente en el sistema antes de iniciar la descarga del binario local.

### 4. Mejoras en el Editor de Metadatos (Tag Editor) - **Prioridad Baja**
- **Problema Actual:**
  El modal actual permite editar Título, Artistas y Álbum, pero no incluye la edición del Año de lanzamiento ni la sustitución manual de la carátula local.
- **Solución Planificada (v2):**
  - Agregar campo para el año (`year`) y selector opcional de imagen local en el modal de edición de metadatos.

---

## 🛠️ Plan de Trabajo para la Versión v2 (Pasos Futuros)

### Paso 1: Refactorización del Reproductor de Audio
1. Actualizar `src-tauri/capabilities/default.json` para permitir la lectura segura de archivos multimedia locales.
2. Modificar `AudioPlayer.tsx` para utilizar `convertFileSrc` de `@tauri-apps/api/core`.
3. Propagar la propiedad `onPlayTrack` a `ComparisonPanel.tsx` y `DownloadsPanel.tsx`.

### Paso 2: Robustez en Autenticación y Descargador
1. Añadir reintentos con puertos alternativos en `lib.rs` para la autenticación de Spotify.
2. Añadir chequeo de `yt-dlp` global en `downloader.rs`.

### Paso 3: Pulido de Interfaz y Experiencia de Usuario (UX)
1. Extender los controles del modal de Tag Editor.
2. Asegurar la respuesta fluida del diseño con el reproductor acoplado en la parte inferior.

---

*Nota: Este documento ha sido elaborado durante la fase de análisis, verificación y pruebas sin modificar el código fuente de la aplicación.*
