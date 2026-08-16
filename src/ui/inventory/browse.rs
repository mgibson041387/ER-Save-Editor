use eframe::{egui::{self, Margin, TextFormat, Ui}, epaint::{text::LayoutJob, Color32}};
use crate::vm::{inventory::InventoryTypeRoute, vm::vm::ViewModel};

pub fn browse_inventory(ui: &mut Ui, vm:&mut ViewModel) {
    let inventory_vm = &mut vm.slots[vm.index].inventory_vm;

    ui.columns(2, |uis| {
        let equipped_button = uis[0].add_sized([100.,40.], egui::widgets::Button::new("Equipped"));
        let storage_button = uis[1].add_sized([100.,40.], egui::widgets::Button::new("Storage Box"));

        if equipped_button.clicked() {inventory_vm.at_storage_box = false;};
        if storage_button.clicked() {inventory_vm.at_storage_box = true;};

        if inventory_vm.at_storage_box {storage_button.highlight();}
        else {equipped_button.highlight();}
    });

    ui.add_space(6.);

    ui.horizontal(|ui| {
        let clear_button = ui.add(egui::widgets::Button::new(egui::RichText::new("Clear All Inventory").color(Color32::LIGHT_RED)));
        if clear_button.clicked() {
            inventory_vm.confirm_clear_all = true;
        }
        if inventory_vm.confirm_clear_all {
            egui::Window::new("Confirm Clear Inventory")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0., 0.])
                .show(ui.ctx(), |ui| {
                    ui.label("This removes every item from held inventory AND the storage box for this character.\nEquipped gear is not affected, but will point to a removed item until you re-equip something else.\nThis cannot be undone once you save.");
                    ui.add_space(8.);
                    ui.horizontal(|ui| {
                        if ui.button("Clear Everything").clicked() {
                            inventory_vm.clear_all_inventory();
                            inventory_vm.confirm_clear_all = false;
                        }
                        if ui.button("Cancel").clicked() {
                            inventory_vm.confirm_clear_all = false;
                        }
                    });
                });
        }
    });

    ui.add_space(6.);

    ui.columns(6,|uis| {
        let common_items = uis[0].add_sized([uis[0].available_width(), 40.], egui::Button::new("Common Item"));
        let key_items = uis[1].add_sized([uis[1].available_width(), 40.], egui::Button::new("Key Item"));
        let weapons = uis[2].add_sized([uis[2].available_width(), 40.], egui::Button::new("Weapons"));
        let armors = uis[3].add_sized([uis[3].available_width(), 40.], egui::Button::new("Armors"));
        let ashofwar = uis[4].add_sized([uis[4].available_width(), 40.], egui::Button::new("Ash of War"));
        let talismans = uis[5].add_sized([uis[5].available_width(), 40.], egui::Button::new("Talismans"));

        if common_items.clicked() {inventory_vm.current_type_route = InventoryTypeRoute::CommonItems}
        if key_items.clicked() {inventory_vm.current_type_route = InventoryTypeRoute::KeyItems}
        if weapons.clicked() {inventory_vm.current_type_route = InventoryTypeRoute::Weapons}
        if armors.clicked() {inventory_vm.current_type_route = InventoryTypeRoute::Armors}
        if ashofwar.clicked() {inventory_vm.current_type_route = InventoryTypeRoute::AshOfWar}
        if talismans.clicked() {inventory_vm.current_type_route = InventoryTypeRoute::Talismans}

        // Highlight active 
        match inventory_vm.current_type_route {
            InventoryTypeRoute::CommonItems => {common_items.highlight();},
            InventoryTypeRoute::KeyItems => {key_items.highlight();},
            InventoryTypeRoute::Weapons => {weapons.highlight();},
            InventoryTypeRoute::Armors => {armors.highlight();},
            InventoryTypeRoute::AshOfWar => {ashofwar.highlight();},
            InventoryTypeRoute::Talismans => {talismans.highlight();},
        }
    });

    ui.add_space(6.);

    ui.horizontal(|ui|{
        let height = 20.;
        let label = ui.label("Filter: ");
        if ui.add_sized([ui.available_size().x,height], egui::widgets::TextEdit::singleline(&mut inventory_vm.filter_text)).labelled_by(label.id).changed() {
            inventory_vm.filter();
        };
    });

    let mut frame = egui::Frame::none();
    frame.inner_margin = Margin { top: 8., left: 0., bottom: 8., right: 0. };
    frame.show(ui,|ui| {
        egui::Grid::new("browse_header").spacing([16., 16.]).min_col_width(ui.available_width()/5.).striped(true).show(ui, |ui| {
            // Table Header
            let mut job = LayoutJob::default();
            job.append("Item ID", 0., TextFormat{
                color: Color32::BLACK,
                ..Default::default()
            });
            ui.label(job);
    
            let mut job = LayoutJob::default();
            job.append("Item Name", 0., TextFormat{
                color: Color32::BLACK,
                ..Default::default()
                });
            ui.label(job);
    
            let mut job = LayoutJob::default();
            job.append("Quantity", 0., TextFormat{
                color: Color32::BLACK,
                ..Default::default()
                });
            ui.label(job);
    
            let mut job = LayoutJob::default();
            job.append("Acquisition Sort ID", 0., TextFormat{
                color: Color32::BLACK,
                ..Default::default()
            });
            ui.label(job);

            ui.label("");
            ui.end_row();
        });
    });

    let is_key_items = matches!(inventory_vm.current_type_route, InventoryTypeRoute::KeyItems);
    let storage_index = inventory_vm.at_storage_box as usize;
    let current_inventory_list = match inventory_vm.current_type_route {
        InventoryTypeRoute::CommonItems => &inventory_vm.storage[storage_index].filtered_items,
        InventoryTypeRoute::KeyItems => &inventory_vm.storage[storage_index].filtered_key_items,
        InventoryTypeRoute::Weapons => &inventory_vm.storage[storage_index].filtered_weapons,
        InventoryTypeRoute::Armors => &inventory_vm.storage[storage_index].filtered_armors,
        InventoryTypeRoute::AshOfWar => &inventory_vm.storage[storage_index].filtered_aows,
        InventoryTypeRoute::Talismans => &inventory_vm.storage[storage_index].filtered_accessories,
    };

    let mut to_remove: Option<u32> = None;
    let mut to_edit: Option<usize> = None;
    egui::ScrollArea::vertical().show_rows(ui, 10., current_inventory_list.len(), |ui, row_range| {
        egui::Grid::new("browse_body").spacing([8., 8.]).min_col_width(ui.available_width()/5.).striped(true).show(ui, |ui| {
            for i in row_range {
                let item = &current_inventory_list[i];
                // The whole row opens the edit popup when clicked, not just the Edit button --
                // matches the "click an item to edit it" expectation directly.
                if ui.add(egui::Label::new(format!("{}",item.item_id)).sense(egui::Sense::click())).clicked() {
                    to_edit = Some(i);
                }
                if ui.add(egui::Label::new(item.item_name.to_string()).wrap(true).sense(egui::Sense::click())).clicked() {
                    to_edit = Some(i);
                }
                if ui.add(egui::Label::new(format!("{}",item.quantity)).sense(egui::Sense::click())).clicked() {
                    to_edit = Some(i);
                }
                if ui.add(egui::Label::new(format!("{}",item.inventory_index)).sense(egui::Sense::click())).clicked() {
                    to_edit = Some(i);
                }
                ui.horizontal(|ui| {
                    if ui.small_button("Edit").clicked() {
                        to_edit = Some(i);
                    }
                    if ui.small_button("Remove").clicked() {
                        to_remove = Some(item.ga_item_handle);
                    }
                });
                ui.end_row();
            }
        });
    });

    if let Some(i) = to_edit {
        let item = current_inventory_list[i].clone();
        inventory_vm.start_editing(storage_index, is_key_items, &item);
    }

    if let Some(ga_item_handle) = to_remove {
        if is_key_items {
            inventory_vm.remove_key_item(storage_index, ga_item_handle);
        } else {
            inventory_vm.remove_common_item(storage_index, ga_item_handle);
        }
    }

    edit_popup(ui, inventory_vm);
}

fn edit_popup(ui: &mut Ui, inventory_vm: &mut crate::vm::inventory::InventoryViewModel) {
    let Some(editing) = inventory_vm.editing_item.clone() else { return; };

    let mut apply = false;
    let mut cancel = false;

    egui::Window::new("Edit Item")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0., 0.])
        .show(ui.ctx(), |ui| {
            ui.heading(&editing.item_name);
            ui.label(format!("Item ID: {}", editing.item_id));

            if let Some(image_url) = crate::vm::inventory::edit::image_url_for(&editing.item_name) {
                ui.add(egui::Image::from_uri(image_url).max_height(150.).max_width(150.));
            }

            ui.hyperlink_to(
                format!("{} View on Wiki", egui_phosphor::regular::ARROW_SQUARE_OUT),
                crate::vm::inventory::edit::wiki_url_for(&editing.item_name),
            );

            ui.add_space(8.);

            if editing.r#type == crate::vm::inventory::InventoryGaitemType::WEAPON {
                ui.horizontal(|ui| {
                    ui.label("Upgrade level:");
                    let mut level = inventory_vm.editing_item.as_ref().unwrap().upgrade_level_edit;
                    if ui.add(egui::DragValue::new(&mut level).clamp_range(0..=editing.max_upgrade_level)).changed() {
                        if let Some(editing) = &mut inventory_vm.editing_item {
                            editing.upgrade_level_edit = level;
                        }
                    }
                    // Jumps straight to the correct cap (+25 standard / +10 somber, whichever
                    // applies to this weapon) instead of dragging and guessing -- the DragValue
                    // above is already clamped so it can't be *exceeded*, but this avoids
                    // needing to know the right number to type in.
                    if ui.button(format!("Max (+{})", editing.max_upgrade_level)).clicked() {
                        if let Some(editing) = &mut inventory_vm.editing_item {
                            editing.upgrade_level_edit = editing.max_upgrade_level;
                        }
                    }
                });
            }

            if editing.r#type == crate::vm::inventory::InventoryGaitemType::ITEM {
                ui.horizontal(|ui| {
                    ui.label("Quantity:");
                    let mut quantity = inventory_vm.editing_item.as_ref().unwrap().quantity_edit;
                    if ui.add(egui::DragValue::new(&mut quantity).clamp_range(1..=999)).changed() {
                        if let Some(editing) = &mut inventory_vm.editing_item {
                            editing.quantity_edit = quantity;
                        }
                    }
                });
            }

            ui.add_space(8.);
            ui.horizontal(|ui| {
                if ui.button("Apply").clicked() {
                    apply = true;
                }
                if ui.button("Cancel").clicked() {
                    cancel = true;
                }
            });
        });

    if apply {
        inventory_vm.apply_editing();
    } else if cancel {
        inventory_vm.cancel_editing();
    }
}