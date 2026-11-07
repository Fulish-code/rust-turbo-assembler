#!/bin/bash
# ============================================================================
#  Instalador de Rust Turbo Assembler para macOS (Intel y Apple Silicon)
#
#  Abre la app "Terminal" (Aplicaciones > Utilidades > Terminal), pega esto
#  y presiona Enter:
#
#    curl -fsSL https://raw.githubusercontent.com/Fulish-code/rust-turbo-assembler/main/instalar-macos.sh | bash
#
#  Desinstalar:
#
#    curl -fsSL https://raw.githubusercontent.com/Fulish-code/rust-turbo-assembler/main/instalar-macos.sh | bash -s -- --quitar
#
#  No pide contraseña: todo se instala en tu carpeta personal (~/Applications).
#  Incluye TASM, TLINK y Turbo Debugger (16 bits), los manuales y DOSBox Staging.
# ============================================================================
set -euo pipefail

REPO="Fulish-code/rust-turbo-assembler"
APPNAME="Rust Turbo Assembler"
DOSBOX_VERSION="v0.83.0"
DOSBOX_URL="https://github.com/dosbox-staging/dosbox-staging/releases/download/${DOSBOX_VERSION}/dosbox-staging-macOS-${DOSBOX_VERSION}.dmg"
APPS="$HOME/Applications"

if [ -t 1 ]; then AZ=$'\033[1;34m'; VE=$'\033[1;32m'; AM=$'\033[1;33m'; RO=$'\033[1;31m'; NE=$'\033[1m'; FIN=$'\033[0m'
else AZ=""; VE=""; AM=""; RO=""; NE=""; FIN=""; fi
paso()  { printf "\n%s▶ %s%s\n" "$AZ" "$*" "$FIN"; }
bien()  { printf "  %s✔%s %s\n" "$VE" "$FIN" "$*"; }
aviso() { printf "  %s!%s %s\n" "$AM" "$FIN" "$*"; }
falla() { printf "\n%s✖ %s%s\n\n" "$RO" "$*" "$FIN"; exit 1; }

# Pregunta sí/no (por defecto: sí). Funciona también con "curl | bash" porque
# lee de la terminal (/dev/tty). Sin terminal (pruebas automáticas) responde sí.
# RTASM_ESCRITORIO=s|n fija la respuesta sin preguntar.
preguntar() {
    r="${RTASM_ESCRITORIO:-}"
    if [ -z "$r" ] && ( : < /dev/tty ) 2>/dev/null; then
        printf "  %s?%s %s [S/n]: " "$AZ" "$FIN" "$1" > /dev/tty
        read -r r < /dev/tty || r=""
    fi
    case "$r" in [nN]*) return 1 ;; *) return 0 ;; esac
}

TMP="$(mktemp -d)"
MNT=""
limpiar() { if [ -n "$MNT" ]; then hdiutil detach -quiet "$MNT" 2>/dev/null || true; fi; rm -rf "$TMP"; }
trap limpiar EXIT

descargar() { curl -fL --retry 3 --progress-bar -o "$2" "$1"; }

hay_dosbox() {
    for p in "$APPS/DOSBox Staging.app/Contents/MacOS/dosbox" "/Applications/DOSBox Staging.app/Contents/MacOS/dosbox" \
             /opt/homebrew/bin/dosbox /usr/local/bin/dosbox /opt/homebrew/bin/dosbox-x /usr/local/bin/dosbox-x; do
        if [ -x "$p" ]; then echo "$p"; return 0; fi
    done
    if command -v dosbox >/dev/null 2>&1; then command -v dosbox; return 0; fi
    return 1
}

if [ "${1:-}" = "--quitar" ] || [ "${1:-}" = "--desinstalar" ]; then
    paso "Desinstalando $APPNAME"
    rm -rf "$APPS/$APPNAME.app"
    rm -f "$HOME/Desktop/$APPNAME"
    bien "Listo. DOSBox Staging, tus archivos .asm y tu configuración no se tocaron."
    exit 0
fi

printf "%s\n  %s — instalador para macOS%s\n" "$NE" "$APPNAME" "$FIN"
[ "$(uname -s)" = "Darwin" ] || falla "Este instalador es para macOS. En Linux usa instalar.sh."
mkdir -p "$APPS"

# ---------- 1. Programa ----------
paso "Descargando $APPNAME"
if [ -n "${RTASM_ZIP_LOCAL:-}" ]; then
    cp "$RTASM_ZIP_LOCAL" "$TMP/app.zip"      # solo para pruebas automáticas
else
    descargar "https://github.com/$REPO/releases/latest/download/rust-tasm-macos.zip" "$TMP/app.zip" \
        || falla "No se pudo descargar. Revisa tu conexión a internet e inténtalo de nuevo."
fi
ditto -x -k "$TMP/app.zip" "$TMP/app"
[ -d "$TMP/app/$APPNAME.app" ] || falla "El archivo descargado está incompleto."
rm -rf "$APPS/$APPNAME.app"
mv "$TMP/app/$APPNAME.app" "$APPS/"
xattr -dr com.apple.quarantine "$APPS/$APPNAME.app" 2>/dev/null || true
bien "Instalado en $APPS (con TASM, TLINK, Turbo Debugger y los manuales)"

# ---------- 2. DOSBox ----------
paso "Revisando DOSBox (necesario para TASM, TLINK y TD)"
if [ -z "${RTASM_FORZAR_DOSBOX:-}" ] && D="$(hay_dosbox)"; then
    bien "Ya tienes DOSBox: $D"
else
    aviso "Descargando DOSBox Staging (libre y gratuito)…"
    if descargar "$DOSBOX_URL" "$TMP/dosbox.dmg"; then
        MNT="$TMP/mnt"
        mkdir -p "$MNT"
        if hdiutil attach -nobrowse -readonly -quiet -mountpoint "$MNT" "$TMP/dosbox.dmg"; then
            SRC="$(find "$MNT" -maxdepth 1 -name '*.app' | head -n1)"
            if [ -n "$SRC" ]; then
                rm -rf "$APPS/DOSBox Staging.app"
                cp -R "$SRC" "$APPS/DOSBox Staging.app"
                xattr -dr com.apple.quarantine "$APPS/DOSBox Staging.app" 2>/dev/null || true
            fi
            hdiutil detach -quiet "$MNT" || true
            MNT=""
        fi
    fi
    if [ -x "$APPS/DOSBox Staging.app/Contents/MacOS/dosbox" ]; then
        bien "DOSBox Staging instalado en $APPS"
    elif command -v brew >/dev/null 2>&1; then
        aviso "Probando con Homebrew…"
        if brew install dosbox-staging; then bien "DOSBox Staging instalado con Homebrew"; else aviso "No se pudo instalar DOSBox."; fi
    else
        aviso "No pude instalar DOSBox. Descárgalo de https://www.dosbox-staging.org y arrástralo a Aplicaciones."
    fi
fi

# ---------- 3. Ícono en el Escritorio ----------
paso "Accesos directos"
bien "Launchpad y carpeta Aplicaciones (con su ícono)"
if [ -d "$HOME/Desktop" ] && preguntar "¿Crear un ícono en el Escritorio?"; then
    rm -f "$HOME/Desktop/$APPNAME"
    ln -s "$APPS/$APPNAME.app" "$HOME/Desktop/$APPNAME"
    bien "Ícono en el Escritorio"
fi

printf "\n%s✔ ¡Listo! %s está instalado.%s\n" "$VE" "$APPNAME" "$FIN"
printf "  Ábrelo desde Launchpad, la carpeta Aplicaciones o el ícono del Escritorio (busca %s\"Turbo\"%s).\n\n" "$NE" "$FIN"
if [ -z "${RTASM_ZIP_LOCAL:-}" ]; then open "$APPS/$APPNAME.app" 2>/dev/null || true; fi
