use crate::{
    db::weapon_name::weapon_name::WEAPON_NAME,
    util::regulation::Regulation,
};
use super::{EditingItem, InventoryGaitemType, InventoryItemViewModel, InventoryViewModel};

impl InventoryViewModel {
    /// Opens the edit popup for the given item (identified by its position in Browse).
    pub fn start_editing(&mut self, storage_index: usize, is_key_item: bool, item: &InventoryItemViewModel) {
        let max_upgrade_level = if item.r#type == InventoryGaitemType::WEAPON {
            let base_id = (item.item_id / 100) * 100;
            match Regulation::equip_weapon_params_map().get(&base_id) {
                Some(weapon_param) => if is_somber_reinforce_type(weapon_param.data.reinforceTypeId) { 10 } else { 25 },
                None => 25,
            }
        } else {
            0
        };

        self.editing_item = Some(EditingItem {
            storage_index,
            is_key_item,
            ga_item_handle: item.ga_item_handle,
            item_id: item.item_id,
            item_name: item.item_name.clone(),
            r#type: item.r#type.clone(),
            quantity_edit: item.quantity,
            upgrade_level_edit: if item.r#type == InventoryGaitemType::WEAPON { item.item_id % 100 } else { 0 },
            max_upgrade_level,
        });
    }

    pub fn cancel_editing(&mut self) {
        self.editing_item = None;
    }

    /// Applies whatever's currently in the edit popup's fields back to the underlying item.
    pub fn apply_editing(&mut self) {
        let Some(editing) = self.editing_item.clone() else { return; };

        match editing.r#type {
            InventoryGaitemType::WEAPON => {
                self.set_weapon_upgrade_level(editing.storage_index, editing.ga_item_handle, editing.upgrade_level_edit);
            }
            InventoryGaitemType::ITEM => {
                self.set_item_quantity(editing.storage_index, editing.is_key_item, editing.ga_item_handle, editing.quantity_edit);
            }
            _ => {}
        }

        self.editing_item = None;
        self.filter();
    }

    /// Changes an existing weapon's upgrade level in place, preserving its current affinity --
    /// the affinity is encoded as part of item_id too, but reliably decomposing an existing
    /// item_id into (base weapon, affinity) isn't derivable without risk of landing on the wrong
    /// weapon variant, so only the upgrade suffix (the low two digits) is changed here, matching
    /// the exact `(item_id/100)*100` convention already used for weapon name lookups elsewhere.
    fn set_weapon_upgrade_level(&mut self, storage_index: usize, ga_item_handle: u32, new_upgrade_level: u32) {
        let Some(storage) = self.storage.get_mut(storage_index) else { return; };
        let Some(item) = storage.common_items.iter_mut().find(|i| i.ga_item_handle == ga_item_handle) else { return; };

        let base_with_affinity = (item.item_id / 100) * 100;
        let new_item_id = base_with_affinity + new_upgrade_level;
        item.item_id = new_item_id;

        let display_name = match WEAPON_NAME.lock().unwrap().get(&base_with_affinity) {
            Some(name) => if new_upgrade_level > 0 { format!("{name} +{new_upgrade_level}") } else { name.to_string() },
            None => item.item_name.clone(),
        };
        item.item_name = display_name;

        if let Some(gaitem) = self.gaitem_map.iter_mut().find(|g| g.gaitem_handle == ga_item_handle) {
            gaitem.item_id = new_item_id;
        }

        self.changed = true;
    }

    pub(crate) fn set_item_quantity(&mut self, storage_index: usize, is_key_item: bool, ga_item_handle: u32, new_quantity: u32) {
        let Some(storage) = self.storage.get_mut(storage_index) else { return; };
        let items = if is_key_item { &mut storage.key_items } else { &mut storage.common_items };
        if let Some(item) = items.iter_mut().find(|i| i.ga_item_handle == ga_item_handle) {
            item.quantity = new_quantity;
            self.changed = true;
        }
    }

    /// Tops off every Flask of Crimson/Cerulean Tears currently held (not in the storage box) to
    /// its maximum charge count for its current upgrade level. Only refills flasks the character
    /// already has -- it doesn't grant a flask that hasn't been unlocked yet.
    pub fn max_out_flask_charges(&mut self) {
        let Some(storage) = self.storage.get_mut(0) else { return; };
        let mut changed = false;
        for item in storage.common_items.iter_mut() {
            if item.item_name.starts_with("Flask of Crimson Tears") || item.item_name.starts_with("Flask of Cerulean Tears") {
                if let Some(item_param) = Regulation::equip_goods_param_map().get(&item.item_id) {
                    item.quantity = item_param.data.maxNum as u32;
                    changed = true;
                }
            }
        }
        if changed {
            self.changed = true;
        }
    }
}

/// Same somber-reinforcement-type check used in vm.rs's match-making level calculation.
fn is_somber_reinforce_type(reinforce_type_id: i16) -> bool {
    reinforce_type_id != 0 && (
        reinforce_type_id % 2200 == 0 ||
        reinforce_type_id % 2400 == 0 ||
        reinforce_type_id % 3200 == 0 ||
        reinforce_type_id % 3300 == 0 ||
        reinforce_type_id % 8300 == 0 ||
        reinforce_type_id % 8500 == 0
    )
}

const WEAPON_AFFINITY_PREFIXES: [&str; 12] = [
    "Heavy ", "Keen ", "Quality ", "Flame Art ", "Fire ", "Lightning ", "Sacred ",
    "Magic ", "Cold ", "Poison ", "Blood ", "Occult ",
];

/// Strips both a weapon affinity prefix ("Heavy Uchigatana" -> "Uchigatana") and an
/// upgrade-level suffix (" +9") from a display name, matching the single base-name identity
/// the wiki actually has one page for. Used for both the wiki link and the image lookup, so
/// both resolve identically for the same underlying weapon regardless of infusion/level.
fn strip_weapon_name_decorations(item_name: &str) -> &str {
    let without_suffix = match item_name.rfind(" +") {
        Some(pos) if item_name.len() > pos + 2 && item_name[pos + 2..].chars().all(|c| c.is_ascii_digit()) => &item_name[..pos],
        _ => item_name,
    };
    for prefix in WEAPON_AFFINITY_PREFIXES {
        if let Some(stripped) = without_suffix.strip_prefix(prefix) {
            return stripped;
        }
    }
    without_suffix
}

/// Best-effort Fextralife wiki page URL for an item, constructed from its display name.
/// Not guaranteed to resolve for every item (disambiguation pages, name mismatches), but works
/// for the common case and needs no per-item lookup or scraping.
pub fn wiki_url_for(item_name: &str) -> String {
    let base_name = strip_weapon_name_decorations(item_name);
    let encoded = base_name.trim().replace(' ', "+");
    format!("https://eldenring.wiki.fextralife.com/{encoded}")
}

/// Looks up a precomputed item picture URL (see db/item_image_url.rs), if one was found for
/// this item during the one-time wiki scrape. Falls back through the same base-name
/// normalization as `wiki_url_for` so infused/upgraded weapons still resolve.
pub fn image_url_for(item_name: &str) -> Option<&'static str> {
    let base_name = strip_weapon_name_decorations(item_name);
    crate::db::item_image_url::item_image_url::ITEM_IMAGE_URL.get(base_name).copied()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wiki_url_strips_upgrade_suffix() {
        assert_eq!(wiki_url_for("Uchigatana +9"), "https://eldenring.wiki.fextralife.com/Uchigatana");
        assert_eq!(wiki_url_for("Moonveil"), "https://eldenring.wiki.fextralife.com/Moonveil");
        assert_eq!(wiki_url_for("Golden Order Seal"), "https://eldenring.wiki.fextralife.com/Golden+Order+Seal");
        // Not actually an upgrade suffix (not purely digits after " +") -- must stay intact.
        assert_eq!(wiki_url_for("Add +Something"), "https://eldenring.wiki.fextralife.com/Add++Something");
    }

    #[test]
    fn set_weapon_upgrade_level_preserves_affinity_offset() {
        let mut vm = InventoryViewModel::default();
        vm.storage = vec![super::super::InventoryStorage::default(), super::super::InventoryStorage::default()];
        vm.storage[0].common_items.push(InventoryItemViewModel {
            ga_item_handle: 0x80000001,
            item_id: 1_000_200 + 5, // base 1000000 + Keen affinity (200) + upgrade level 5
            item_name: "Test Weapon +5".to_string(),
            quantity: 1,
            inventory_index: 0,
            equip_index: 0,
            r#type: InventoryGaitemType::WEAPON,
        });
        vm.gaitem_map.push(crate::save::common::save_slot::GaItem {
            gaitem_handle: 0x80000001,
            item_id: 1_000_200 + 5,
            ..Default::default()
        });

        vm.set_weapon_upgrade_level(0, 0x80000001, 10);

        assert_eq!(vm.storage[0].common_items[0].item_id, 1_000_200 + 10, "affinity offset (200) must survive, only the upgrade suffix changes");
        assert_eq!(vm.gaitem_map[0].item_id, 1_000_200 + 10, "gaitem_map entry must stay in sync with the storage entry");
        assert!(vm.changed);
    }

    #[test]
    fn set_item_quantity_updates_matching_item_only() {
        let mut vm = InventoryViewModel::default();
        vm.storage = vec![super::super::InventoryStorage::default()];
        vm.storage[0].common_items.push(InventoryItemViewModel {
            ga_item_handle: 0xB0000001,
            item_id: 500,
            item_name: "Rowa Fruit".to_string(),
            quantity: 3,
            inventory_index: 0,
            equip_index: 0,
            r#type: InventoryGaitemType::ITEM,
        });

        vm.set_item_quantity(0, false, 0xB0000001, 99);

        assert_eq!(vm.storage[0].common_items[0].quantity, 99);
        assert!(vm.changed);
    }
}
