# Music Tools 🎵 (Spotify & Local Music Utilities)

**Music Tools** es una aplicación de escritorio multiplataforma (desarrollada para Linux y Windows) que permite conectar tu cuenta de Spotify, gestionar tus playlists locales y remotas, y mantener organizada tu biblioteca de música física en el disco duro.

Está construida sobre **Tauri v2**, **React 19**, **TypeScript** y **Rust**, ofreciendo una interfaz de usuario fluida y moderna de alto rendimiento con acceso nativo al sistema de archivos.

---

## 🚀 Características Principales

### 1. Integración con Spotify (OAuth2 PKCE)
* **Autenticación Segura:** Inicio de sesión seguro con Spotify utilizando el flujo de código de autorización con **PKCE** (sin exponer claves secretas).
* **Servidor Callback de Rust:** El backend en Rust levanta un servidor TCP local temporal en el puerto `8000` para capturar de forma nativa el callback de Spotify.
* **Carga de Playlists:** Descarga playlists públicas, privadas y colaborativas ingresando la URL completa o el ID.
* **Paginación Automática:** Carga completa de listas de reproducción de cualquier tamaño utilizando la API oficial de Spotify.

### 2. Exportación de Listas de Reproducción
* **Formatos Compatibles:** Exporta los metadatos de tus playlists en formato **JSON** (estructurado) o **TXT** (lista de texto plano).
* **Diálogo Nativo:** Utiliza el diálogo nativo del sistema para seleccionar la ruta exacta de guardado.

### 3. Motor de Comparación de Música 🔍
Permite comparar música en tres modalidades distintas para identificar canciones faltantes o diferencias:
1. **Carpeta local vs. Playlist:** Compara un directorio de música local contra una playlist cargada o un archivo exportado anteriormente. Muy útil para saber qué canciones te faltan descargar de una playlist de Spotify.
2. **Playlist vs. Playlist:** Compara dos archivos de playlists exportadas (.json o .txt) para encontrar adiciones o eliminaciones.
3. **Carpeta vs. Carpeta:** Compara dos directorios de tu disco para sincronizar contenidos.

> [!NOTE]
> **Heurísticas del Motor de Búsqueda:**
> Las canciones suelen tener títulos ligeramente diferentes en formato local y en Spotify (ej. *Let It Be - Remastered 2009* vs. *Let It Be*).
> Nuestro motor realiza una **normalización de metadatos** (conversión a minúsculas, remoción de acentos/diacríticos, eliminación de caracteres especiales) y **limpieza de palabras clave** (*remaster*, *live*, *deluxe*, *bonus*, *mono/stereo*, *acoustic*, etc.). 
> Luego aplica un algoritmo de **similitud de bigramas (Sørensen-Dice)** para identificar coincidencias exactas o probables. El umbral de precisión difusa es completamente personalizable.

### 4. Utilidades de Disco Local (Limpieza de Biblioteca) 🧹
* **Detector de Duplicados:**
  * **Duplicados Exactos:** Identifica archivos idénticos en el disco basándose en su tamaño en bytes exacto (rápido y eficiente).
  * **Duplicados Semánticos:** Identifica archivos que corresponden al mismo tema musical mediante metadatos y lógica difusa (por ejemplo, comparar un archivo `.mp3` y un archivo `.flac` del mismo tema).
* **Detector de Archivos Dañados o Corruptos:**
  * Encuentra archivos vacíos (`0 bytes`).
  * Utiliza la biblioteca **Lofty** en Rust para leer cabeceras y etiquetas ID3v1, ID3v2, Vorbis, etc. Si el archivo está incompleto o su cabecera está rota, se marca como dañado/corrupto indicando el motivo técnico del error.
* **Acciones Integradas:**
  * ▶️ **Reproducir:** Abre y reproduce el archivo local directamente en el reproductor de audio por defecto de tu sistema mediante el plugin nativo de Tauri.
  * 📋 **Copiar Ruta:** Copia la ruta absoluta del archivo en el portapapeles.
  * 🗑️ **Eliminar:** Elimina permanentemente el archivo seleccionado del disco duro (con confirmación de seguridad).

### 5. Configuración General ⚙️
* Configura carpetas por defecto para música local y descargas.
* Ajusta el slider de **Umbral de Similitud Difusa** (Fuzzy Match Threshold) para afinar qué tan estricta debe ser la coincidencia al comparar títulos y artistas.

---

## 🛠️ Stack Tecnológico

* **Frontend:**
  * [React 19](https://react.dev/)
  * [TypeScript](https://www.typescriptlang.org/)
  * [Vite](https://vite.dev/) (Empaquetador rápido y moderno)
  * [Vanilla CSS](https://developer.mozilla.org/es/docs/Web/CSS) con una interfaz oscura (dark mode) estilizada y responsive.
  * [Vitest](https://vitest.dev/) para pruebas unitarias.
* **Backend (Escritorio):**
  * [Tauri v2](https://v2.tauri.app/) (Framework ligero de aplicaciones de escritorio).
  * [Rust](https://www.rust-lang.org/) para acceso de bajo nivel al sistema de archivos, lectura de metadatos de audio con la librería [Lofty](https://crates.io/crates/lofty), y servidor de autenticación local.

---

## 📂 Estructura del Proyecto

* [src/](file:///home/rvelasco/Proyectos/personales/spotify_tools/src) - Código del Frontend (React + TS):
  * [components/](file:///home/rvelasco/Proyectos/personales/spotify_tools/src/components) - Componentes de la interfaz de usuario (Diálogo de ajustes, tablas, paneles de comparación y utilidades).
  * [features/](file:///home/rvelasco/Proyectos/personales/spotify_tools/src/features) - Lógica de negocio (módulos de conexión de Spotify, motor de comparación en memoria y formateo de exportación).
  * [types/](file:///home/rvelasco/Proyectos/personales/spotify_tools/src/types) - Interfaces y tipos comunes compartidos.
  * [utils/](file:///home/rvelasco/Proyectos/personales/spotify_tools/src/utils) - Funciones de ayuda puras (algoritmo de comparación, normalizadores y gestión de configuraciones locales mediante `localStorage`).
* [src-tauri/](file:///home/rvelasco/Proyectos/personales/spotify_tools/src-tauri) - Backend en Rust (Tauri):
  * [src/lib.rs](file:///home/rvelasco/Proyectos/personales/spotify_tools/src-tauri/src/lib.rs) - Orquestador y registro de los comandos Tauri expuestos al frontend.
  * [src/scanner.rs](file:///home/rvelasco/Proyectos/personales/spotify_tools/src-tauri/src/scanner.rs) - Motor de lectura del sistema de archivos recursivo y extracción de etiquetas usando la biblioteca Lofty.
  * [capabilities/](file:///home/rvelasco/Proyectos/personales/spotify_tools/src-tauri/capabilities) - Definición de los permisos del sistema de archivos y plugins para Tauri v2.

---

## ⚙️ Configuración y Configuración de Spotify

Para poder cargar listas de reproducción desde Spotify, la aplicación requiere un **Client ID** de Spotify.

### 1. Registrar tu aplicación en Spotify Developers
1. Ve al [Spotify Developer Dashboard](https://developer.spotify.com/dashboard).
2. Inicia sesión y haz clic en **Create app**.
3. Configura los siguientes campos:
   * **App name:** Music Tools (o el de tu preferencia).
   * **Redirect URIs:** Debes agregar **exactamente** la siguiente dirección URL:
     ```text
     http://127.0.0.1:8000/callback
     ```
     *(Nota: No uses `localhost` en su lugar. Spotify requiere explícitamente la dirección IP loopback `127.0.0.1`).*
4. Guarda los cambios y copia el **Client ID** que te proporciona Spotify.

### 2. Configurar variables de entorno
En el directorio raíz del proyecto, renombra el archivo `.env.example` a `.env` y coloca tu Client ID:

```env
VITE_SPOTIFY_CLIENT_ID=tu_spotify_client_id_aqui
```

---

## 🏃 Cómo Ejecutar y Construir la Aplicación

### Requisitos Previos
* **Node.js** (v18 o superior) y `npm`.
* Herramientas de **Rust** (compilador rustc y cargo). Sigue las instrucciones de instalación en [rustup.rs](https://rustup.rs/).
* Dependencias del sistema recomendadas para el desarrollo con Tauri en tu sistema operativo (puedes consultar la [Guía de Prerrequisitos de Tauri](https://v2.tauri.app/start/prerequisites/)).

### Pasos de Instalación

1. Clona este repositorio y navega al directorio del proyecto.
2. Instala las dependencias del frontend:
   ```bash
   npm install
   ```

### Ejecutar en modo Desarrollo
Para levantar el servidor de desarrollo de Vite y abrir la ventana nativa de Tauri:
```bash
npm run dev
```
O bien mediante el CLI de Tauri directamente si lo tienes global:
```bash
npm run tauri dev
```

### Ejecutar las Pruebas Unitarias
Para correr los tests del frontend con Vitest:
```bash
npm run test
```

### Compilar para Producción
Para generar el ejecutable nativo optimizado (`.deb`, `.appImage` en Linux o `.msi`, `.exe` en Windows):
```bash
npm run build
```
*(Los instaladores compilados se guardarán en la carpeta `src-tauri/target/release/bundle/`).*

---

## 🏷️ Licencia
Este es un proyecto personal. Siéntete libre de clonarlo, usarlo y proponer mejoras.
