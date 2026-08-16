#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    er_save_editor::native::run_native()
}

// When compiling to web using trunk:
#[cfg(target_arch = "wasm32")]
fn main() {
    console_error_panic_hook::set_once();
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        let result = eframe::WebRunner::new()
            .start(
                "the_canvas_id", // hardcoded, must match index.html
                web_options,
                Box::new(|cc| {
                    egui_extras::install_image_loaders(&cc.egui_ctx);
                    er_save_editor::web_theme::apply(&cc.egui_ctx);
                    Box::new(er_save_editor::App::new(cc))
                }),
            )
            .await;

        // eframe 0.26 doesn't remove the loading placeholder itself -- do it here so a
        // successful start doesn't leave the spinner stuck on top of the canvas forever,
        // and a failed start replaces it with a visible error instead of spinning forever
        // silently (the real error still also goes to the browser console via WebLogger).
        if let Some(loading_text) = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.get_element_by_id("loading_text"))
        {
            match result {
                Ok(_) => loading_text.remove(),
                Err(err) => {
                    loading_text.set_inner_html(&format!(
                        "<p>Failed to start ER Save Editor: {err:?}</p><p>Check the browser console for details.</p>"
                    ));
                }
            }
        }
    });
}
