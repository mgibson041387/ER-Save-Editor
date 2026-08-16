mod vm;
mod save;
mod util;
mod read;
mod write;
mod ui;
mod db;

#[cfg(not(target_arch = "wasm32"))]
pub mod native;
#[cfg(target_arch = "wasm32")]
pub mod web_theme;

use std::sync::mpsc::{channel, Receiver};

use eframe::{egui::{self, text::LayoutJob, Align, FontSelection, Id, LayerId, Layout, Order, RichText, Style}, epaint::Color32};
use save::save::save::{Save, SaveType};
use ui::{cheats::cheats::{cheats, CheatsRoute}, equipment::equipment::equipment, events::events::events, general::general::general, importer::import::character_importer, inventory::inventory::inventory::inventory, menu::menu::{menu, Route}, none::none::none, regions::regions::regions, stats::stats::stats};
use vm::{importer::general_view_model::ImporterViewModel, vm::vm::ViewModel};
use crate::write::write::Write as w;

pub const WINDOW_WIDTH: f32 = 1920.;
pub const WINDOW_HEIGHT: f32 = 960.;

/// Runs an async task to completion in the background, on whichever platform we're on.
/// Native: a dedicated OS thread. Web: the browser's microtask queue.
#[cfg(not(target_arch = "wasm32"))]
fn spawn_async(fut: impl std::future::Future<Output = ()> + Send + 'static) {
    std::thread::spawn(move || pollster::block_on(fut));
}
#[cfg(target_arch = "wasm32")]
fn spawn_async(fut: impl std::future::Future<Output = ()> + 'static) {
    wasm_bindgen_futures::spawn_local(fut);
}

/// Writes to a picked file, backing up whatever's already there first. On native this can
/// inspect the real filesystem; on web there's no filesystem to overwrite in the first place
/// (a browser "save" is always a fresh download), so there's nothing to back up.
#[cfg(not(target_arch = "wasm32"))]
async fn write_with_backup(handle: &rfd::FileHandle, bytes: &[u8]) -> std::io::Result<()> {
    let path = handle.path();
    if path.exists() {
        let mut backup_path = path.as_os_str().to_owned();
        backup_path.push(".bak");
        std::fs::copy(path, &backup_path)?;
    }
    handle.write(bytes).await
}
#[cfg(target_arch = "wasm32")]
async fn write_with_backup(handle: &rfd::FileHandle, bytes: &[u8]) -> std::io::Result<()> {
    handle.write(bytes).await
}

type OpenResult = Option<(String, Vec<u8>)>;
type SaveResult = Result<String, String>;

pub struct App {
    save: Save,
    vm: ViewModel,
    loaded_file_name: Option<String>,
    current_route: Route,
    current_cheat_route: CheatsRoute,
    importer_vm: ImporterViewModel,
    importer_open: bool,

    save_message: Option<(String, bool)>,

    open_rx: Option<Receiver<OpenResult>>,
    save_rx: Option<Receiver<SaveResult>>,
    import_rx: Option<Receiver<OpenResult>>,
}

impl App {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            save: Save::default(),
            vm: ViewModel::default(),
            loaded_file_name: None,
            current_route: Route::None,
            current_cheat_route: CheatsRoute::default(),
            importer_vm: Default::default(),
            importer_open: Default::default(),
            save_message: None,
            open_rx: None,
            save_rx: None,
            import_rx: None,
        }
    }

    fn open_from_bytes(&mut self, display_name: String, bytes: &[u8]) {
        match Save::from_bytes(bytes) {
            Ok(save) => {
                self.save = save;
                self.vm = ViewModel::from_save(&self.save);
                self.loaded_file_name = Some(display_name);
            }
            Err(err) => {
                eprintln!("Failed to open save file {display_name}: {err}");
                self.save = Save::default();
                self.vm = ViewModel::default();
                self.vm.active = Some(false);
                self.vm.invalid_reason = Some(format!("failed to parse save file: {err}"));
            }
        }
    }

    fn start_open(&mut self) {
        let (tx, rx) = channel();
        self.open_rx = Some(rx);
        spawn_async(async move {
            let file = rfd::AsyncFileDialog::new()
                .add_filter("SL2", &["sl2"])
                .add_filter("TXT", &["txt"])
                .add_filter("All files", &["*"])
                .pick_file()
                .await;
            let result = match file {
                Some(handle) => Some((handle.file_name(), handle.read().await)),
                None => None,
            };
            let _ = tx.send(result);
        });
    }

    fn start_import(&mut self) {
        let (tx, rx) = channel();
        self.import_rx = Some(rx);
        spawn_async(async move {
            let file = rfd::AsyncFileDialog::new()
                .add_filter("SL2", &["sl2"])
                .add_filter("TXT", &["txt"])
                .add_filter("All files", &["*"])
                .pick_file()
                .await;
            let result = match file {
                Some(handle) => Some((handle.file_name(), handle.read().await)),
                None => None,
            };
            let _ = tx.send(result);
        });
    }

    fn start_save(&mut self) {
        self.vm.update_save(&mut self.save.save_type);

        let bytes = match self.save.write() {
            Ok(bytes) => bytes,
            Err(err) => {
                self.save_message = Some((format!("Failed to serialize save data: {err}"), true));
                return;
            }
        };

        let suggested_name = self.loaded_file_name.clone().unwrap_or_else(|| "ER0000.sl2".to_string());

        let (tx, rx) = channel();
        self.save_rx = Some(rx);
        spawn_async(async move {
            let file = rfd::AsyncFileDialog::new()
                .add_filter("SL2", &["sl2"])
                .set_file_name(&suggested_name)
                .save_file()
                .await;
            let result: SaveResult = match file {
                Some(handle) => write_with_backup(&handle, &bytes).await
                    .map(|_| handle.file_name())
                    .map_err(|e| e.to_string()),
                None => Err(String::new()), // cancelled -- no toast
            };
            let _ = tx.send(result);
        });
    }

    fn poll_async_results(&mut self) {
        if let Some(rx) = &self.open_rx {
            if let Ok(result) = rx.try_recv() {
                if let Some((name, bytes)) = result {
                    self.open_from_bytes(name, &bytes);
                }
                self.open_rx = None;
            }
        }

        if let Some(rx) = &self.import_rx {
            if let Ok(result) = rx.try_recv() {
                if let Some((_name, bytes)) = result {
                    match Save::from_bytes(&bytes) {
                        Ok(save) => {
                            self.importer_vm = ImporterViewModel::new(save, &self.vm);
                            self.importer_open = true;
                        }
                        Err(err) => {
                            self.save_message = Some((format!("Failed to read character import file: {err}"), true));
                        }
                    }
                }
                self.import_rx = None;
            }
        }

        if let Some(rx) = &self.save_rx {
            if let Ok(result) = rx.try_recv() {
                match result {
                    Ok(name) => self.save_message = Some((format!("Saved to {name}"), false)),
                    Err(err) if err.is_empty() => {} // cancelled
                    Err(err) => self.save_message = Some((format!("Failed to save: {err}"), true)),
                }
                self.save_rx = None;
            }
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // The 1.75x zoom was tuned for native's fixed 1920x960 window. The web canvas has no
        // such guarantee -- it just fills whatever the actual browser viewport is, which can be
        // much smaller, pushing the side-panel nav buttons (General/Stats/Equipment/...)
        // partially or fully off-screen even though they still visually render. Not scaling up
        // on web keeps the layout inside whatever space is actually available.
        #[cfg(not(target_arch = "wasm32"))]
        ctx.set_zoom_factor(1.75);

        self.poll_async_results();

        // Save result toast
        if let Some((message, is_error)) = self.save_message.clone() {
            egui::TopBottomPanel::bottom("save_status").show(ctx, |ui| {
                ui.horizontal(|ui| {
                    let color = if is_error { Color32::DARK_RED } else { Color32::DARK_GREEN };
                    ui.label(RichText::new(&message).color(color));
                    if ui.small_button("x").clicked() {
                        self.save_message = None;
                    }
                });
            });
        }

        // TOP PANEL
        egui::TopBottomPanel::top("toolbar").default_height(35.).show(ctx, |ui| {
            ui.columns(2, |uis|{
                uis[0].with_layout(Layout::left_to_right(Align::Center),| ui| {
                    if ui.button(egui::RichText::new(format!("{} open", egui_phosphor::regular::FOLDER_OPEN))).clicked() {
                        self.start_open();
                    }
                    if ui.button(egui::RichText::new(format!("{} save", egui_phosphor::regular::FLOPPY_DISK))).clicked() {
                        self.start_save();
                    }
                });

                uis[1].with_layout(Layout::right_to_left(egui::Align::Center),|ui| {
                    let import_button = egui::widgets::Button::new(egui::RichText::new(format!("{} Import Character", egui_phosphor::regular::DOWNLOAD_SIMPLE)));
                    if ui.add_enabled(!self.vm.steam_id.is_empty(), import_button).clicked() {
                        self.start_import();
                    }
                    character_importer(ui, &mut self.importer_open, &mut self.importer_vm, &mut self.save, &mut self.vm);
                });
            });

        });

        // TOP PANEL
        egui::TopBottomPanel::top("top").show(ctx, |ui| {
            if self.loaded_file_name.is_some() {
                let save_type = match self.save.save_type {
                    SaveType::Unknown => {
                        "Platform: Unknown"
                    }
                    SaveType::PC(_) => {
                        "Platform: PC"
                    }
                    SaveType::PlayStation(_) => {
                        "Platform: Playstation"
                    },
                };

                ui.columns(2,| uis| {
                    if self.vm.active.is_some_and(|valid| valid) {
                        egui::Frame::none().show(&mut uis[1], |ui| {
                            let steam_id_text_edit = egui::widgets::TextEdit::singleline(&mut self.vm.steam_id)
                            .char_limit(17)
                            .desired_width(125.);
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.label(format!("Character: {}", self.vm.slots[self.vm.index].general_vm.character_name));

                                match self.save.save_type {
                                    SaveType::Unknown => {},
                                    SaveType::PC(_) => {
                                        let steam_id_text_edit = ui.add(steam_id_text_edit).labelled_by(ui.label("Steam Id:").id);
                                        if steam_id_text_edit.hovered() {
                                            egui::popup::show_tooltip(ui.ctx(), steam_id_text_edit.id, |ui|{
                                                ui.label(egui::RichText::new("Important: This needs to match the id of the steam account that will use this save!").size(8.0).color(Color32::PLACEHOLDER));
                                            });
                                        }
                                    },
                                    SaveType::PlayStation(_) => {},
                                };
                            });
                        });
                    }
                    egui::Frame::none().show(&mut uis[0], |ui| {
                        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                            ui.label(format!("{}",save_type));
                        });
                    });
                });
            }
        });

        // Character List Panel
        if self.vm.active.is_some_and(|valid| valid) {
            egui::SidePanel::left("characters").show(ctx, |ui| {
                egui::ScrollArea::vertical()
                    .id_source("left")
                    .show(ui, |ui| {
                        ui.vertical(|ui| {
                            for i in 0..0xA {
                                if self.vm.profile_summary[i].active {
                                    let button = ui.add_sized([120., 40.], egui::Button::new(&self.vm.slots[i].general_vm.character_name));
                                    if button.clicked() {self.vm.index = i;}
                                    if self.vm.index == i {button.highlight();}
                                }
                            }
                        })
                    });
            });

            // Slot Section Panel
            egui::SidePanel::left("slot_sections_menu").show(ctx, |ui| {
                egui::ScrollArea::vertical() .id_source("left") .show(ui, |ui| {
                    ui.vertical(|ui| {
                        menu(ui, self);
                    })
                });
            });

            // Main Content
            egui::CentralPanel::default().show(ctx, |ui| {
                match self.current_route {
                    Route::None => none(ui),
                    Route::General => general(ui, &mut self.vm),
                    Route::Stats => stats(ui, &mut self.vm),
                    Route::Equipment => equipment(ui, &mut self.vm),
                    Route::Inventory => inventory(ui, &mut self.vm),
                    Route::EventFlags => events(ui, &mut self.vm),
                    Route::Regions => regions(ui, &mut self.vm),
                    Route::Cheats => cheats(ui, &mut self.vm, &mut self.current_cheat_route),
                }
            });
        }
        // No file loaded View
        else {
            // Listen for dragged files and update path
            egui::CentralPanel::default().show(ctx, |ui| {
                // Check if hovering a file
                let path = ctx.input(|i| {
                    if let Some(file) = i.raw.hovered_files.first() {
                        if let Some(path) = &file.path {
                            return path.to_string_lossy().into_owned();
                        }
                        // Browsers don't expose the file name while it's still just hovering,
                        // only once dropped -- show a generic indicator instead.
                        return "Drop file to open".to_string();
                    }
                    "".to_string()
                });

                // Display indicator of hovering file
                ui.centered_and_justified(|ui| {
                    if !path.is_empty() {
                        let painter =
                            ctx.layer_painter(LayerId::new(Order::Foreground, Id::new("file_drop_target")));

                        let screen_rect = ctx.screen_rect();
                        painter.rect_filled(screen_rect, 0.0, Color32::from_black_alpha(96));
                        ui.label(egui::RichText::new(path));
                    }
                    else {
                        let style = Style::default();
                        let mut layout_job = LayoutJob::default();
                        if self.vm.active.is_some_and(|valid| !valid) {
                            let reason = self.vm.invalid_reason.as_deref().unwrap_or("unknown reason");
                            RichText::new(format!("Save file has irregular data!\n{reason}\n\n"))
                            .color(Color32::DARK_RED)
                            .append_to(
                                &mut layout_job,
                                &style,
                                FontSelection::Default,
                                Align::Center,
                            );
                        }
                        RichText::new("Drop a save file here or click 'Open' to browse")
                        .append_to(
                            &mut layout_job,
                            &style,
                            FontSelection::Default,
                            Align::Center,
                        );
                        ui.label(layout_job);
                    }
                });

                // Check a file that has been dropped in the window. Native: read straight off
                // disk (path is set). Web: the browser already read it into memory for us.
                let dropped = ctx.input(|i| i.raw.dropped_files.first().cloned());
                if let Some(file) = dropped {
                    if let Some(path) = &file.path {
                        match std::fs::read(path) {
                            Ok(bytes) => {
                                let name = path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
                                self.open_from_bytes(name, &bytes);
                            }
                            Err(err) => {
                                self.save_message = Some((format!("Failed to read dropped file: {err}"), true));
                            }
                        }
                    } else if let Some(bytes) = &file.bytes {
                        self.open_from_bytes(file.name.clone(), bytes);
                    }
                }
            });
        }
    }
}
