use std::time::Duration;

use egui::WidgetText;
use egui_toast::{Toast, ToastKind, ToastOptions, Toasts};

fn add_toast<I>(toasts: &mut Toasts, message: I, kind: ToastKind)
where
    I: Into<WidgetText>,
{
    toasts.add(Toast {
        text: message.into(),
        kind,
        options: ToastOptions::default()
            .duration(Duration::from_secs(5))
            .show_progress(true),
        ..Default::default()
    });
}

pub fn error<I>(toasts: &mut Toasts, message: I)
where
    I: Into<WidgetText>,
{
    add_toast(toasts, message, ToastKind::Error);
}

pub fn info<I>(toasts: &mut Toasts, message: I)
where
    I: Into<WidgetText>,
{
    add_toast(toasts, message, ToastKind::Info);
}
pub fn success<I>(toasts: &mut Toasts, message: I)
where
    I: Into<WidgetText>,
{
    add_toast(toasts, message, ToastKind::Success);
}
