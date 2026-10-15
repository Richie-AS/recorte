# Recorte

Herramienta de captura y edición rápida de pantalla para GNOME Wayland, escrita en Rust.

- Selección de un área que puede cruzar varios monitores.
- Anotaciones: flechas, rectángulos, texto y marcador, con paleta de colores y grosores.
- Pixelado para ocultar datos sensibles.
- Copia al portapapeles (`Enter` / `Ctrl+C`) o guarda en `~/Imágenes/Capturas de pantalla` (`Ctrl+S`).
- Atajo global `Ctrl+Impr Pant` e icono en la bandeja del sistema.

## Requisitos

- Ubuntu 26.04 (o equivalente) con GNOME en Wayland.
- `xdg-desktop-portal` y `xdg-desktop-portal-gnome`.
- Para el icono de la bandeja: extensión AppIndicator (incluida en Ubuntu).
- Para compilar: Rust 1.95 o posterior.

## Instalación

Desde el código fuente, en la carpeta del usuario:

```bash
./instalar.sh
```

O generando un paquete:

```bash
cargo install cargo-deb cargo-generate-rpm
cargo deb            # target/debian/*.deb
cargo generate-rpm   # target/generate-rpm/*.rpm (tras cargo build --release)
```

Después de instalar un paquete, cada usuario activa el atajo y el permiso de captura con:

```bash
recorte --instalar-atajo
```

## Desarrollo

Las decisiones de arquitectura y el estado del proyecto están en [CONTEXTO.md](CONTEXTO.md).

```bash
cargo run      # abre una captura
cargo test
```

## Licencia

GPL-3.0 o posterior. Ver [LICENSE](LICENSE).
