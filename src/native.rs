use eframe::egui::{self, Rounding};
use rust_embed::RustEmbed;

use crate::{App, WINDOW_HEIGHT, WINDOW_WIDTH};

#[derive(RustEmbed)]
#[folder = "icon/"]
struct Asset;

pub fn run_native() -> eframe::Result<()> {
    // App Icon
    let mut app_icon = egui::IconData::default();

    let image = Asset::get("icon.png").expect("Failed to get image data").data;
    let icon = image::load_from_memory(&image).expect("Failed to open icon path").to_rgba8();
    let (icon_width, icon_height) = icon.dimensions();
    app_icon.rgba = icon.into_raw();
    app_icon.width = icon_width;
    app_icon.height = icon_height;

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([WINDOW_WIDTH, WINDOW_HEIGHT])
        .with_icon(app_icon),
        ..Default::default()
    };

    eframe::run_native("ER Save Editor 0.0.21", options, Box::new(|creation_context| {
        egui_extras::install_image_loaders(&creation_context.egui_ctx);

        let mut fonts = egui::FontDefinitions::default();
        egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);
        egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Fill);
        creation_context.egui_ctx.set_fonts(fonts);
        let mut visuals = creation_context.egui_ctx.style().visuals.clone();
        let rounding = 3.;
        visuals.window_rounding = Rounding::default().at_least(rounding);
        visuals.window_highlight_topmost = false;
        creation_context.egui_ctx.set_visuals(visuals);
        Box::new(App::new(creation_context))
    }))
}
