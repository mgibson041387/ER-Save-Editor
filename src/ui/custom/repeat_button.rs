pub mod repeat_button {
    use eframe::egui::{self, Ui};

    const INITIAL_DELAY_SECS: f64 = 0.4;
    const REPEAT_INTERVAL_SECS: f64 = 0.08;

    /// A button that fires once per click, and then keeps firing on an interval for as long as
    /// it's held down -- matching the in-game level-up screen's press-and-hold stat adjustment.
    pub fn repeat_button(ui: &mut Ui, text: &str) -> bool {
        let response = ui.add_sized([24., 24.], egui::Button::new(text));
        let mut triggered = response.clicked();

        let held_id = response.id.with("repeat_button_held_since");
        let repeat_id = response.id.with("repeat_button_last_repeat");

        if response.is_pointer_button_down_on() {
            let now = ui.input(|i| i.time);
            let held_since = ui.data_mut(|d| *d.get_temp_mut_or(held_id, now));

            if now - held_since > INITIAL_DELAY_SECS {
                let last_repeat = ui.data_mut(|d| *d.get_temp_mut_or(repeat_id, held_since));
                if now - last_repeat > REPEAT_INTERVAL_SECS {
                    ui.data_mut(|d| d.insert_temp(repeat_id, now));
                    triggered = true;
                }
            }

            // Keep the UI ticking while held so the repeat timer above keeps advancing even
            // if nothing else on screen is requesting repaints.
            ui.ctx().request_repaint();
        } else {
            ui.data_mut(|d| {
                d.remove_temp::<f64>(held_id);
                d.remove_temp::<f64>(repeat_id);
            });
        }

        triggered
    }
}
