# Feature: Migración y Rediseño Completo a Rust + GPUI (v2.0)

## Objetivo
Rediseñar por completo la aplicación **Music Tools**, migrando de la arquitectura híbrida (Tauri v2 + React 19 + TypeScript) a una arquitectura **100% nativa en Rust con GPUI** (el framework de GPU de Zed Industries utilizado por Sonora), optimizando la velocidad de carga de listas mediante el protocolo binario de Spotify (`librespot` + Protobuf) y caché local en SQLite, manteniendo todas las funcionalidades existentes (descargas concurrentes con yt-dlp, comparación difusa, utilidades de biblioteca local y exportación), enfocado inicialmente en Linux con Wayland/X11.

## Justificación
La versión actual utiliza un WebView que consume ~250MB de memoria, tiene latencia en listas largas por la sobrecarga del DOM y depende de llamadas REST a la API de Spotify con límites de paginación lentos. La migración a Rust + GPUI permite renderizado directo a 120+ FPS en GPU, consumo inferior a 70MB, carga de playlists masivas en milisegundos mediante Protobuf por lotes y ejecución multihilo nativa para escaneo y comparación de colecciones de audio.

## Alcance
- **Plataforma inicial:** Linux (Wayland / X11). La arquitectura aislará dependencias de sistema para permitir soporte futuro a Windows y macOS.
- **Enfoque de producto:** Herramienta de organización, comparación, escaneo local y descargas concurrentes. (No incluye emulación de reproductor completo de streaming de Sonora).
- **Ramas Git:**
  - `legacy/v0.2-tauri-react`: Conserva intacta la versión v0.2.x de Tauri + React.
  - `feature/v2-gpui`: Rama activa de desarrollo para la v2.

## Restricciones
- Conservar todas las características existentes: descargas concurrentes con yt-dlp, tags con lofty, fuzzy matching Sørensen-Dice, duplicados exactos/semánticos, archivos corruptos y exportación.
- Implementación incremental por fases para garantizar estabilidad sin sobrecargas.
- TDD habilitado en módulos de lógica pura (matcher, scanner, storage, downloader).

## Tareas

- [x] `TASK-001`: Configuración del Cargo Workspace base (`crates/core`, `crates/storage`, `crates/ui`, `crates/app`) y ventana inicial GPUI en Linux. (Commit: `b72d036b67106232471a92bacae35a6b5fec2d9e`)
- [x] `TASK-002`: Portado y optimización de motores de dominio a Rust puro (`crates/matcher` con tests de Sørensen-Dice y `crates/storage` con SQLite). (Commit: `be7e7cf4e257969baaa4afdbecb9c81d6bf29b08`)
- [x] `TASK-003`: Portado del escáner recursivo de biblioteca y detección de archivos dañados/duplicados (`crates/scanner` con `lofty` y `walkdir`). (Commit: `c3526caf6553f2f7161efe28de0cc9f710532632`)
- [x] `TASK-004`: Portado del gestor de descargas asíncrono con yt-dlp y etiquetado ID3 (`crates/downloader` con `tokio`). (Commit: `9ad9abd4350404081afc4ffdb7c92b1c661452c7`)
- [x] `TASK-005`: Implementación del cliente de Spotify de alta velocidad con `librespot` y Protobuf (`crates/spotify` + caché SQLite). (Commit: `337eaa9e078b48921134339e654577da2b6bdc12`)
- [x] `TASK-006`: Sistema de diseño e infraestructura UI en GPUI (`crates/ui` con `Table` virtualizada, botones, inputs, tabs, modales, iconos SVG). (Commit: `dee8b12fe29b21a57fa557d8d2da947780a41e99`)
- [x] `TASK-007`: Implementación de Vistas Principales en GPUI (`crates/views` para Omnibar, Mesa de trabajo, Comparador, Descargas y Utilidades). (Commit: `da897d5a552860acf70f003917cebea530658bd4`)
- [x] `TASK-008`: Pruebas de integración, pulido visual, atajos de teclado y empaquetado para Linux. (Commit: `b01ca8317673579ceb17028823344e1262e8a5a4`)
- [x] `TASK-009`: Carga real de pistas en Staging Omnibar (Spotify Embed/Librespot, YouTube extractor con yt-dlp, carpetas locales con expansión `~`).
- [x] `TASK-010`: Selector nativo de carpetas (`rfd` / Zenity) para Configuración, Comparador y Utilidades.
- [x] `TASK-011`: Escaneo real asíncrono en Comparador y Utilidades con tilde expansion y visualización reactiva de resultados.
- [x] `TASK-012`: Sistema de temas dinámico en tiempo real y completitud de opciones de Configuración con persistencia SQLite.
- [x] `TASK-013`: Pruebas de integración visual y validación exhaustiva de experiencia de usuario en Linux.

## Evidencia y Progreso
- **Estado:** Completado - Interactividad completa verificada, cargas reales probadas, selectores de carpetas nativos, motor de temas dinámico y 107 tests pasando.
- **Último Commit:** `7580e0e` (preparando work-unit commit para TASK-009..TASK-013).
- **Verificación:** `cargo check --workspace` (0 errores, 0 warnings); `cargo test --workspace` (107 tests pasados, 0 fallos, 0 emojis).
- **Próximo Paso:** Commit work unit en rama `feature/v2-gpui` y entrega al usuario.
