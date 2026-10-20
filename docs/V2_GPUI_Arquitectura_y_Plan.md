# 🏗️ Plan Maestro de Arquitectura y Migración a Rust + GPUI (v2.0)

Este documento define la arquitectura técnica, la estrategia de migración incremental y el diseño del sistema para la versión **v2.0** de **Music Tools**, reescribiendo la aplicación completamente en **Rust nativo** utilizando el framework de interfaz gráfica acelerada por GPU **GPUI** (inspirado en la arquitectura modular de *Sonora* y *Zed*).

---

## 📌 Tabla de Contenidos

1. [Visión y Principios del Proyecto](#1-visión-y-principios-del-proyecto)
2. [Estrategia de Plataformas: Linux-First](#2-estrategia-de-plataformas-linux-first)
3. [Comparativa de Arquitectura: v1 vs v2](#3-comparativa-de-arquitectura-v1-vs-v2)
4. [Estructura del Cargo Workspace (Crates Modulares)](#4-estructura-del-cargo-workspace-crates-modulares)
5. [Conservación y Potenciación de Funcionalidades Existentes](#5-conservación-y-potenciación-de-funcionalidades-existentes)
6. [Plan de Migración por Fases Incrementales](#6-plan-de-migración-por-fases-incrementales)
7. [Consideraciones Técnicas y Próximos Pasos](#7-consideraciones-técnicas-y-próximos-pasos)

---

## 1. Visión y Principios del Proyecto

* **Rendimiento Extremo (120+ FPS sin DOM):** Eliminación completa del motor WebKit/Chromium y capas de JavaScript. La interfaz se renderiza directamente en la GPU mediante shaders nativos.
* **Consumo Mínimo de Recursos:** Reducción de la huella de memoria de ~250MB a menos de 60-80MB en reposo.
* **Carga Instantánea de Metadatos:** Conexión con Spotify mediante el protocolo binario **Protobuf** (`librespot`), peticiones por lotes de hasta 500 elementos y persistencia en **SQLite local**.
* **Enfoque Foco en Organización y Utilidades:** A diferencia de Sonora (que es exclusivamente un reproductor/streamer y prohíbe descargas), Music Tools mantiene como pilar central:
  * Comparación de playlists remotas vs. carpetas locales mediante similitud difusa.
  * Descargas concurrentes de alta velocidad (`yt-dlp` + `ffmpeg`) con inyección de tags ID3/Lofty.
  * Mantenimiento de biblioteca local (detección de duplicados exactos y semánticos, detección de archivos corruptos).

---

## 2. Estrategia de Plataformas: Linux-First

### 🐧 Fase Actual: Optimizado para Linux
La versión inicial de v2 se construirá y probará en **Linux** (con soporte nativo para **Wayland** y **X11**).
* **Renderizado:** GPUI utiliza Wayland (`wayland-client`) o X11 (`libxcb`, `libxkbcommon`, `libx11`) y drivers Vulkan / Mesa.
* **Sonido:** Integración directa con ALSA y PipeWire.
* **Empaquetado:** Binario directo, script de desarrollo, AppImage y Flatpak.

### 🌐 Hoja de Ruta Cross-Platform (Futuro)
La estructura modular del workspace aislará el código específico del sistema operativo usando condicionales de compilación `#[cfg(target_os = "...")]`:
* **Windows:**
  * Renderizado mediante DirectX / DirectWrite / WGL.
  * Integración con WebView2 solo si se requieren diálogos de login web.
* **macOS:**
  * Renderizado mediante Metal nativo.
  * Soporte de menús y barras de título de Cocoa/AppKit (`objc2`).

---

## 3. Comparativa de Arquitectura: v1 vs v2

| Componente | Versión 1 (Actual) | Versión 2 (Nueva - GPUI) | Beneficio Clave |
| :--- | :--- | :--- | :--- |
| **Interfaz Gráfica** | React 19 + TypeScript + Vite | **GPUI (Rust puro)** | Renderizado en GPU a 120 FPS, 0 ms de overhead de DOM |
| **Contenedor** | Tauri v2 (WebKitGTK en Linux) | **Binario Nativo Autónomo** | Sin dependencias de navegadores web |
| **Tablas / Listas** | HTML Table / CSS | **Virtual Table (GPUI)** | Solo se renderizan ~25 filas en pantalla, soporte para 100K tracks |
| **API de Spotify** | Spotify Web API (REST/JSON) | **Librespot (Protobuf binario)** | Lotes de 500 canciones, respuestas en ms, sin rate limits agresivos |
| **Caché de Listas** | Memoria volátil del navegador | **SQLite local (`cache.sqlite`)** | Carga instantánea sin re-descargar metadatos |
| **Motor de Matching** | TypeScript (Sørensen-Dice en JS) | **Rust Multihilo (`rayon`)** | Comparación de miles de temas en <20ms |
| **Escáner de Disco** | Rust secuencial en Tauri | **Rust Paralelo (`rayon` + `walkdir`)** | Escaneo de terabytes de música en segundos |
| **Descargas MP3** | Proceso Tauri invocando `yt-dlp` | **`tokio::process` con canales async** | Monitoreo de progreso en tiempo real sin serialización IPC |

---

## 4. Estructura del Cargo Workspace (Crates Modulares)

Siguiendo el diseño de arquitectura limpia de *Sonora*:

```text
music_tools/
├── Cargo.toml               # Workspace manifest (edition 2024, resolver = "3")
├── crates/
│   ├── app/                 # Binario principal: inicialización de GPUI, ventana, ciclo de vida
│   ├── ui/                  # Componentes reutilizables: botones, tablas virtualizadas, tabs, modales, iconos SVG
│   ├── views/               # Pantallas y vistas de negocio:
│   │   ├── omnibar/         # Barra superior de búsqueda y parseo de URLs
│   │   ├── staging/         # Mesa de trabajo y previsualización de tracks
│   │   ├── compare/         # Panel de comparación (Carpeta vs Playlist, etc.)
│   │   ├── downloads/       # Panel y cola de descargas concurrentes
│   │   └── utilities/       # Inspector de duplicados y archivos corruptos
│   ├── core/                # Modelos de datos puros (Track, Album, Playlist, DownloadItem, etc.)
│   ├── spotify/             # Cliente Spotify rápido (librespot, OAuth PKCE, Protobuf)
│   ├── downloader/          # Orquestador yt-dlp con tokio, ETA, progreso y tagging ID3 (Lofty)
│   ├── matcher/             # Algoritmo de similitud difusa (Sørensen-Dice), normalizador de metadatos
│   ├── scanner/             # Escáner de sistema de archivos recursivo y lectura de tags
│   └── storage/             # Conexión y esquemas de SQLite (cache.sqlite y config.sqlite)
```

---

## 5. Conservación y Potenciación de Funcionalidades Existentes

1. **Gestor de Descargas Concurrentes:**
   * Motor `yt-dlp` gestionado con `tokio::process::Child`.
   * Límite de 3 descargas simultáneas configurables.
   * Parseo en tiempo real del porcentaje, velocidad (KiB/s, MiB/s) y tiempo estimado (ETA).
   * Inyección automática de etiquetas ID3v2 completas con carátula y número de pista vía `lofty`.
   * Reintento automático de fallidas.

2. **Motor de Comparación Difusa (Fuzzy Matcher):**
   * Normalización estricta: minúsculas, diacríticos/acentos, caracteres especiales eliminados.
   * Eliminación de términos de ruido: `remaster`, `deluxe`, `live`, `bonus`, `mono`, `stereo`, `mix`, etc.
   * Cálculo de coeficientes de similitud bigrama (Sørensen-Dice) parametrizable mediante slider en la interfaz.

3. **Utilidades Locales de Biblioteca:**
   * **Duplicados exactos:** Detección instantánea por tamaño en bytes exacto y hash rápido.
   * **Duplicados semánticos:** Agrupación difusa por título y artista similar entre formatos (.mp3, .flac, .m4a).
   * **Archivos dañados:** Verificación de cabeceras de audio con `lofty` y detección de archivos vacíos (0 bytes).
   * Acciones: Abrir archivo, reproducir localmente, copiar ruta y eliminar archivo con confirmación.

4. **Exportación e Importación:**
   * Exportación a formatos estructurados `.json` y listas `.txt`.
   * Importación de archivos `.json` y `.txt` previamente guardados a la mesa de trabajo o a la cola de descargas.

---

## 6. Plan de Migración por Fases Incrementales

### 🔹 Fase 0: Inicialización del Workspace y Entorno Base
* Configuración del `Cargo.toml` raíz con edición 2024 y resolver 3.
* Creación de los crates base: `core`, `storage`, `ui`, `app`.
* Verificación de la compilación y apertura de una ventana básica vacía con GPUI en Linux (Wayland/X11).

### 🔹 Fase 1: Motores de Dominio (Sin UI aún)
* Migración y optimización de `crates/matcher`: tests unitarios de similitud difusa portados de `songMatcher.test.ts`.
* Portado y optimización de `crates/scanner`: escaneo de archivos locales con `lofty` y `walkdir`.
* Portado de `crates/downloader`: wrapper asíncrono de `yt-dlp` con inyección de tags.
* Implementación de `crates/storage`: base de datos SQLite para caché y configuración.

### 🔹 Fase 2: Conexión con Spotify de Alta Velocidad
* Integración de `librespot-core`, `librespot-oauth` y `librespot-protocol`.
* Flujo de inicio de sesión con PKCE y servidor TCP local en puerto de callback.
* Obtención de metadatos en lotes de 500 canciones (Protobuf).
* Almacenamiento en caché de playlists en SQLite para carga instantánea.

### 🔹 Fase 3: Sistema de Diseño y Componentes GPUI
* Configuración de paleta de colores oscuros con acentos esmeralda/verde Spotify.
* Tipografía y escalado de métricas DPI.
* Componente `Table` virtualizado (overscan de 2-3 filas) para listas de reproducción infinitas.
* Componentes base: `Button`, `Input`, `Tabs`, `ProgressBar`, `Modal`, `Icon` (SVGs embebidos).

### 🔹 Fase 4: Vistas y Experiencia de Usuario
* **Omnibar & Staging Workspace:** Búsqueda rápida, inserción de URLs (Spotify / YouTube / Archivos) y tabla de previsualización.
* **Panel de Comparación:** Selector de fuentes (Carpeta vs Playlist), tabla comparativa de canciones encontradas y faltantes.
* **Panel de Descargas:** Cola interactiva, estados (en cola, descargando, completada, fallida, cancelada) y progreso en vivo.
* **Panel de Utilidades Locales:** Resultados de escaneo, grupos de duplicados y lista de archivos dañados con acciones directas.
* **Diálogo de Configuración:** Rutas por defecto, umbral difuso y preferencias generales.

### 🔹 Fase 5: Pruebas, Optimización y Empaquetado
* Pruebas de estrés con bibliotecas de +20,000 canciones.
* Generación de paquetes para Linux: Binario optimizado con LTO, AppImage y Flatpak.
* Documentación de usuario final y guía de compilación.

---

## 7. Consideraciones Técnicas y Próximos Pasos

1. **Ramas Git:**
   * `legacy/v0.2-tauri-react`: Conserva de forma inmutable la versión 0.2 basada en React y Tauri.
   * `feature/v2-gpui`: Rama de trabajo activa donde se construirá la nueva arquitectura en Rust y GPUI.
2. **Dependencias del Sistema en Linux (CachyOS / Arch):**
   * GPUI requiere que el sistema cuente con `fontconfig`, `libxkbcommon`, `wayland`, `vulkan-icd-loader` y `alsa-lib`. Dado que Sonora ya compila y corre en este entorno, la compatibilidad del entorno local está 100% verificada.
