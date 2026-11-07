# Rust Turbo Assembler

<p align="center"><img src="assets/icon-256.png" alt="Ícono de Rust Turbo Assembler" width="128"></p>

**Instalar:** [Linux](#instalar-linux) · [macOS](#instalar-macos-intel-y-apple-silicon) ·
[Windows](#instalar-windows-1011) — en Linux y macOS es pegar una línea en la Terminal;
en Windows, [⬇ descargar el instalador](https://github.com/Fulish-code/rust-turbo-assembler/releases/latest/download/rust-tasm-windows-setup.exe).

Port multiplataforma (Linux, macOS y Windows) del **GUI Turbo Assembler** de Lakhya Jyoti Nath
([ljnath/GUI-Turbo-Assembler](https://github.com/ljnath/GUI-Turbo-Assembler)),
reescrito desde cero en **Rust** con [egui](https://github.com/emilk/egui).

Igual que el original, ejecuta las herramientas reales de Borland de 16 bits
(**TASM 4.1, TLINK 7.1 y Turbo Debugger 5.0**) dentro de **DOSBox**. Así,
el ensamblado, el enlazado y la depuración con TD funcionan exactamente como en
Windows.

## Instalar (Linux)

Abre una **Terminal**, pega esta línea y presiona Enter:

```bash
curl -fsSL https://raw.githubusercontent.com/Fulish-code/rust-turbo-assembler/main/instalar.sh | bash
```

Eso es todo. El instalador:

- descarga el programa ya compilado (funciona en Ubuntu 20.04+, Debian 11+, Mint, Fedora, Arch…);
- incluye Turbo Assembler, Turbo Linker, Turbo Debugger y los manuales;
- instala **DOSBox Staging** si no lo tienes (sin pedir contraseña);
- crea el acceso en el menú de aplicaciones, con su ícono, y te pregunta si
  quieres también un **ícono en el Escritorio**.

No necesita permisos de administrador. Para desinstalar:

```bash
curl -fsSL https://raw.githubusercontent.com/Fulish-code/rust-turbo-assembler/main/instalar.sh | bash -s -- --quitar
```

**Sin internet:** descarga `rust-tasm-linux-x86_64.tar.gz` de la página de
[Releases](https://github.com/Fulish-code/rust-turbo-assembler/releases),
descomprímelo y ejecuta `./instalar.sh` dentro de la carpeta.

## Instalar (macOS, Intel y Apple Silicon)

Abre la app **Terminal** (Aplicaciones › Utilidades › Terminal), pega esta línea y presiona Enter:

```bash
curl -fsSL https://raw.githubusercontent.com/Fulish-code/rust-turbo-assembler/main/instalar-macos.sh | bash
```

El instalador:

- instala la app, con su ícono, en tu carpeta Aplicaciones (sin pedir contraseña);
- incluye Turbo Assembler, Turbo Linker, Turbo Debugger y los manuales;
- descarga **DOSBox Staging** si no lo tienes;
- te pregunta si quieres un **ícono en el Escritorio** y abre el programa al terminar.

Después búscalo en Launchpad como "Rust Turbo Assembler". Requiere macOS 11 o superior.
Para desinstalar:

```bash
curl -fsSL https://raw.githubusercontent.com/Fulish-code/rust-turbo-assembler/main/instalar-macos.sh | bash -s -- --quitar
```


## Instalar (Windows 10/11)

1. Descarga [**rust-tasm-windows-setup.exe**](https://github.com/Fulish-code/rust-turbo-assembler/releases/latest/download/rust-tasm-windows-setup.exe).
2. Ábrelo. Si Windows muestra "Windows protegió su PC", haz clic en
   **Más información › Ejecutar de todas formas** (el instalador no está firmado).
3. Sigue los pasos. El instalador te pregunta si quieres un **acceso directo en
   el Escritorio**; siempre crea uno en el menú Inicio.

Incluye Turbo Assembler, Turbo Linker, Turbo Debugger, los manuales y
**DOSBox Staging**: no hay que instalar nada más. Se instala en tu carpeta de
usuario sin pedir contraseña de administrador. Para desinstalar: Configuración ›
Aplicaciones › Rust Turbo Assembler › Desinstalar.

## Compilar desde el código

```bash
cargo run --release      # Rust 1.85+
./empaquetar.sh          # crea dist/rust-tasm-linux-x86_64.tar.gz (requiere zig + cargo-zigbuild)
./empaquetar-macos.sh    # crea dist/rust-tasm-macos.zip (en macOS)
./empaquetar-windows.ps1 # crea dist/rust-tasm-windows-setup.exe (en Windows, requiere Inno Setup 6)
cargo test -- --include-ignored   # incluye pruebas con DOSBox real
```

El programa busca `BIN/` (herramientas Turbo) y `docs/` (manuales) junto al
ejecutable o en una carpeta superior, y DOSBox en el PATH o en `dosbox-staging/`
(Linux) o `dosbox\` (Windows) junto al ejecutable. Al subir una etiqueta `vX.Y.Z`,
GitHub Actions compila los tres sistemas, prueba los instaladores y adjunta los
paquetes a la versión.

## Funciones (las mismas del original)

| Original (GUI Turbo Assembler 5.x) | Aquí |
|---|---|
| Editor con números de línea, resaltado, deshacer/rehacer | ✔ (TASM/MASM, 8086–486, x87) |
| Pestañas acoplables, diseño guardado | ✔ (egui_dock) |
| Ensamblar / Construir / Ejecutar / Depurar (TD) | ✔ mismos mensajes y preguntas ("¿guardar y continuar?", "no se ha ensamblado…") |
| Ensamblado silencioso (sin ventana) | ✔ DOSBox sin ventana; si no se puede, abre la ventana sola |
| Assembly Launcher ("Press any key to exit…") | ✔ (opción en Preferencias) |
| Ventana de salida con errores | ✔ y además: clic en un error → salta a la línea; línea marcada en el editor |
| Terminal con historial | ✔ (TASM, TLINK, TD, DIR… dentro de DOSBox) |
| Buscar/Reemplazar, Ir a línea, comentar, mover líneas, mayúsculas/minúsculas, zoom | ✔ |
| Archivos recientes (20), restaurar sesión, arrastrar y soltar | ✔ |
| Propiedades de archivo, abrir carpeta, imprimir | ✔ (imprimir abre una vista lista para imprimir en el navegador) |
| Temas Azul / Oscuro / Claro + editor claro/oscuro | ✔ |
| Idiomas (EN, FR, DE, PL, RU, HI, ZH-HANT) | ✔ los mismos `.lang` + **español** |
| Argumentos personalizados de TASM/TLINK | ✔ (predeterminados `/zi` y `/v` para depurar) |
| Reglas de nombre DOS (8 caracteres, sin espacios) | ✔ |
| Ejemplos | ✔ 8 ejemplos comentados en español |
| Manual de TASM | ✔ botón **📖 Manual TASM** (completo o por capítulo) |

No se portaron porque dependen de servidores del autor original: buscar
actualizaciones, donaciones y estadísticas de uso. El modo de 32 bits
(TASM32/TLINK32/TD32) tampoco, porque son programas de Windows y no de DOS.

## Atajos

`Alt+F9` ensamblar · `F6` construir · `F9` ensamblar y construir ·
`Ctrl+F9` ensamblar, construir y ejecutar · `Shift+F9` ejecutar · `F8` depurar con TD ·
`Ctrl+F/H/G` buscar/reemplazar/ir a línea · `Ctrl+/` comentar · `Alt+↑/↓` mover líneas.

## Estructura

```
src/
  main.rs        arranque, ventana, ícono
  app.rs         interfaz: menús, barra, pestañas, diálogos, flujo de acciones
  editor.rs      editor de código (números de línea, marcas, operaciones)
  highlight.rs   resaltado de sintaxis TASM
  toolchain.rs   TASM → TLINK → ejecutar → TD; lectura de errores
  dosbox.rs      detección y ejecución de DOSBox (C: BIN, D: tu carpeta, E: intercambio)
  config.rs      preferencias y sesión (~/.config/rust-tasm)
  i18n.rs        idiomas (.lang del original)
  theme.rs       temas Azul/Oscuro/Claro
  devshot.rs     capturas de pantalla para pruebas (RTASM_SHOT=archivo.png)
assets/          ícono (PNG 1024/256 px, .ico de Windows)
windows/         script de Inno Setup del instalador de Windows
BIN/             TASM, TLINK, TD (+ RTM, DPMI16BI, TDHELP) de Borland
docs/            manual de TASM en PDF
locales/         idiomas
examples/        programas de ejemplo
```

## Licencia

GPL-3.0, igual que el original. Turbo Assembler, Turbo Linker y Turbo Debugger
son © Borland International y se incluyen tal como venían en el GUI Turbo
Assembler original.

---

Desarrollado por **Fulish-code**. Herramientas utilizadas: Rust, egui, DOSBox Staging e Inno Setup.

**Uso de inteligencia artificial:** este proyecto se desarrolló con ayuda de
[Claude](https://claude.com) (Anthropic) como asistente de programación: escritura y
revisión de código, scripts de instalación (Windows, Linux y macOS) y documentación.
Las decisiones de diseño, las pruebas y la publicación son del autor.
