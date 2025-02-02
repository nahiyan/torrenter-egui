use egui::{
    Color32, Context, CursorIcon, Label, Pos2, Rect, Response, Rounding, Sense, Stroke, Ui, Vec2,
    Widget,
};

pub struct AddTorrentWidget<'a> {
    has_hovering_files: bool,
    is_clicked: &'a mut bool,
    ctx: &'a Context,
}

impl<'a> AddTorrentWidget<'a> {
    pub fn new(about_to_drop: bool, is_clicked: &'a mut bool, ctx: &'a Context) -> Self {
        Self {
            has_hovering_files: about_to_drop,
            is_clicked,
            ctx,
        }
    }
}

impl<'a> Widget for AddTorrentWidget<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        // Drag and drop guide
        ui.horizontal(|ui| {
            let start_pos = Pos2::new(ui.next_widget_position().x, ui.min_rect().top());
            let drop_rect = Rect::from_min_size(start_pos, Vec2::new(ui.available_width(), 50.0));
            let drop_element = ui.allocate_rect(drop_rect, Sense::click());
            if drop_element.hovered() {
                self.ctx.set_cursor_icon(CursorIcon::PointingHand);
            }
            *self.is_clicked = drop_element.clicked();

            let color = ui.style().visuals.panel_fill;
            let stroke = if self.has_hovering_files {
                Stroke::new(2.0, Color32::LIGHT_GREEN)
            } else if drop_element.hovered() {
                Stroke::new(2.0, Color32::WHITE.gamma_multiply(1.0))
            } else {
                Stroke::new(2.0, Color32::WHITE.gamma_multiply(0.5))
            };
            ui.painter()
                .rect(drop_rect, Rounding::from(2.5), color, stroke);

            let text = Label::new(
                "Paste a magnet URL ⚪ \
                Drag and drop a torrent file ⚪ \
                Click to select a torrent file",
            )
            .sense(Sense::focusable_noninteractive())
            .selectable(false);
            ui.put(drop_rect, text);
        });
        ui.response()
    }
}
