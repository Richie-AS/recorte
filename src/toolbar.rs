//! Barra flotante translúcida (estilo GNOME) que acompaña a la selección.

use egui::{
    Align2, Color32, CornerRadius, FontId, Frame, Id, Margin, Order, Pos2, Rect, Sense, Shape,
    Stroke, StrokeKind, Vec2,
};

use crate::annotations::{PALETTE, Style, Thickness, Tool};

const BUTTON_SIZE: f32 = 34.0;
/// Separación entre la barra y la selección o el borde del monitor.
const GAP: f32 = 8.0;
const ICON_COLOR: Color32 = Color32::from_gray(235);
const ACCENT: Color32 = Color32::from_rgb(0x35, 0x84, 0xe4);

/// Tamaño aproximado antes del primer frame (luego se usa el medido).
pub const ESTIMATED_SIZE: Vec2 = Vec2::new(520.0, 46.0);

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ToolbarAction {
    SetTool(Tool),
    Undo,
    Copy,
    Save,
    About,
    Close,
}

#[derive(Clone, Copy)]
enum Icon {
    Tool(Tool),
    Undo,
    Copy,
    Save,
    About,
    Close,
}

/// Coloca la barra debajo de la selección; si no cabe en ese monitor, encima;
/// y si tampoco, dentro de la selección. Todo en coordenadas lógicas globales.
pub fn place(selection: Rect, monitors: &[Rect], size: Vec2) -> Rect {
    let monitor_at = |p: Pos2| {
        monitors
            .iter()
            .copied()
            .find(|m| m.contains(p))
            .or_else(|| {
                // El punto cae fuera de todos (p. ej. en un hueco): el monitor más cercano.
                monitors.iter().copied().min_by(|a, b| {
                    a.distance_sq_to_pos(p).total_cmp(&b.distance_sq_to_pos(p))
                })
            })
            .unwrap_or(selection)
    };

    let below_anchor = Pos2::new(selection.center().x, selection.max.y);
    let above_anchor = Pos2::new(selection.center().x, selection.min.y);
    let below_monitor = monitor_at(below_anchor);
    let above_monitor = monitor_at(above_anchor);

    let below_y = selection.max.y + GAP;
    let above_y = selection.min.y - GAP - size.y;
    let (y, monitor) = if below_y + size.y <= below_monitor.max.y - GAP {
        (below_y, below_monitor)
    } else if above_y >= above_monitor.min.y + GAP {
        (above_y, above_monitor)
    } else {
        (selection.max.y - GAP - size.y, below_monitor)
    };

    let min_x = monitor.min.x + GAP;
    let max_x = (monitor.max.x - GAP - size.x).max(min_x);
    let x = (selection.center().x - size.x / 2.0).clamp(min_x, max_x);
    Rect::from_min_size(Pos2::new(x, y), size)
}

/// Dibuja la barra en `local_pos` (coordenadas de la ventana).
/// Devuelve la acción pulsada y el tamaño real de la barra; `style` cambia si el
/// usuario elige otro color o grosor.
pub fn show(
    ctx: &egui::Context,
    local_pos: Pos2,
    current_tool: Tool,
    can_undo: bool,
    style: &mut Style,
) -> (Option<ToolbarAction>, Vec2) {
    let mut action = None;
    let response = egui::Area::new(Id::new("barra"))
        .fixed_pos(local_pos)
        .order(Order::Foreground)
        .show(ctx, |ui| {
            Frame::new()
                .fill(Color32::from_rgba_premultiplied(30, 30, 30, 225))
                .stroke(Stroke::new(1.0, Color32::from_white_alpha(28)))
                .corner_radius(CornerRadius::same(14))
                .inner_margin(Margin::same(6))
                .show(ui, |ui| {
                    ui.spacing_mut().item_spacing = Vec2::new(2.0, 0.0);
                    ui.horizontal(|ui| {
                        for tool in Tool::ALL {
                            let tip = format!("{} ({:?})", tool.name(), tool.key());
                            if button(ui, Icon::Tool(tool), tool == current_tool, true, &tip) {
                                action = Some(ToolbarAction::SetTool(tool));
                            }
                        }
                        separator(ui);
                        style_button(ui, style);
                        separator(ui);
                        let t = crate::i18n::t();
                        if button(ui, Icon::Undo, false, can_undo, t.tip_undo) {
                            action = Some(ToolbarAction::Undo);
                        }
                        separator(ui);
                        if button(ui, Icon::Copy, false, true, t.tip_copy) {
                            action = Some(ToolbarAction::Copy);
                        }
                        if button(ui, Icon::Save, false, true, t.tip_save) {
                            action = Some(ToolbarAction::Save);
                        }
                        separator(ui);
                        if button(ui, Icon::About, false, true, t.tip_about) {
                            action = Some(ToolbarAction::About);
                        }
                        if button(ui, Icon::Close, false, true, t.tip_close) {
                            action = Some(ToolbarAction::Close);
                        }
                    });
                });
        });
    (action, response.response.rect.size())
}

fn separator(ui: &mut egui::Ui) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(9.0, BUTTON_SIZE), Sense::hover());
    ui.painter().line_segment(
        [rect.center_top() + Vec2::new(0.0, 7.0), rect.center_bottom() - Vec2::new(0.0, 7.0)],
        Stroke::new(1.0, Color32::from_white_alpha(40)),
    );
}

/// Botón con el color activo que abre la paleta de colores y grosores.
fn style_button(ui: &mut egui::Ui, style: &mut Style) {
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(BUTTON_SIZE), Sense::click());
    if response.hovered() {
        ui.painter().rect_filled(rect, 9.0, Color32::from_white_alpha(22));
    }
    let radius = match style.thickness {
        Thickness::Thin => 6.0,
        Thickness::Medium => 8.0,
        Thickness::Thick => 10.0,
    };
    ui.painter().circle(rect.center(), radius, style.color, Stroke::new(1.5, ICON_COLOR));
    let response = response.on_hover_text(crate::i18n::t().color_and_size);

    egui::Popup::from_toggle_button_response(&response)
        .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
        .show(|ui| {
            ui.spacing_mut().item_spacing = Vec2::new(4.0, 6.0);
            ui.horizontal(|ui| {
                for (color, name) in PALETTE.into_iter().zip(crate::i18n::t().colors) {
                    let (rect, swatch) = ui.allocate_exact_size(Vec2::splat(26.0), Sense::click());
                    let selected = style.color == color;
                    if selected {
                        ui.painter().circle_stroke(rect.center(), 12.5, Stroke::new(2.0, ACCENT));
                    }
                    ui.painter().circle(rect.center(), 9.5, color, Stroke::new(1.0, Color32::from_gray(90)));
                    if swatch.on_hover_text(name).clicked() {
                        style.color = color;
                    }
                }
            });
            ui.separator();
            ui.horizontal(|ui| {
                for thickness in Thickness::ALL {
                    let (rect, option) = ui.allocate_exact_size(Vec2::new(66.0, 28.0), Sense::click());
                    if style.thickness == thickness {
                        ui.painter().rect_filled(rect, 7.0, ACCENT);
                    } else if option.hovered() {
                        ui.painter().rect_filled(rect, 7.0, Color32::from_white_alpha(22));
                    }
                    let width = Style { thickness, ..*style }.stroke_width();
                    let line = [rect.left_center() + Vec2::new(14.0, 0.0), rect.right_center() - Vec2::new(14.0, 0.0)];
                    ui.painter().line_segment(line, Stroke::new(width, ICON_COLOR));
                    if option.on_hover_text(thickness.name()).clicked() {
                        style.thickness = thickness;
                    }
                }
            });
        });
}

/// Botón cuadrado con icono. Devuelve `true` si se pulsó.
fn button(ui: &mut egui::Ui, icon: Icon, selected: bool, enabled: bool, tooltip: &str) -> bool {
    let sense = if enabled { Sense::click() } else { Sense::hover() };
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(BUTTON_SIZE), sense);
    let painter = ui.painter();

    if selected {
        painter.rect_filled(rect, 9.0, ACCENT);
    } else if enabled && response.hovered() {
        painter.rect_filled(rect, 9.0, Color32::from_white_alpha(22));
    }
    let color = if enabled { ICON_COLOR } else { Color32::from_gray(110) };
    paint_icon(painter, rect.shrink(9.0), icon, color);

    let response = response.on_hover_text(tooltip);
    enabled && response.clicked()
}

/// Iconos vectoriales de 16×16 aproximadamente, dibujados dentro de `r`.
fn paint_icon(painter: &egui::Painter, r: Rect, icon: Icon, color: Color32) {
    let stroke = Stroke::new(1.6, color);
    let at = |x: f32, y: f32| r.lerp_inside(Vec2::new(x, y));

    match icon {
        Icon::Tool(Tool::Select) => {
            // Esquinas de un marco de selección.
            let len = r.width() * 0.32;
            for (corner, dx, dy) in [
                (r.left_top(), 1.0, 1.0),
                (r.right_top(), -1.0, 1.0),
                (r.left_bottom(), 1.0, -1.0),
                (r.right_bottom(), -1.0, -1.0),
            ] {
                painter.line_segment([corner, corner + Vec2::new(dx * len, 0.0)], stroke);
                painter.line_segment([corner, corner + Vec2::new(0.0, dy * len)], stroke);
            }
        }
        Icon::Tool(Tool::Arrow) => {
            let (from, to) = (at(0.05, 0.95), at(0.95, 0.05));
            painter.line_segment([from, at(0.75, 0.25)], stroke);
            painter.add(Shape::convex_polygon(
                vec![to, at(0.45, 0.12), at(0.88, 0.55)],
                color,
                Stroke::NONE,
            ));
        }
        Icon::Tool(Tool::Rectangle) => {
            painter.rect_stroke(r.shrink(1.0), 2.0, stroke, StrokeKind::Middle);
        }
        Icon::Tool(Tool::Marker) => {
            let highlight = Color32::from_rgba_unmultiplied(0xf6, 0xd3, 0x2d, 200);
            painter.line_segment([at(0.1, 0.75), at(0.9, 0.25)], Stroke::new(6.0, highlight));
        }
        Icon::Tool(Tool::Text) => {
            painter.text(r.center(), Align2::CENTER_CENTER, "T", FontId::proportional(19.0), color);
        }
        Icon::Tool(Tool::Pixelate) => {
            let cell = r.width() / 3.0;
            for row in 0..3 {
                for col in 0..3 {
                    let alpha = if (row + col) % 2 == 0 { 1.0 } else { 0.4 };
                    let min = r.min + Vec2::new(col as f32, row as f32) * cell;
                    let square = Rect::from_min_size(min, Vec2::splat(cell)).shrink(0.6);
                    painter.rect_filled(square, 1.0, color.gamma_multiply(alpha));
                }
            }
        }
        Icon::Undo => {
            // Arco de ~260° con la punta de flecha al inicio.
            let center = at(0.55, 0.55);
            let radius = r.width() * 0.4;
            let points: Vec<Pos2> = (0..=20)
                .map(|i| {
                    let angle = (200.0_f32 - 260.0 * i as f32 / 20.0).to_radians();
                    center + Vec2::new(angle.cos(), -angle.sin()) * radius
                })
                .collect();
            let start = points[0];
            painter.add(Shape::line(points, stroke));
            painter.add(Shape::convex_polygon(
                vec![start + Vec2::new(-4.0, -5.0), start + Vec2::new(5.0, -3.0), start + Vec2::new(-1.0, 4.0)],
                color,
                Stroke::NONE,
            ));
        }
        Icon::Copy => {
            let back = Rect::from_min_max(at(0.3, 0.0), at(1.0, 0.7));
            let front = Rect::from_min_max(at(0.0, 0.3), at(0.7, 1.0));
            painter.rect_stroke(back, 2.0, stroke, StrokeKind::Middle);
            painter.rect_filled(front, 2.0, Color32::from_rgb(30, 30, 30));
            painter.rect_stroke(front, 2.0, stroke, StrokeKind::Middle);
        }
        Icon::Save => {
            painter.line_segment([at(0.5, 0.0), at(0.5, 0.62)], stroke);
            painter.add(Shape::line(vec![at(0.25, 0.4), at(0.5, 0.65), at(0.75, 0.4)], stroke));
            painter.add(Shape::line(vec![at(0.0, 0.7), at(0.0, 1.0), at(1.0, 1.0), at(1.0, 0.7)], stroke));
        }
        Icon::About => {
            painter.circle_stroke(r.center(), r.width() * 0.5, stroke);
            painter.circle_filled(at(0.5, 0.25), 1.4, color);
            painter.line_segment([at(0.5, 0.42), at(0.5, 0.78)], Stroke::new(1.8, color));
        }
        Icon::Close => {
            painter.line_segment([at(0.1, 0.1), at(0.9, 0.9)], stroke);
            painter.line_segment([at(0.9, 0.1), at(0.1, 0.9)], stroke);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SIZE: Vec2 = Vec2::new(400.0, 46.0);

    fn screen() -> Vec<Rect> {
        vec![Rect::from_min_size(Pos2::ZERO, Vec2::new(1920.0, 1080.0))]
    }

    #[test]
    fn goes_below_when_there_is_room() {
        let selection = Rect::from_min_max(Pos2::new(500.0, 200.0), Pos2::new(900.0, 500.0));
        let bar = place(selection, &screen(), SIZE);
        assert_eq!(bar.min.y, 500.0 + GAP);
        assert_eq!(bar.center().x, selection.center().x);
    }

    #[test]
    fn jumps_above_when_no_room_below() {
        let selection = Rect::from_min_max(Pos2::new(500.0, 600.0), Pos2::new(900.0, 1060.0));
        let bar = place(selection, &screen(), SIZE);
        assert_eq!(bar.max.y, 600.0 - GAP);
    }

    #[test]
    fn goes_inside_when_selection_fills_screen() {
        let selection = Rect::from_min_size(Pos2::ZERO, Vec2::new(1920.0, 1080.0));
        let bar = place(selection, &screen(), SIZE);
        assert!(selection.contains_rect(bar));
    }

    #[test]
    fn stays_within_monitor_horizontally() {
        let selection = Rect::from_min_max(Pos2::new(0.0, 100.0), Pos2::new(40.0, 140.0));
        let bar = place(selection, &screen(), SIZE);
        assert_eq!(bar.min.x, GAP);
    }

    #[test]
    fn uses_the_monitor_under_the_selection() {
        // Monitor 4K a la izquierda (lógico 1920×1080 al 200 %) y Full HD más bajo a la derecha.
        let monitors = vec![
            Rect::from_min_size(Pos2::ZERO, Vec2::new(1920.0, 1080.0)),
            Rect::from_min_size(Pos2::new(1920.0, 0.0), Vec2::new(1920.0, 1200.0)),
        ];
        // Termina a 1060 px: no cabe debajo en el izquierdo, pero sí en el derecho.
        let selection = Rect::from_min_max(Pos2::new(2200.0, 600.0), Pos2::new(2600.0, 1060.0));
        let bar = place(selection, &monitors, SIZE);
        assert_eq!(bar.min.y, 1060.0 + GAP);
    }
}
