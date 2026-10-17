# Recorte

Herramienta de captura y edición rápida de pantalla para **GNOME en Wayland**, hecha en **Rust** 🦀.

Pulsas `Ctrl + Impr Pant`, la pantalla se congela, seleccionas un área (aunque cruce varios
monitores), la anotas y la copias o guardas.

Recorte nació porque Flameshot no funcionaba con varios monitores en GNOME con Wayland. Está
inspirado en sus funciones, pero pensado desde cero para Wayland y escritorios con varios
monitores.

**⬇️ Descargar:** [.deb (Ubuntu/Debian)](https://github.com/Richie-AS/recorte/releases/latest/download/recorte_amd64.deb)
· [.rpm (Fedora/openSUSE)](https://github.com/Richie-AS/recorte/releases/latest/download/recorte_x86_64.rpm)
· [todas las versiones](https://github.com/Richie-AS/recorte/releases)

## Contenido

- [Funciones](#funciones)
- [Requisitos](#requisitos)
- [Instalación](#instalación)
- [Uso](#uso)
- [Icono en la bandeja](#icono-en-la-bandeja)
- [Varios monitores](#varios-monitores)
- [Línea de comandos](#línea-de-comandos)
- [Archivos que usa Recorte](#archivos-que-usa-recorte)
- [Solución de problemas](#solución-de-problemas)
- [Desinstalar](#desinstalar)
- [Desarrollo](#desarrollo)
- [Licencia](#licencia)

## Funciones

- **Captura nativa en Wayland** mediante el portal de capturas del escritorio (sin X11).
- **Selección entre monitores**: el área puede cruzar pantallas con distinta resolución y escala.
- **Anotaciones**: flecha, rectángulo, marcador translúcido y texto.
- **Paleta de 8 colores y 3 grosores**, que se recuerdan para la próxima captura.
- **Pixelado** para ocultar datos sensibles (bloques de 8×8 píxeles).
- **Copiar al portapapeles** o **guardar como PNG** en la carpeta de capturas.
- **Barra flotante** junto a la selección, que salta arriba si no cabe debajo.
- **Atajo global** `Ctrl + Impr Pant` e **icono en la bandeja del sistema**.
- Paquetes **.deb** y **.rpm**.

## Requisitos

| Requisito | Notas |
|---|---|
| GNOME en Wayland | Probado en Ubuntu 26.04 (GNOME 50). |
| `xdg-desktop-portal` y `xdg-desktop-portal-gnome` | Vienen instalados en Ubuntu. Los paquetes los declaran como dependencia. |
| Extensión **AppIndicator** | Solo para el icono en la bandeja. Ubuntu la trae activada (`ubuntu-appindicators@ubuntu.com`). |
| `notify-send` | Opcional: muestra los errores como notificación. |
| Rust 1.95 o posterior | Solo para compilar desde el código fuente. |

> **Compatibilidad de los paquetes:** se compilan contra la versión de glibc del equipo donde se
> generan. Un paquete hecho en Ubuntu 26.04 no se instala en versiones más antiguas; para ellas hay
> que generarlo en esa misma versión.

## Instalación

### Desde un paquete

Descarga la última versión e instálala:

```bash
# Ubuntu / Debian
wget https://github.com/Richie-AS/recorte/releases/latest/download/recorte_amd64.deb
sudo apt install ./recorte_amd64.deb

# Fedora / openSUSE
wget https://github.com/Richie-AS/recorte/releases/latest/download/recorte_x86_64.rpm
sudo dnf install ./recorte_x86_64.rpm
```

Después, **cada usuario** ejecuta una vez:

```bash
recorte --instalar-atajo
```

Este comando:

1. Crea el atajo `Ctrl + Impr Pant` en GNOME, sin tocar tus otros atajos personalizados.
2. Concede a Recorte el permiso del portal para capturar la pantalla sin preguntar.

El segundo paso es necesario: GNOME solo puede mostrar el diálogo de permiso si la aplicación tiene
el foco, y al lanzarse desde el atajo Recorte todavía no tiene ninguna ventana.

### Desde el código fuente

```bash
git clone https://github.com/Richie-AS/recorte.git
cd recorte
./instalar.sh
```

`instalar.sh` compila en modo optimizado, instala el ejecutable en `~/.local/bin/recorte`, crea el
lanzador en el menú de aplicaciones y ejecuta `recorte --instalar-atajo`. No necesita `sudo`.

> Si tienes a la vez el paquete instalado y la copia de `~/.local/bin`, se usará la de
> `~/.local/bin`. Conviene quedarse con una sola.

## Uso

### Abrir una captura

- Pulsa `Ctrl + Impr Pant`, **o**
- abre **Recorte** desde el menú de aplicaciones, **o**
- elige **Capturar pantalla** en el icono de la bandeja.

La pantalla se congela y se oscurece. Si pulsas el atajo varias veces seguidas, solo se abre una
captura.

### Seleccionar el área

Arrastra con el ratón para marcar el área. Para cambiarla, vuelve a la herramienta **Selección**
(`S`) y arrastra de nuevo. Mientras no haya un área seleccionada, cualquier herramienta empieza
seleccionando.

### Herramientas

Se eligen en la barra flotante o con su tecla (sin modificadores):

| Herramienta | Tecla | Cómo se usa |
|---|---|---|
| Selección | `S` | Arrastra para marcar o rehacer el área. |
| Flecha | `A` | Arrastra desde el origen hasta la punta. |
| Rectángulo | `R` | Arrastra de esquina a esquina. |
| Marcador | `M` | Dibuja a mano alzada con trazo translúcido, como un resaltador. |
| Texto | `T` | Haz clic donde quieras escribir. |
| Pixelar | `P` | Arrastra sobre la zona que quieres ocultar. |

Las anotaciones quedan dentro del área seleccionada.

**Texto:** mientras escribes, las teclas van al texto. `Enter` o un clic fuera lo confirman,
`Retroceso` borra y `Esc` lo descarta.

### Color y grosor

El botón con un círculo de color, en la barra, abre la paleta:

- **Colores:** rojo, naranja, amarillo, verde, azul, morado, blanco y negro.
- **Grosores:** fino, medio y grueso. El grosor también cambia el tamaño del texto y el ancho del
  marcador.

Cada anotación conserva el color y grosor con que se dibujó: cambiarlos no altera lo ya dibujado. El
marcador usa el color elegido en versión translúcida; para el resaltado amarillo clásico, elige
amarillo.

### Barra flotante

Aparece debajo de la selección o encima si no cabe. De izquierda a derecha:

| Botón | Acción |
|---|---|
| Herramientas | Selección, flecha, rectángulo, marcador, texto y pixelar. |
| Color y grosor | Abre la paleta. |
| Deshacer | Quita la última anotación (`Ctrl + Z`). |
| Copiar | Copia al portapapeles y cierra (`Enter` / `Ctrl + C`). |
| Guardar | Guarda como PNG y cierra (`Ctrl + S`). |
| Acerca de | Foto del autor, versión, licencia y créditos (`F1`). |
| Cerrar | Sale sin copiar ni guardar (`Esc`). |

### Atajos de teclado

| Atajo | Acción |
|---|---|
| `Ctrl + Impr Pant` | Abrir una captura (atajo global). |
| `Enter` | Copiar y salir, **previa confirmación**. |
| `Ctrl + C` | Copiar y salir directamente. |
| `Ctrl + S` | Guardar y salir, previa confirmación. |
| `Ctrl + Z` | Deshacer la última anotación. |
| `S` `A` `R` `M` `T` `P` | Cambiar de herramienta. |
| `F1` | Acerca de Recorte. |
| `Esc` | Salir sin guardar, o cancelar el texto o diálogo abierto. |

En los diálogos de confirmación, `Enter` confirma y `Esc` vuelve a la edición.

### Copiar y guardar

- **Copiar:** la imagen queda en el portapapeles lista para pegar. En Linux el portapapeles
  pertenece al programa que copió, así que Recorte deja un pequeño proceso en segundo plano que lo
  mantiene hasta que copies otra cosa.
- **Guardar:** se guarda en `~/Imágenes/Capturas de pantalla/Captura AAAA-MM-DD HH-MM-SS.png`. Si
  esa carpeta no existe, en `~/Imágenes`. Dos capturas en el mismo segundo no se pisan.

## Icono en la bandeja

Al abrir Recorte por primera vez aparece su icono en la barra superior, y se queda ahí hasta que
eliges **Salir**. Su menú:

| Opción | Qué hace |
|---|---|
| Capturar pantalla | Abre una captura (espera a que se cierre el menú para que no salga en la imagen). |
| Abrir carpeta de capturas | Abre la carpeta donde se guardan los PNG. |
| Iniciar con la sesión | Si está marcada, el icono aparece al iniciar sesión. Desactivada por defecto. |
| Salir | Cierra el icono. El atajo sigue funcionando. |

Cada captura corre en su propio proceso: si una falla, el icono no se cierra.

## Varios monitores

- Se abre una ventana a pantalla completa por monitor, y todas comparten la misma selección y
  anotaciones.
- La selección puede cruzar de una pantalla a otra, aunque tengan distinta resolución o escala (por
  ejemplo, 4K al 200 % junto a Full HD al 100 %). La imagen final usa la resolución real de la
  captura.
- La barra flotante se coloca en el monitor donde está la selección.

## Línea de comandos

| Comando | Qué hace |
|---|---|
| `recorte` | Abre una captura (e inicia el icono de la bandeja si no está). |
| `recorte --bandeja` | Solo muestra el icono en la bandeja. |
| `recorte --instalar-atajo` | Configura el atajo y el permiso de captura para el usuario actual. |

Variables de entorno útiles para diagnosticar problemas gráficos:

| Variable | Efecto |
|---|---|
| `WGPU_POWER_PREF=high` | Usar la GPU dedicada en lugar de la integrada. |
| `WGPU_BACKEND=gl` | Usar OpenGL en lugar de Vulkan. |

## Archivos que usa Recorte

| Ruta | Contenido |
|---|---|
| `~/.config/recorte/ajustes` | Último color y grosor elegidos. |
| `~/.config/autostart/io.github.Richie_AS.Recorte.desktop` | Existe si "Iniciar con la sesión" está marcado. |
| `~/Imágenes/Capturas de pantalla/` | Capturas guardadas. |
| `$XDG_RUNTIME_DIR/recorte.lock` | Evita dos capturas a la vez. |
| `$XDG_RUNTIME_DIR/recorte-bandeja.lock` | Evita dos iconos en la bandeja. |

El identificador de la aplicación es `io.github.Richie_AS.Recorte`. Coincide con el nombre del
lanzador `.desktop` y es el nombre con que el portal guarda el permiso de captura.

## Solución de problemas

**El atajo no hace nada o aparece "No se pudo capturar la pantalla".**
Falta el permiso del portal. Ejecuta `recorte --instalar-atajo`, o abre Recorte una vez desde el
menú de aplicaciones y acepta el diálogo.

**No aparece el icono en la bandeja.**
Comprueba que la extensión AppIndicator esté activa:

```bash
gnome-extensions enable ubuntu-appindicators@ubuntu.com
```

**La pantalla se congela o Recorte se cierra al abrirse.**
Recorte usa la GPU integrada para evitar problemas con GPUs dedicadas antiguas. Si aun así falla,
prueba `WGPU_BACKEND=gl recorte` desde una terminal.

**`Ctrl + Impr Pant` ya lo usa otra aplicación.**
Cámbialo en *Configuración → Teclado → Atajos de teclado → Atajos personalizados → Recorte*.

**Ver los mensajes de error.**

```bash
journalctl --user -b | grep -i recorte
```

## Desinstalar

```bash
# 1. Quitar el programa
sudo apt remove recorte                 # o: sudo dnf remove recorte
# (si lo instalaste con instalar.sh:)
rm ~/.local/bin/recorte ~/.local/share/applications/io.github.Richie_AS.Recorte.desktop

# 2. Quitar ajustes e inicio automático
rm -r ~/.config/recorte
rm -f ~/.config/autostart/io.github.Richie_AS.Recorte.desktop

# 3. Quitar el permiso de captura
gdbus call --session \
  --dest org.freedesktop.impl.portal.PermissionStore \
  --object-path /org/freedesktop/impl/portal/PermissionStore \
  --method org.freedesktop.impl.portal.PermissionStore.DeletePermission \
  screenshot screenshot io.github.Richie_AS.Recorte
```

El atajo se quita desde *Configuración → Teclado → Atajos personalizados*.

## Desarrollo

### Compilar y probar

```bash
cargo run      # abre una captura
cargo test     # pruebas unitarias (incluye el renderizado de anotaciones)
```

Para ver la imagen que genera la prueba de renderizado:

```bash
RENDER_TEST_OUT=/tmp/render.png cargo test render
```

### Estructura del código

| Archivo | Responsabilidad |
|---|---|
| `src/main.rs` | Punto de entrada, ventanas por monitor, estado compartido, ratón y teclado. |
| `src/capture.rs` | Captura con el portal XDG (`ashpd`). |
| `src/annotations.rs` | Herramientas, estilos, vista previa y renderizado final (`tiny-skia`, `ab_glyph`). |
| `src/toolbar.rs` | Barra flotante, iconos y paleta de colores. |
| `src/clipboard.rs` | Copia al portapapeles con un proceso en segundo plano (`arboard`). |
| `src/save.rs` | Guardado en PNG. |
| `src/settings.rs` | Ajustes persistentes (color y grosor). |
| `src/shortcut.rs` | `--instalar-atajo`: atajo de GNOME y permiso del portal. |
| `src/tray.rs` | `--bandeja`: icono en la bandeja (`ksni`). |
| `src/about.rs` | Diálogo "Acerca de". |
| `packaging/` | Lanzador `.desktop` para los paquetes. |
| `assets/autor.jpg` | Foto del autor del diálogo "Acerca de" (256×256, incluida en el ejecutable). |

### Arquitectura

- **Captura:** usa el portal `org.freedesktop.portal.Screenshot` en modo no interactivo. GNOME no
  implementa `wlr-screencopy`, así que no sirven herramientas como `libwayshot`. El portal
  devuelve un único PNG con todos los monitores, y la captura se hace antes de abrir las ventanas,
  por eso la pantalla se "congela".
- **Interfaz:** `egui` sobre `winit` + `wgpu`, sin `eframe`. En Wayland una ventana no puede
  abarcar varios monitores ni elegir su posición, así que se abre una ventana sin bordes a pantalla
  completa por monitor.
- **Coordenadas:** todo el estado (selección, anotaciones) está en coordenadas lógicas globales.
  Cada ventana convierte con su propia escala, y al exportar se mapea a los píxeles reales de la
  captura.
- **GPU:** se pide la GPU integrada (`LowPower`), suficiente para dibujar y más estable en equipos
  con gráficos híbridos.

Las decisiones de diseño y el estado del proyecto están con más detalle en
[CONTEXTO.md](CONTEXTO.md).

### Generar los paquetes

```bash
cargo install cargo-deb cargo-generate-rpm
cargo deb              # → target/debian/recorte_<versión>-1_amd64.deb
cargo generate-rpm     # → target/generate-rpm/recorte-<versión>-1.x86_64.rpm
```

`cargo deb` compila en modo release; `cargo generate-rpm` reutiliza ese binario, así que se ejecuta
después. Antes de publicar una versión nueva, sube `version` en `Cargo.toml`: si no, `apt` y `dnf`
no la reconocen como actualización.

### Publicar una versión

Los enlaces de descarga directa (`releases/latest/download/recorte_amd64.deb` y
`recorte_x86_64.rpm`) dependen de que **cada versión incluya copias de los instaladores con esos
nombres fijos**, además de los nombres con número de versión:

```bash
V=0.1.4   # la versión nueva
cp target/debian/recorte_${V}-1_amd64.deb /tmp/recorte_amd64.deb
cp target/generate-rpm/recorte-${V}-1.x86_64.rpm /tmp/recorte_x86_64.rpm
gh release create v${V} target/debian/recorte_${V}-1_amd64.deb \
  target/generate-rpm/recorte-${V}-1.x86_64.rpm /tmp/recorte_amd64.deb /tmp/recorte_x86_64.rpm
```

## Licencia

Recorte es software libre bajo la licencia **GPL-3.0 o posterior**. Ver [LICENSE](LICENSE).

Usa, entre otras, las bibliotecas `egui`, `winit`, `wgpu`, `ashpd`, `arboard`, `image`, `tiny-skia`,
`ab_glyph` y `ksni`, y la fuente Ubuntu (Ubuntu Font Licence 1.0).
