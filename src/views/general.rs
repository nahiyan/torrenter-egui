use egui::Widget;

pub struct GeneralWidget {}

impl Widget for GeneralWidget {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.label("General");
        ui.response()
    }
}
