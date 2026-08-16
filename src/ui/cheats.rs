pub mod cheats {
    use eframe::egui::{self, Ui};
    use crate::vm::{
        inventory::InventoryItemType,
        regulation::regulation_view_model::RegulationItemViewModel,
        vm::vm::ViewModel,
    };

    const MAX_SOULS: u32 = 999_999_999;
    // "Rune Arc" in item_name.rs / db/items.rs (id 190, i.e. 0x400000BE with the common-item mask).
    const RUNE_ARC_ID: u32 = 190;

    #[derive(Default, Clone, Copy, PartialEq)]
    pub enum CheatsRoute {
        #[default] Souls,
        Stats,
        RuneArcs,
        Flasks,
        Unlocks,
    }

    pub fn cheats(ui: &mut Ui, vm: &mut ViewModel, cheats_route: &mut CheatsRoute) {
        egui::SidePanel::left("cheats_menu").show(ui.ctx(), |ui| {
            egui::ScrollArea::vertical()
                .id_source("cheats_menu_scroll")
                .show(ui, |ui| {
                    ui.vertical(|ui| {
                        let souls = ui.add_sized([120., 40.], egui::Button::new("Souls"));
                        let stats = ui.add_sized([120., 40.], egui::Button::new("Stats"));
                        let rune_arcs = ui.add_sized([120., 40.], egui::Button::new("Rune Arcs"));
                        let flasks = ui.add_sized([120., 40.], egui::Button::new("Flasks"));
                        let unlocks = ui.add_sized([120., 40.], egui::Button::new("Unlocks"));

                        if souls.clicked() { *cheats_route = CheatsRoute::Souls; }
                        if stats.clicked() { *cheats_route = CheatsRoute::Stats; }
                        if rune_arcs.clicked() { *cheats_route = CheatsRoute::RuneArcs; }
                        if flasks.clicked() { *cheats_route = CheatsRoute::Flasks; }
                        if unlocks.clicked() { *cheats_route = CheatsRoute::Unlocks; }

                        match cheats_route {
                            CheatsRoute::Souls => { souls.highlight(); }
                            CheatsRoute::Stats => { stats.highlight(); }
                            CheatsRoute::RuneArcs => { rune_arcs.highlight(); }
                            CheatsRoute::Flasks => { flasks.highlight(); }
                            CheatsRoute::Unlocks => { unlocks.highlight(); }
                        }
                    })
                });
        });

        egui::CentralPanel::default().show(ui.ctx(), |ui| {
            match cheats_route {
                CheatsRoute::Souls => souls_cheats(ui, vm),
                CheatsRoute::Stats => stats_cheats(ui, vm),
                CheatsRoute::RuneArcs => rune_arc_cheats(ui, vm),
                CheatsRoute::Flasks => flask_cheats(ui, vm),
                CheatsRoute::Unlocks => unlock_cheats(ui, vm),
            }
        });
    }

    fn souls_cheats(ui: &mut Ui, vm: &mut ViewModel) {
        const AMOUNTS: [(&str, u32); 7] = [
            ("+1,000", 1_000),
            ("+10,000", 10_000),
            ("+50,000", 50_000),
            ("+100,000", 100_000),
            ("+500,000", 500_000),
            ("+1,000,000", 1_000_000),
            ("+10,000,000", 10_000_000),
        ];

        ui.heading("Add Souls");
        ui.add_space(8.0);
        ui.label(format!("Current Souls: {:09}", vm.slots[vm.index].stats_vm.souls));
        ui.add_space(8.0);

        ui.horizontal_wrapped(|ui| {
            for (label, amount) in AMOUNTS {
                if ui.add_sized([110., 30.], egui::Button::new(label)).clicked() {
                    let souls = &mut vm.slots[vm.index].stats_vm.souls;
                    *souls = souls.saturating_add(amount).min(MAX_SOULS);
                }
            }
            if ui.add_sized([110., 30.], egui::Button::new("Max")).clicked() {
                vm.slots[vm.index].stats_vm.souls = MAX_SOULS;
            }
        });
    }

    fn stats_cheats(ui: &mut Ui, vm: &mut ViewModel) {
        ui.heading("Stats");
        ui.add_space(8.0);
        ui.label("Sets every attribute (Vigor, Mind, Endurance, Strength, Dexterity, Intelligence, Faith, Arcane) to 99.");
        ui.add_space(8.0);

        if ui.add_sized([160., 30.], egui::Button::new("Max All Stats")).clicked() {
            let stats_vm = &mut vm.slots[vm.index].stats_vm;
            stats_vm.vigor = 99;
            stats_vm.mind = 99;
            stats_vm.endurance = 99;
            stats_vm.strength = 99;
            stats_vm.dexterity = 99;
            stats_vm.intelligence = 99;
            stats_vm.faith = 99;
            stats_vm.arcane = 99;
        }
    }

    fn rune_arc_cheats(ui: &mut Ui, vm: &mut ViewModel) {
        const AMOUNTS: [(&str, i16); 4] = [
            ("+1", 1),
            ("+5", 5),
            ("+10", 10),
            ("+99", 99),
        ];

        ui.heading("Add Rune Arcs");
        ui.add_space(8.0);
        ui.label("Adds Rune Arcs to the held inventory (capped at the item's normal carry limit).");
        ui.add_space(8.0);

        ui.horizontal_wrapped(|ui| {
            for (label, amount) in AMOUNTS {
                if ui.add_sized([80., 30.], egui::Button::new(label)).clicked() {
                    vm.slots[vm.index].inventory_vm.add_to_inventory(&RegulationItemViewModel {
                        id: RUNE_ARC_ID,
                        name: "Rune Arc".to_string(),
                        is_key_item: false,
                        item_type: InventoryItemType::ITEM,
                        quantity: Some(amount),
                        ..Default::default()
                    });
                }
            }
        });
    }

    fn flask_cheats(ui: &mut Ui, vm: &mut ViewModel) {
        ui.heading("Flasks");
        ui.add_space(8.0);
        ui.label("Refills every held Flask of Crimson/Cerulean Tears to its max charge count. Only tops off flasks you already have -- it won't grant a flask you haven't unlocked.");
        ui.add_space(8.0);

        if ui.add_sized([180., 30.], egui::Button::new("Max Out Flask Charges")).clicked() {
            vm.slots[vm.index].inventory_vm.max_out_flask_charges();
        }
    }

    fn unlock_cheats(ui: &mut Ui, vm: &mut ViewModel) {
        ui.heading("Unlocks");
        ui.add_space(8.0);
        ui.label("Bulk-grants an entire item category in one click, same as ticking every box on the Add Item -> Bulk screen.");
        ui.add_space(8.0);

        ui.vertical(|ui| {
            if ui.add_sized([220., 30.], egui::Button::new("Unlock All Weapons")).clicked() {
                vm.slots[vm.index].inventory_vm.unlock_all_weapons();
            }
            ui.add_space(4.0);
            if ui.add_sized([220., 30.], egui::Button::new("Unlock All Armor")).clicked() {
                vm.slots[vm.index].inventory_vm.unlock_all_armor();
            }
            ui.add_space(4.0);
            if ui.add_sized([220., 30.], egui::Button::new("Unlock All Talismans")).clicked() {
                vm.slots[vm.index].inventory_vm.unlock_all_talismans();
            }
            ui.add_space(4.0);
            if ui.add_sized([220., 30.], egui::Button::new("Unlock All Ashes of War")).clicked() {
                vm.slots[vm.index].inventory_vm.unlock_all_ashes_of_war();
            }
            ui.add_space(4.0);
            if ui.add_sized([220., 30.], egui::Button::new("Unlock All Spells")).clicked() {
                vm.slots[vm.index].inventory_vm.unlock_all_spells();
            }
            ui.add_space(4.0);
            if ui.add_sized([220., 30.], egui::Button::new("Unlock All Crafting Materials")).clicked() {
                vm.slots[vm.index].inventory_vm.unlock_all_crafting_materials();
            }
            ui.add_space(4.0);
            if ui.add_sized([220., 30.], egui::Button::new("Unlock All Consumables")).clicked() {
                vm.slots[vm.index].inventory_vm.unlock_all_consumables();
            }
            ui.add_space(4.0);
            if ui.add_sized([220., 30.], egui::Button::new("Unlock All Bell Bearings")).clicked() {
                vm.slots[vm.index].inventory_vm.unlock_all_bell_bearings();
            }
            ui.add_space(4.0);
            if ui.add_sized([220., 30.], egui::Button::new("Unlock All Crystal Tears")).clicked() {
                vm.slots[vm.index].inventory_vm.unlock_all_crystal_tears();
            }
            ui.add_space(4.0);
            if ui.add_sized([220., 30.], egui::Button::new("Unlock All Spirit Ashes")).clicked() {
                vm.slots[vm.index].inventory_vm.unlock_all_spirit_ashes();
            }
            ui.add_space(4.0);
            if ui.add_sized([220., 30.], egui::Button::new("Unlock All Sites of Grace")).clicked() {
                vm.slots[vm.index].events_vm.graces.values_mut().for_each(|on| *on = true);
            }
        });
    }
}
