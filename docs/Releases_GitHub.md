# Guía para Publicar "Releases" en GitHub con Tauri

Como esta aplicación está construida sobre **Tauri**, se compila directamente a binarios nativos del sistema operativo (un `.exe` para Windows, un `.deb` o `.AppImage` para Linux, y un `.dmg` o `.app` para macOS).

Para publicar estos archivos compilados en la sección de **Releases** (Lanzamientos) de tu repositorio de GitHub, hay dos métodos: el **método automático (Recomendado con GitHub Actions)** y el **método manual (Compilar localmente y subir)**.

---

## 🚀 Método Automático: GitHub Actions (Recomendado)

Podemos configurar un flujo automatizado en GitHub (CI/CD) para que cada vez que crees una versión o "etiqueta" (tag) en tu repositorio (por ejemplo, `v0.1.0`), GitHub compile la aplicación para todos los sistemas operativos en sus servidores y cree el Release automáticamente con los archivos instaladores adjuntos.

### 1. Configurar el Workflow de GitHub
Para configurar esto, debes crear un archivo llamado `.github/workflows/release.yml` en la raíz del proyecto con la siguiente estructura:

```yaml
name: "Publish Release"

on:
  push:
    tags:
      - "v*" # Se activa al hacer push a un tag de versión, ej: v0.1.0

jobs:
  publish-tauri:
    permissions:
      contents: write
    strategy:
      fail-fast: false
      matrix:
        include:
          - platform: "macos-latest" # Para compilar .dmg
            args: "--target universal-apple-darwin"
          - platform: "ubuntu-22.04" # Para compilar .deb y .AppImage
            args: ""
          - platform: "windows-latest" # Para compilar .exe y .msi
            args: ""

    runs-on: ${{ matrix.platform }}
    steps:
      - name: Checkout repository
        uses: actions/checkout@v4

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: 20
          cache: "npm"

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          # Necesitamos compilar a Apple Silicon y Intel en Mac
          targets: ${{ matrix.platform == 'macos-latest' && 'aarch64-apple-darwin,x86_64-apple-darwin' || '' }}

      - name: Install Linux dependencies (Ubuntu only)
        if: matrix.platform == 'ubuntu-22.04'
        run: |
          sudo apt-get update
          sudo apt-get install -y libwebkit2gtk-4.1-dev build-essential curl wget file libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev

      - name: Install frontend dependencies
        run: npm install

      # Tauri Action se encarga de compilar y subir los assets al Release
      - name: Build and Publish
        uses: tauri-apps/tauri-action@v0
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        with:
          tagName: ${{ github.ref_name }}
          releaseName: "Music Tools ${{ github.ref_name }}"
          releaseBody: "Instaladores oficiales de Music Tools para Windows, macOS y Linux."
          releaseDraft: true # Lo crea como borrador para que lo revises antes de publicar
          prerelease: false
          args: ${{ matrix.args }}
```

### 2. ¿Cómo publicar una nueva versión con este flujo?
Una vez creado el archivo en la rama principal en GitHub, cada lanzamiento se realiza siguiendo estos pasos:

1. Actualiza el número de versión en `src-tauri/tauri.conf.json` y `package.json` (por ejemplo, a `"0.1.0"`).
2. Haz commit y súbelo a GitHub:
   ```bash
   git commit -am "bump: versión 0.1.0"
   git push origin main
   ```
3. Crea y sube la etiqueta de versión (tag):
   ```bash
   git tag v0.1.0
   git push origin v0.1.0
   ```
4. **¡Listo!** GitHub Actions detectará la etiqueta, comenzará a compilar para los tres sistemas operativos (tardará aproximadamente de 5 a 10 minutos) y generará un borrador de **Release** en tu repositorio con los archivos adjuntos.

---

## 💻 Método Manual: Compilar localmente y subir

Si no deseas configurar automatizaciones en la nube, puedes generar los instaladores tú mismo en tu propia computadora y subirlos a la web de GitHub de forma manual.

### Pasos para compilar en tu PC:

1. **Compilar en modo producción:**
   Ejecuta el siguiente comando en la terminal de tu proyecto:
   ```bash
   npm run tauri build
   ```
   *Nota: Esto generará el instalador correspondiente a tu sistema operativo actual.*

2. **Ubicación de los archivos generados:**
   Los archivos compilados listos para distribuir se guardan en la carpeta:
   - **En Windows:** `src-tauri/target/release/bundle/msi/` y `.../nsis/` (archivos `.msi` y `.exe`).
   - **En Linux:** `src-tauri/target/release/bundle/deb/` y `.../appimage/` (archivos `.deb` y `.AppImage`).
   - **En macOS:** `src-tauri/target/release/bundle/dmg/` y `.../macos/` (archivos `.dmg` y `.app`).

3. **Subir a GitHub:**
   - Ve a tu repositorio en GitHub.
   - En la barra lateral derecha, haz clic en **Releases** -> **Create a new release** (o "Draft a new release").
   - Escribe el número del tag (ej. `v0.1.0`), agrega un título y notas sobre lo que incluye.
   - Arrastra y suelta los instaladores generados en tu carpeta `src-tauri/target/release/bundle/...` en la caja de carga.
   - Haz clic en **Publish release**.

---

## 🔑 Consideraciones Importantes sobre Firma de Código (Code Signing)

Cuando descargas un software de internet que no ha sido "firmado digitalmente", los sistemas operativos suelen mostrar alertas de seguridad:
* **Windows (SmartScreen):** Mostrará una advertencia azul diciendo que el editor es desconocido. El usuario debe presionar *"Más información"* -> *"Ejecutar de todas formas"*.
* **macOS (Gatekeeper):** Mostrará un bloqueo diciendo que el desarrollador no está verificado. El usuario debe abrir la app haciendo clic derecho en ella y seleccionando *"Abrir"*, o dar permiso desde *Ajustes del Sistema -> Privacidad y Seguridad*.

Para evitar estas alertas, los desarrolladores compran certificados de firma digital (Apple Developer Account por $99/año, certificados de Windows por montos similares). Si el proyecto es personal o de código abierto y de confianza mutua, **puedes omitir la firma** y simplemente documentar en el README cómo abrir la aplicación saltándose estas advertencias.
