#!/usr/bin/env bash
# Compila en modo optimizado e instala recorte con su lanzador y el atajo Ctrl+Impr Pant.
set -euo pipefail
cd "$(dirname "$0")"

cargo build --release

mkdir -p ~/.local/bin ~/.local/share/applications
install -s -m 755 target/release/recorte ~/.local/bin/recorte

# El nombre del lanzador debe coincidir con el identificador de la app (APP_ID).
rm -f ~/.local/share/applications/recorte.desktop ~/.local/share/applications/py.com.code100.Recorte.desktop
cat > ~/.local/share/applications/io.github.Richie_AS.Recorte.desktop <<EOF
[Desktop Entry]
Type=Application
Name=Recorte
Comment=Captura y edición rápida de pantalla
Exec=$HOME/.local/bin/recorte
Icon=applets-screenshooter
Terminal=false
Categories=Utility;Graphics;
EOF

# Atajo global de GNOME (Ctrl+Impr Pant), sin borrar otros atajos personalizados.
~/.local/bin/recorte --instalar-atajo

echo "Instalado en ~/.local/bin/recorte (atajo: Ctrl+Impr Pant)"
