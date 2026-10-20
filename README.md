# Music Tools 🎵 (v2.0 - Rust & GPUI)

<p align="center">
  <img src="assets/music-tools.svg" alt="Music Tools Logo" width="128" height="128" />
</p>

<p align="center">
  <strong>Organizador, comparador difuso y gestor de descargas de música 100% nativo en Rust con aceleración por GPU.</strong>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/version-2.0.0-emerald?style=flat-square" alt="Version 2.0.0" />
  <img src="https://img.shields.io/badge/language-Rust_2024-orange?style=flat-square&logo=rust" alt="Rust 2024" />
  <img src="https://img.shields.io/badge/gui-GPUI_120fps-blue?style=flat-square" alt="GPUI" />
  <img src="https://img.shields.io/badge/memory-%3C70MB_RAM-brightgreen?style=flat-square" alt="Memory Consumption" />
  <img src="https://img.shields.io/badge/platform-Linux_(Wayland%2FX11)-purple?style=flat-square" alt="Platform Linux" />
</p>

---

## ⚡ ¿Por qué Music Tools v2.0?

La versión **v2.0** representa una reescritura arquitectónica integral desde cero. Hemos reemplazado por completo el stack híbrido anterior (Tauri + React + WebView) por una **arquitectura 100% nativa en Rust impulsada por GPUI** (el framework de renderizado en GPU desarrollado por Zed Industries):

* **Renderizado Nativo en GPU (120+ FPS):** Desplazamiento ultra fluido en listas de miles de canciones sin sobrecarga del DOM ni recolección de basura de JavaScript.
* **Consumo Ultrabajo de Memoria:** Huella de memoria inferior a **70 MB de RAM** (frente a los ~250 MB requeridos anteriormente por el entorno WebView).
* **Carga en Milisegundos con Protobuf:** Conexión binaria directa con Spotify mediante `librespot` y Protobuf por lotes, superando las limitaciones y latencia de la API REST tradicional.
* **Caché Persistente en SQLite:** Almacenamiento local ultrarrápido con SQLite en modo WAL para apertura instantánea de playlists y configuración sin latencia.
* **Motor Asíncrono Multihilo:** Descargas concurrentes con `yt-dlp`, escaneo de disco en paralelo con `rayon` y comparación difusa en hilos dedicados sin bloquear la interfaz.

---

## 🚀 Características Principales

### 1. Mesa de Trabajo (Staging) y Omnibar
* **Omnibar Inteligente:** Pega cualquier enlace de Spotify (`playlist`, `album`, `track`) o ruta local en el Omnibar superior para procesarla de inmediato.
* **Tabla Virtualizada de Alto Rendimiento:** Visualiza colecciones de decenas de miles de pistas con ordenamiento por título, artista, álbum o duración.
* **Exportación Rápida:** Transfiere pistas seleccionadas o la lista completa a la cola de descargas con un solo clic o atajo.

### 2. Motor de Comparación Difusa (Sørensen-Dice) 🔍
Permite contrastar tu biblioteca física local con cualquier playlist o archivo de exportación para detectar qué canciones te faltan:
* **Normalización Avanzada de Metadatos:** Eliminación inteligente de acentos/diacríticos, mayúsculas, signos de puntuación y etiquetas redundantes (`Remastered`, `Live`, `Deluxe`, `Bonus Track`, etc.).
* **Similitud Sørensen-Dice:** Algoritmo de bigramas con umbral difuso personalizable (por defecto 80%), capaz de identificar coincidencias exactas y probables incluso ante ligeras variaciones de nombre.
* **Exportación Directa de Faltantes:** Envía automáticamente todas las canciones que faltan en tu disco a la cola de descargas.

### 3. Gestor de Descargas Asíncrono Concurrente 📥
* **Descarga Paralela:** Procesa descargas concurrentes en segundo plano configurables (hasta 3 de manera simultánea por defecto).
* **Supervisión en Tiempo Real:** Monitorea el porcentaje exacto, velocidad de transferencia (KB/MB) y tiempo restante estimado (ETA).
* **Omitido Automático de Existentes:** Detecta canciones que ya existen en el disco destino para evitar consumo innecesario de ancho de banda.
* **Inyección de Etiquetas ID3:** Etiquetado automático de metadatos (título, artistas, álbum, año) mediante la biblioteca nativa `lofty`.

### 4. Utilidades de Biblioteca Local 🧹
* **Duplicados Exactos:** Detecta archivos idénticos por tamaño en bytes exacto y cálculo opcional de hash criptográfico SHA-256.
* **Duplicados Semánticos:** Identifica pistas repetidas en distintos formatos (ej. `.mp3` vs `.flac` o distintas versiones del mismo tema).
* **Detección de Archivos Dañados:** Encuentra archivos vacíos (`0 bytes`) o con cabeceras de audio corruptas que no pueden ser decodificadas.
* **Acciones Rápidas:** Copiar ruta, abrir en explorador y eliminación definitiva con modal de confirmación seguro.

---

## ⌨️ Atajos de Teclado Globales

Music Tools está diseñado con un flujo de trabajo keyboard-first:

| Atajo | Acción | Descripción |
| :--- | :--- | :--- |
| `Ctrl + 1` | **Mesa de Trabajo** | Cambia a la vista de Staging y carga de playlists |
| `Ctrl + 2` | **Comparador** | Abre la vista de comparación difusa |
| `Ctrl + 3` | **Descargas** | Accede a la cola de descargas activas e historial |
| `Ctrl + 4` | **Utilidades** | Abre el analizador de duplicados y archivos dañados |
| `Ctrl + 5` / `Ctrl + ,` | **Ajustes** | Abre el panel de preferencias y configuración |
| `Escape` | **Cancelar / Salir** | Cierra diálogos modales abiertos o deselecciona filas activas |

*(En macOS, las combinaciones admiten de forma nativa la tecla `Cmd` en lugar de `Ctrl`).*

---

## 📂 Arquitectura del Espacio de Trabajo (Cargo Workspace)

El proyecto está estructurado como un espacio de trabajo modular en Rust:

```text
music_tools/
├── crates/
│   ├── app/           # Punto de entrada binario, ventana GPUI y registro de atajos globales
│   ├── core/          # Modelos de dominio puros (Track, Playlist, DownloadItem, estados)
│   ├── storage/       # Capa de persistencia y caché local con SQLite (rusqlite)
│   ├── matcher/       # Motor de comparación difusa y normalización Sørensen-Dice
│   ├── scanner/       # Escáner de biblioteca en disco con Rayon y Lofty
│   ├── downloader/    # Gestor de descargas asíncrono con Tokio y yt-dlp
│   ├── spotify/       # Cliente Spotify de alta velocidad con librespot y Protobuf
│   ├── ui/            # Sistema de componentes GPUI (Tabla virtualizada, botones, modales, pestañas)
│   └── views/         # Pantallas principales (Staging, Compare, Downloads, Utilities, Settings)
├── assets/            # Iconos vectoriales SVG e integración de escritorio Linux (.desktop)
├── scripts/           # Scripts de compilación y empaquetado para distribución en Linux
└── tests/             # Suite de pruebas de integración completa de extremo a extremo (E2E)
```

---

## 🛠️ Requisitos de Compilación en Linux

Para compilar Music Tools v2.0 desde el código fuente, tu sistema requiere el compilador de Rust (edición 2024), las herramientas de compilación estándar de C y las bibliotecas de gráficos/audio de Linux:

### CachyOS / Arch Linux
```bash
sudo pacman -S --needed base-devel rustup fontconfig alsa-lib libxkbcommon wayland libx11 pkgconf
rustup default stable
```

### Ubuntu / Debian / Pop!_OS
```bash
sudo apt update
sudo apt install build-essential pkg-config libfontconfig1-dev libasound2-dev libxkbcommon-dev libwayland-dev libx11-dev
```

### Fedora / RHEL
```bash
sudo dnf install @development-tools fontconfig-devel alsa-lib-devel libxkbcommon-devel wayland-devel libX11-devel
```

### Herramientas Opcionales recomendadas en tiempo de ejecución
* **yt-dlp:** Para descarga de audio (`sudo pacman -S yt-dlp` o `pip install yt-dlp`). Si no está instalado, la aplicación puede descargarlo de manera autónoma.
* **ffmpeg:** Para conversión de formatos y metadatos (`sudo pacman -S ffmpeg` o `sudo apt install ffmpeg`).

---

## 🏃 Compilación y Ejecución

### 1. Ejecutar en modo desarrollo
```bash
cargo run -p app
```

### 2. Ejecutar la suite completa de pruebas
```bash
cargo test --workspace
```
*(Incluye todas las pruebas unitarias de los 9 crates y la prueba de integración de extremo a extremo `tests/e2e_flow_test.rs`).*

### 3. Generar empaquetado optimizado para Linux (Release con LTO)
Hemos preparado un script automatizado que verifica dependencias, compila con optimizaciones Link-Time (LTO), remueve símbolos de depuración (`strip`) y genera la estructura portable lista para ejecutar:

```bash
./scripts/build-linux.sh
```

El paquete generado se ubicará en `target/dist-linux/`:
* `target/dist-linux/bin/music-tools`: Binario nativo optimizado.
* `target/dist-linux/share/applications/music-tools.desktop`: Acceso directo para el menú de aplicaciones de tu entorno de escritorio.
* `target/dist-linux/share/icons/hicolor/scalable/apps/music-tools.svg`: Icono vectorial en alta resolución.
* `target/dist-linux/run.sh`: Lanzador ejecutable directo.

---

## 🗺️ Hoja de Ruta Multiplataforma

* [x] **v2.0 (Linux Wayland / X11):** Migración 100% Rust + GPUI completada, pruebas E2E y empaquetado inicial.
* [ ] **v2.1 (Windows):** Backend de renderizado nativo Direct3D/Vulkan en GPUI e instalador MSIX.
* [ ] **v2.2 (macOS):** Backend de renderizado nativo Metal en GPUI y empaquetado Universal `.dmg`.
* [ ] **v2.3 (Sincronización Bidireccional):** Exportación directa hacia dispositivos locales y reproductores DAP.

---

## 🏷️ Licencia
Distribuido bajo la licencia MIT o Apache-2.0.
