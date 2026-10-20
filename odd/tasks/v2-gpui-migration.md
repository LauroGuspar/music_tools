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
- [ ] `TASK-003`: Portado del escáner recursivo de biblioteca y detección de archivos dañados/duplicados (`crates/scanner` con `lofty` y `walkdir`).
- [ ] `TASK-004`: Portado del gestor de descargas asíncrono con yt-dlp y etiquetado ID3 (`crates/downloader` con `tokio`).
- [ ] `TASK-005`: Implementación del cliente de Spotify de alta velocidad con `librespot` y Protobuf (`crates/spotify` + caché SQLite).
- [ ] `TASK-006`: Sistema de diseño e infraestructura UI en GPUI (`crates/ui` con `Table` virtualizada, botones, inputs, tabs, modales, iconos SVG).
- [ ] `TASK-007`: Implementación de Vistas Principales en GPUI (`crates/views` para Omnibar, Mesa de trabajo, Comparador, Descargas y Utilidades).
- [ ] `TASK-008`: Pruebas de integración, pulido visual, atajos de teclado y empaquetado para Linux.

## Evidencia y Progreso
- **Estado:** `TASK-002` completada y verificada.
- **Commit:** `be7e7cf` (`feat(matcher): port fuzzy song matcher engine and enhance storage configuration`).
- **Verificación:** `cargo check --workspace` (0 errores, 0 warnings); `cargo test --workspace` (25 tests pasados: core [3], matcher [16], storage [4], ui [2]).
- **Próximo Paso:** Continuar con `TASK-003`: portar escáner recursivo de biblioteca y detección de archivos dañados/duplicados (`crates/scanner` con `lofty` y `walkdir`).
