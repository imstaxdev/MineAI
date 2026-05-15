# MineIA

Idioma: [English](README.md) | [Español](README.es.md)

MineIA es un launcher open source de Minecraft, hecho con asistencia de IA y pensado para funcionar primero en local. El objetivo es ofrecer perfiles offline, bajo consumo de recursos, importación simple de mods y shaders, y archivos de runtime claros para que cualquier persona pueda revisar cómo funciona.

## Características

- Perfiles offline de Minecraft con nombre de usuario local.
- Generación determinística de UUID offline.
- Cache local de versiones usando metadata oficial de Mojang.
- Descargas verificadas con SHA-1 y comprobación de tamaño.
- Detección de Java mediante `MINEIA_JAVA_HOME`, `JAVA_HOME` o `PATH`.
- Proceso de Minecraft separado del launcher en Windows.
- Configuración inicial de bajo consumo para RAM, FPS, distancia de renderizado y argumentos JVM.
- Carpetas por perfil para mods, shaders, resource packs, logs y crash reports.
- Importación de mods `.jar`, shaders `.zip` y modpacks `.mrpack`.
- Hash local de archivos con SHA-256 y BLAKE3.
- Aplicación desktop con Tauri, SolidJS y Tailwind CSS.

## Capturas

Agrega capturas en `docs/screenshots/` antes de publicar una versión final del repositorio.

Imágenes recomendadas:

- pantalla principal del launcher
- selector de versiones
- pantalla de mods y shaders
- ejemplo de carpeta local de perfil

## Estructura Del Repositorio

```text
apps/mineia-launcher/      App desktop, backend Tauri e interfaz SolidJS
crates/mineia-auth/        Validación de usuario offline y generación de UUID
crates/mineia-core/        Rutas locales, perfiles SQLite y hashing
crates/mineia-runtime/     Detección de Java, descargas y lanzamiento de Minecraft
crates/mineia-modpacks/    Lógica para importar mods, shaders y modpacks
docs/                      Documentación del proyecto
scripts/                   Comandos para desarrollo
tools/                     Herramientas internas de mantenimiento
```

## Requisitos

- Windows 10 o superior
- Rust stable
- Node.js 20 o superior
- npm
- Java 17 o superior para versiones modernas de Minecraft

## Instalar Dependencias

```powershell
npm.cmd install --prefix apps/mineia-launcher
```

En algunos sistemas, PowerShell puede bloquear `npm.ps1`. Para evitar problemas en Windows, usa `npm.cmd`.

## Ejecutar En Desarrollo

```powershell
npm.cmd run dev --prefix apps/mineia-launcher
```

## Compilar La App

```powershell
npm.cmd run tauri --prefix apps/mineia-launcher -- build
```

Los ejecutables se generan en:

```text
target/release/
target/release/bundle/
```

## Descargar Ejecutables

Para publicar versiones, adjunta los archivos generados en `target/release/bundle/` dentro de la página de GitHub Releases.

Nombres recomendados:

- `MineIA_1.0.0_x64-setup.exe`
- `MineIA_1.0.0_x64_en-US.msi`

## Versiones

MineIA debe usar versionado semántico:

- `v1.0.0`: primera versión estable
- `v1.1.0`: nuevas funciones sin romper comportamiento existente
- `v1.1.1`: correcciones de errores
- `v2.0.0`: cambios incompatibles o cambios grandes de arquitectura

Actualiza las versiones en:

- `Cargo.toml`
- `apps/mineia-launcher/package.json`
- `apps/mineia-launcher/src-tauri/tauri.conf.json`

## Comandos De Desarrollo

```powershell
cargo fmt --all
cargo check
cargo clippy --workspace --all-targets -- -D warnings
cargo test
npm.cmd run build --prefix apps/mineia-launcher
powershell -ExecutionPolicy Bypass -File scripts/check.ps1
powershell -ExecutionPolicy Bypass -File scripts/build-release.ps1
node tools/clean.mjs
```

Más detalles en [docs/TOOLS.md](docs/TOOLS.md).

## Seguridad

MineIA no incluye login de Microsoft, telemetría, backend ni almacenamiento remoto de cuentas. Está diseñado como launcher local y offline-first.

Notas de seguridad:

- No subas bases de datos locales, logs, perfiles ni archivos descargados del juego.
- Mantén privados los archivos `.env`.
- Valida rutas de archivos antes de extraer modpacks.
- Verifica descargas de Mojang antes de reutilizarlas.
- Trata los mods `.jar` importados como código ejecutable de terceros.

Consulta [SECURITY.md](SECURITY.md).

## Contribuir

1. Haz un fork del repositorio en GitHub.
2. Clona tu fork:

```powershell
git clone https://github.com/tu-usuario/mineia.git
cd mineia
```

3. Crea una rama:

```powershell
git checkout -b feature/descripcion-corta
```

4. Realiza cambios concretos y fáciles de revisar.
5. Ejecuta las verificaciones:

```powershell
powershell -ExecutionPolicy Bypass -File scripts/check.ps1
```

6. Crea el commit y sube la rama:

```powershell
git add .
git commit -m "Mejora la interfaz del launcher"
git push origin feature/descripcion-corta
```

7. Abre un Pull Request.

Los Pull Requests deben incluir resumen, capturas si hay cambios de UI y los comandos de verificación ejecutados.

Consulta [CONTRIBUTING.md](CONTRIBUTING.md).

## Licencia

MineIA se publica bajo la licencia MIT. Consulta [LICENSE](LICENSE).

## Créditos

MineIA es un launcher open source construido con desarrollo asistido por IA y mantenido por sus contribuidores.
