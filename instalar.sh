#!/usr/bin/env bash
# ============================================================================
#  Instalador de Rust Turbo Assembler (Linux)
#
#  Instalar con un solo comando (copia y pega en una terminal):
#
#    curl -fsSL https://raw.githubusercontent.com/Fulish-code/rust-turbo-assembler/main/instalar.sh | bash
#
#  Desinstalar:
#
#    curl -fsSL https://raw.githubusercontent.com/Fulish-code/rust-turbo-assembler/main/instalar.sh | bash -s -- --quitar
#
#  No necesita permisos de administrador: todo se instala en tu carpeta personal.
#  Incluye TASM, TLINK y Turbo Debugger (16 bits), los manuales y DOSBox Staging.
# ============================================================================
set -euo pipefail

REPO="Fulish-code/rust-turbo-assembler"
PAQUETE="rust-tasm-linux-x86_64.tar.gz"
DOSBOX_VERSION="v0.83.0"
DOSBOX_URL="https://github.com/dosbox-staging/dosbox-staging/releases/download/${DOSBOX_VERSION}/dosbox-staging-linux-x86_64-${DOSBOX_VERSION}.tar.xz"

DATOS="${XDG_DATA_HOME:-$HOME/.local/share}"
APP="$DATOS/rust-tasm"
ENLACE="$HOME/.local/bin/rust-tasm"
MENU="$DATOS/applications/rust-tasm.desktop"
ICONO="$DATOS/icons/hicolor/256x256/apps/rust-tasm.png"
ICONO_VIEJO="$DATOS/icons/hicolor/scalable/apps/rust-tasm.svg"   # versiones anteriores a la 1.2

# ---------- mensajes bonitos ----------
if [ -t 1 ]; then AZ=$'\e[1;34m'; VE=$'\e[1;32m'; AM=$'\e[1;33m'; RO=$'\e[1;31m'; NE=$'\e[1m'; FIN=$'\e[0m'
else AZ=""; VE=""; AM=""; RO=""; NE=""; FIN=""; fi
paso()  { printf "\n%s▶ %s%s\n" "$AZ" "$*" "$FIN"; }
bien()  { printf "  %s✔%s %s\n" "$VE" "$FIN" "$*"; }
aviso() { printf "  %s!%s %s\n" "$AM" "$FIN" "$*"; }
falla() { printf "\n%s✖ %s%s\n\n" "$RO" "$*" "$FIN"; exit 1; }

# Pregunta sí/no (por defecto: sí). Funciona también con "curl | bash" porque
# lee de la terminal (/dev/tty). Sin terminal (pruebas automáticas) responde sí.
# RTASM_ESCRITORIO=s|n fija la respuesta sin preguntar.
preguntar() {
    local r="${RTASM_ESCRITORIO:-}"
    if [ -z "$r" ] && ( : < /dev/tty ) 2>/dev/null; then
        printf "  %s?%s %s [S/n]: " "$AZ" "$FIN" "$1" > /dev/tty
        read -r r < /dev/tty || r=""
    fi
    case "$r" in [nN]*) return 1 ;; *) return 0 ;; esac
}

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

descargar() { # url destino
    if command -v curl >/dev/null 2>&1; then
        curl -fL --retry 3 --progress-bar -o "$2" "$1"
    elif command -v wget >/dev/null 2>&1; then
        wget -q --show-progress -O "$2" "$1"
    else
        falla "No encontré 'curl' ni 'wget'. Instala uno de ellos e inténtalo de nuevo."
    fi
}

hay_dosbox() {
    for d in dosbox-staging dosbox-x dosbox; do
        command -v "$d" >/dev/null 2>&1 && { echo "$(command -v "$d")"; return 0; }
    done
    [ -x "$APP/dosbox-staging/dosbox" ] && { echo "$APP/dosbox-staging/dosbox"; return 0; }
    return 1
}

# ---------- desinstalar ----------
if [ "${1:-}" = "--quitar" ] || [ "${1:-}" = "--desinstalar" ]; then
    paso "Desinstalando Rust Turbo Assembler"
    ESCRITORIO="$(xdg-user-dir DESKTOP 2>/dev/null || echo "$HOME/Desktop")"
    rm -rf "$APP" "$ENLACE" "$MENU" "$ICONO" "$ICONO_VIEJO" "$ESCRITORIO/rust-tasm.desktop"
    command -v update-desktop-database >/dev/null 2>&1 && update-desktop-database "$DATOS/applications" 2>/dev/null || true
    bien "Listo. Tus archivos .asm y tu configuración (~/.config/rust-tasm) no se tocaron."
    exit 0
fi

printf "%s\n  Rust Turbo Assembler — instalador%s\n" "$NE" "$FIN"
printf "  Se instalará en: %s\n" "$APP"

[ "$(uname -s)" = "Linux" ] || falla "Este instalador es para Linux. En Windows y macOS, descarga la versión correspondiente desde https://github.com/$REPO/releases"
ARQ="$(uname -m)"

# ---------- 1. Conseguir el programa ----------
FUENTE=""
# Solo cuando el script se ejecuta como archivo (./instalar.sh dentro del paquete);
# con "curl | bash" no hay archivo y siempre se descarga la versión publicada.
AQUI=""
SCRIPT="${BASH_SOURCE[0]:-}"
if [ -n "$SCRIPT" ] && [ -f "$SCRIPT" ]; then AQUI="$(cd "$(dirname "$SCRIPT")" && pwd)"; fi
if [ -n "$AQUI" ] && [ -x "$AQUI/rust-tasm" ] && [ -f "$AQUI/BIN/TASM.EXE" ]; then
    # Se está ejecutando desde un paquete ya descomprimido (instalación sin internet).
    paso "Usando los archivos de esta carpeta"
    FUENTE="$AQUI"
elif [ "$ARQ" = "x86_64" ]; then
    paso "Descargando Rust Turbo Assembler"
    if descargar "https://github.com/$REPO/releases/latest/download/$PAQUETE" "$TMP/paquete.tar.gz"; then
        tar -xzf "$TMP/paquete.tar.gz" -C "$TMP"
        FUENTE="$TMP/rust-tasm"
        bien "Descargado"
    else
        aviso "No se pudo descargar la versión compilada; se compilará desde el código."
    fi
fi

if [ -z "$FUENTE" ]; then
    # Compilar desde el código (otras arquitecturas, p. ej. ARM).
    paso "Compilando desde el código fuente (tarda unos minutos, solo esta vez)"
    command -v git >/dev/null 2>&1 || falla "Necesito 'git' para descargar el código. Instálalo (p. ej. sudo apt install git) e inténtalo de nuevo."
    if ! command -v cargo >/dev/null 2>&1; then
        [ -x "$HOME/.cargo/bin/cargo" ] || {
            aviso "Instalando Rust (el compilador) en tu carpeta personal…"
            descargar https://sh.rustup.rs "$TMP/rustup.sh"
            sh "$TMP/rustup.sh" -y --profile minimal >/dev/null
        }
        export PATH="$HOME/.cargo/bin:$PATH"
    fi
    git clone --depth 1 "https://github.com/$REPO.git" "$TMP/src" >/dev/null 2>&1 || falla "No se pudo descargar el código de GitHub. Revisa tu conexión a internet."
    (cd "$TMP/src" && CARGO_TARGET_DIR="$TMP/target" cargo build --release) || falla "La compilación falló. Puede faltar un compilador de C: instala 'build-essential' (Debian/Ubuntu) o 'base-devel' (Arch)."
    cp "$TMP/target/release/rust-tasm" "$TMP/src/rust-tasm"
    FUENTE="$TMP/src"
    bien "Compilado"
fi

# ---------- 2. Copiar archivos ----------
paso "Instalando"
mkdir -p "$APP"
rm -rf "$APP/BIN" "$APP/docs" "$APP/locales" "$APP/examples"
install -m755 "$FUENTE/rust-tasm" "$APP/rust-tasm"
cp -r "$FUENTE/BIN" "$FUENTE/docs" "$FUENTE/locales" "$FUENTE/examples" "$APP/"
mkdir -p "$(dirname "$ICONO")"
cp "$FUENTE/assets/icon-256.png" "$ICONO"
rm -f "$ICONO_VIEJO"
bien "Programa, Turbo Assembler, Turbo Linker, Turbo Debugger y manuales"

# ---------- 3. DOSBox ----------
paso "Revisando DOSBox (necesario para TASM, TLINK y TD)"
if D="$(hay_dosbox)"; then
    bien "Ya tienes DOSBox: $D"
else
    OK=0
    if [ "$ARQ" = "x86_64" ]; then
        aviso "Descargando DOSBox Staging (libre y gratuito)…"
        if descargar "$DOSBOX_URL" "$TMP/dosbox.tar.xz" && mkdir -p "$TMP/db" && tar -xJf "$TMP/dosbox.tar.xz" -C "$TMP/db"; then
            EXE="$(find "$TMP/db" -type f -name dosbox -perm -u+x | head -n1)"
            if [ -n "$EXE" ]; then
                rm -rf "$APP/dosbox-staging"
                mv "$(dirname "$EXE")" "$APP/dosbox-staging"
                if "$APP/dosbox-staging/dosbox" --version >/dev/null 2>&1; then
                    OK=1
                    bien "DOSBox Staging instalado"
                else
                    rm -rf "$APP/dosbox-staging"
                fi
            fi
        fi
    fi
    if [ "$OK" = 0 ]; then
        aviso "Se instalará DOSBox con el gestor de paquetes de tu sistema."
        aviso "Te pedirá TU contraseña (la de tu sesión); al escribirla no se ve nada, es normal."
        if command -v apt-get >/dev/null 2>&1; then
            sudo apt-get update -qq || true
            sudo apt-get install -y dosbox-staging || sudo apt-get install -y dosbox || true
        elif command -v dnf >/dev/null 2>&1; then
            sudo dnf install -y dosbox-staging || sudo dnf install -y dosbox || true
        elif command -v pacman >/dev/null 2>&1; then
            sudo pacman -Syu --needed --noconfirm dosbox-staging || true
        elif command -v zypper >/dev/null 2>&1; then
            sudo zypper --non-interactive install dosbox-staging || sudo zypper --non-interactive install dosbox || true
        elif command -v flatpak >/dev/null 2>&1; then
            flatpak install -y --user flathub io.github.dosbox-staging || true
        fi
        if D="$(hay_dosbox)"; then bien "DOSBox instalado: $D"
        else aviso "No pude instalar DOSBox. El programa funciona, pero para ensamblar necesitas instalar 'dosbox-staging' o 'dosbox'."; fi
    fi
fi

# ---------- 4. Accesos directos ----------
paso "Creando accesos directos"
mkdir -p "$(dirname "$ENLACE")" "$(dirname "$MENU")"
ln -sf "$APP/rust-tasm" "$ENLACE"
cat > "$MENU" <<DESK
[Desktop Entry]
Type=Application
Name=Rust Turbo Assembler
GenericName=IDE de ensamblador (TASM)
Comment=Escribe, ensambla, ejecuta y depura programas en ensamblador (TASM 16 bits)
Exec="$APP/rust-tasm" %F
Icon=rust-tasm
Terminal=false
Categories=Development;IDE;Education;
Keywords=asm;assembler;ensamblador;tasm;dos;
MimeType=text/x-asm;
StartupWMClass=rust-tasm
DESK
chmod +x "$MENU"
command -v update-desktop-database >/dev/null 2>&1 && update-desktop-database "$DATOS/applications" 2>/dev/null || true
command -v gtk-update-icon-cache >/dev/null 2>&1 && gtk-update-icon-cache -q "$DATOS/icons/hicolor" 2>/dev/null || true
bien "Menú de aplicaciones"

ESCRITORIO="$(xdg-user-dir DESKTOP 2>/dev/null || echo "$HOME/Desktop")"
if [ -d "$ESCRITORIO" ] && preguntar "¿Crear un ícono en el Escritorio?"; then
    cp "$MENU" "$ESCRITORIO/rust-tasm.desktop"
    chmod +x "$ESCRITORIO/rust-tasm.desktop"
    command -v gio >/dev/null 2>&1 && gio set "$ESCRITORIO/rust-tasm.desktop" metadata::trusted true 2>/dev/null || true
    bien "Ícono en el Escritorio"
fi

printf "\n%s✔ ¡Listo! Rust Turbo Assembler está instalado.%s\n" "$VE" "$FIN"
printf "  Ábrelo desde el menú de aplicaciones (busca %s\"Turbo\"%s) o con su ícono del Escritorio.\n\n" "$NE" "$FIN"
