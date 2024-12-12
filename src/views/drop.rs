use egui::{
    Align2, Color32, FontId, Pos2, Rect, Response, Rounding, Sense, Stroke, Ui, Vec2, Widget,
};

pub struct DropWidget<'a> {
    about_to_drop: bool,
    is_clicked: &'a mut bool,
}

impl<'a> DropWidget<'a> {
    pub fn new(about_to_drop: bool, is_clicked: &'a mut bool) -> Self {
        Self {
            about_to_drop,
            is_clicked,
        }
    }
}

impl<'a> Widget for DropWidget<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        // Drag and drop guide
        ui.horizontal(|ui| {
            let start_pos = Pos2::new(ui.next_widget_position().x, ui.min_rect().top());
            let drop_rect = Rect::from_min_size(start_pos, Vec2::new(ui.available_width(), 50.0));
            let drop_element = ui.allocate_rect(drop_rect, Sense::click());
            *self.is_clicked = drop_element.clicked();

            //     let rect = ui.allocate_rect(drop_rect, Sense::hover());
            // let hovering_files = ctx.input(|i| i.raw.hovered_files.clone());
            // let pasted_content = ctx.input(|i| i.raw.);
            let color = ui.style().visuals.panel_fill;
            let stroke = if self.about_to_drop {
                Stroke::new(2.0, Color32::LIGHT_GREEN)
            } else if drop_element.hovered() {
                Stroke::new(2.0, Color32::WHITE.gamma_multiply(1.0))
            } else {
                Stroke::new(2.0, Color32::WHITE.gamma_multiply(0.5))
            };
            ui.painter()
                .rect(drop_rect, Rounding::from(2.5), color, stroke);
            ui.painter().text(
                drop_rect.center(),
                Align2::CENTER_CENTER,
                "(1) Paste a magnet URL. \
                (2) Drag and drop a torrent file here. \
                (3) Click to select a file manually.",
                FontId::default(),
                ui.style().visuals.text_color(),
            );
        });
        ui.response()
    }
}
