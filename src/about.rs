//! Diálogo "Acerca de". Los datos salen de `Cargo.toml`.

const NAME: &str = "Recorte";
const VERSION: &str = env!("CARGO_PKG_VERSION");
const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");
const AUTHORS: &str = env!("CARGO_PKG_AUTHORS");
const REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");
const LICENSE: &str = env!("CARGO_PKG_LICENSE");

/// Foto del autor (256×256), recortada a un círculo al mostrarla.
const AUTHOR_PHOTO: &[u8] = include_bytes!("../assets/autor.jpg");
const PHOTO_SIZE: f32 = 112.0;

const CREDITS: &[(&str, &str)] = &[
    ("egui", "interfaz"),
    ("winit", "ventanas"),
    ("wgpu", "gráficos"),
    ("ashpd", "portal de capturas"),
    ("ksni", "icono en la bandeja"),
    ("arboard", "portapapeles"),
    ("image · tiny-skia · ab_glyph", "imagen y dibujo"),
    ("Fuente Ubuntu", "Ubuntu Font Licence 1.0"),
];

/// Dibuja el diálogo. Devuelve `true` si el usuario lo cerró con el botón.
pub fn show(ctx: &egui::Context) -> bool {
    let mut close = false;
    egui::Modal::new(egui::Id::new("acerca_de")).show(ctx, |ui| {
        ui.set_width(380.0);
        ui.vertical_centered(|ui| {
            if let Some(photo) = author_photo(ctx) {
                let image = egui::Image::from_texture(&photo)
                    .fit_to_exact_size(egui::Vec2::splat(PHOTO_SIZE))
                    .corner_radius(PHOTO_SIZE / 2.0);
                ui.add(image);
                ui.add_space(8.0);
            }
            ui.heading(NAME);
            ui.label(format!("Versión {VERSION}"));
            ui.add_space(4.0);
            ui.label(DESCRIPTION);
        });
        ui.add_space(12.0);

        egui::Grid::new("datos").num_columns(2).spacing([12.0, 6.0]).show(ui, |ui| {
            // Cargo separa varios autores con ':'.
            if !AUTHORS.is_empty() {
                ui.strong("Autor");
                ui.label(AUTHORS.replace(':', ", "));
                ui.end_row();
            }
            if !REPOSITORY.is_empty() {
                ui.strong("Código fuente");
                ui.hyperlink(REPOSITORY);
                ui.end_row();
            }
            ui.strong("Licencia");
            ui.label(LICENSE.replace("-or-later", " o posterior"));
            ui.end_row();
        });

        ui.add_space(8.0);
        ui.small(
            "Este programa es software libre: puede redistribuirlo y modificarlo bajo los \
             términos de la Licencia Pública General de GNU. Se distribuye SIN NINGUNA GARANTÍA.",
        );

        ui.add_space(8.0);
        ui.collapsing("Créditos", |ui| {
            for (name, role) in CREDITS {
                ui.label(format!("{name} — {role}"));
            }
        });

        ui.add_space(12.0);
        ui.vertical_centered(|ui| {
            if ui.button("Cerrar  (Esc)").clicked() {
                close = true;
            }
        });
    });
    close
}

/// Textura de la foto, cargada una vez por ventana (cada monitor tiene su contexto).
fn author_photo(ctx: &egui::Context) -> Option<egui::TextureHandle> {
    let id = egui::Id::new("foto_autor");
    if let Some(texture) = ctx.data(|d| d.get_temp::<egui::TextureHandle>(id)) {
        return Some(texture);
    }
    let rgba = image::load_from_memory(AUTHOR_PHOTO).ok()?.to_rgba8();
    let size = [rgba.width() as usize, rgba.height() as usize];
    let pixels = egui::ColorImage::from_rgba_unmultiplied(size, rgba.as_raw());
    let texture = ctx.load_texture("foto_autor", pixels, egui::TextureOptions::LINEAR);
    ctx.data_mut(|d| d.insert_temp(id, texture.clone()));
    Some(texture)
}

#[cfg(test)]
mod tests {
    #[test]
    fn author_photo_decodes() {
        let photo = image::load_from_memory(super::AUTHOR_PHOTO).expect("JPEG válido");
        assert_eq!((photo.width(), photo.height()), (256, 256));
    }
}
