//! Diálogo "Acerca de". Los datos salen de `Cargo.toml`.

const NAME: &str = "Recorte";
const VERSION: &str = env!("CARGO_PKG_VERSION");
const AUTHORS: &str = env!("CARGO_PKG_AUTHORS");
const REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");

/// Foto del autor (256×256), recortada a un círculo al mostrarla.
const AUTHOR_PHOTO: &[u8] = include_bytes!("../assets/autor.jpg");
const PHOTO_SIZE: f32 = 112.0;

fn credits(t: &crate::i18n::Texts) -> [(&'static str, &'static str); 8] {
    [
        ("egui", t.credit_ui),
        ("winit", t.credit_windows),
        ("wgpu", t.credit_graphics),
        ("ashpd", t.credit_portal),
        ("ksni", t.credit_tray),
        ("arboard", t.credit_clipboard),
        ("image · tiny-skia · ab_glyph", t.credit_image),
        (t.credit_font, "Ubuntu Font Licence 1.0"),
    ]
}

/// Dibuja el diálogo. Devuelve `true` si el usuario lo cerró con el botón.
pub fn show(ctx: &egui::Context) -> bool {
    let mut close = false;
    let t = crate::i18n::t();
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
            ui.label(crate::i18n::fill(t.about_version, &[("version", VERSION)]));
            ui.add_space(4.0);
            ui.label(t.about_description);
        });
        ui.add_space(12.0);

        egui::Grid::new("datos").num_columns(2).spacing([12.0, 6.0]).show(ui, |ui| {
            // Cargo separa varios autores con ':'.
            if !AUTHORS.is_empty() {
                ui.strong(t.about_author);
                ui.label(AUTHORS.replace(':', ", "));
                ui.end_row();
            }
            if !REPOSITORY.is_empty() {
                ui.strong(t.about_source);
                ui.hyperlink(REPOSITORY);
                ui.end_row();
            }
            ui.strong(t.about_license);
            ui.label(t.about_license_value);
            ui.end_row();
        });

        ui.add_space(8.0);
        ui.small(t.about_disclaimer);

        ui.add_space(8.0);
        ui.collapsing(t.about_credits, |ui| {
            for (name, role) in credits(t) {
                ui.label(format!("{name} — {role}"));
            }
        });

        ui.add_space(12.0);
        ui.vertical_centered(|ui| {
            if ui.button(t.about_close).clicked() {
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
