use eframe::egui::{self, Color32, FontFamily, FontId, Rounding, Stroke, TextStyle};

const HEADING_FAMILY: &str = "cinzel";
const BODY_FAMILY: &str = "eb_garamond";

/// Applies a dark, gold-accented theme evoking Elden Ring's UI -- deep near-black panels, warm
/// parchment text, antique-gold highlights on selection/hover -- plus a serif display font for
/// headings and a period-appropriate body serif. Web-only: native keeps eframe's default look.
pub fn apply(ctx: &egui::Context) {
    apply_fonts(ctx);
    apply_visuals(ctx);
}

fn apply_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);
    egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Fill);

    fonts.font_data.insert(
        BODY_FAMILY.to_owned(),
        egui::FontData::from_static(include_bytes!("../assets/fonts/EBGaramond.ttf")),
    );
    fonts.font_data.insert(
        HEADING_FAMILY.to_owned(),
        egui::FontData::from_static(include_bytes!("../assets/fonts/Cinzel.ttf")),
    );

    // Body serif becomes the primary proportional font (used for Body/Button/Small unless
    // overridden below), with egui's default sans-serif kept as a fallback for any glyphs
    // the serif doesn't cover.
    fonts.families
        .entry(FontFamily::Proportional)
        .or_default()
        .insert(0, BODY_FAMILY.to_owned());

    // Separate named family for headings only.
    fonts.families.insert(
        FontFamily::Name(HEADING_FAMILY.into()),
        vec![HEADING_FAMILY.to_owned()],
    );

    ctx.set_fonts(fonts);

    ctx.style_mut(|style| {
        style.text_styles.insert(
            TextStyle::Heading,
            FontId::new(26.0, FontFamily::Name(HEADING_FAMILY.into())),
        );
    });
}

fn apply_visuals(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();

    // Palette: deep warm near-black backgrounds, aged parchment text, antique-gold accents.
    let bg_deepest = Color32::from_rgb(12, 10, 8);
    let bg_panel = Color32::from_rgb(20, 17, 14);
    let bg_window = Color32::from_rgb(26, 22, 18);
    let bg_widget = Color32::from_rgb(34, 29, 23);
    let bg_widget_hovered = Color32::from_rgb(48, 40, 30);
    let bg_widget_active = Color32::from_rgb(58, 46, 28);
    let text_parchment = Color32::from_rgb(222, 208, 182);
    let gold = Color32::from_rgb(198, 155, 82);
    let gold_bright = Color32::from_rgb(224, 184, 108);
    let gold_dim = Color32::from_rgb(120, 96, 58);

    visuals.override_text_color = Some(text_parchment);
    visuals.hyperlink_color = gold_bright;
    visuals.faint_bg_color = bg_widget;
    visuals.extreme_bg_color = bg_deepest;
    visuals.code_bg_color = bg_deepest;
    visuals.panel_fill = bg_panel;
    visuals.window_fill = bg_window;
    visuals.window_stroke = Stroke::new(1.0, gold_dim);
    visuals.window_rounding = Rounding::same(2.0);

    visuals.widgets.noninteractive.bg_fill = bg_panel;
    visuals.widgets.noninteractive.weak_bg_fill = bg_panel;
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, gold_dim);
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, text_parchment);

    visuals.widgets.inactive.bg_fill = bg_widget;
    visuals.widgets.inactive.weak_bg_fill = bg_widget;
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, gold_dim);
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, text_parchment);
    visuals.widgets.inactive.rounding = Rounding::same(2.0);

    visuals.widgets.hovered.bg_fill = bg_widget_hovered;
    visuals.widgets.hovered.weak_bg_fill = bg_widget_hovered;
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.5, gold);
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, gold_bright);
    visuals.widgets.hovered.rounding = Rounding::same(2.0);

    visuals.widgets.active.bg_fill = bg_widget_active;
    visuals.widgets.active.weak_bg_fill = bg_widget_active;
    visuals.widgets.active.bg_stroke = Stroke::new(1.5, gold_bright);
    visuals.widgets.active.fg_stroke = Stroke::new(1.0, gold_bright);
    visuals.widgets.active.rounding = Rounding::same(2.0);

    visuals.widgets.open.bg_fill = bg_widget_active;
    visuals.widgets.open.weak_bg_fill = bg_widget_active;
    visuals.widgets.open.bg_stroke = Stroke::new(1.0, gold);

    visuals.selection.bg_fill = gold_dim;
    visuals.selection.stroke = Stroke::new(1.0, gold_bright);

    visuals.window_highlight_topmost = false;

    ctx.set_visuals(visuals);
}
