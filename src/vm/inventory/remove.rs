use crate::save::common::save_slot::GaItem;

use super::{InventoryItemViewModel, InventoryViewModel};

impl InventoryViewModel {
    /// Removes a single common item (weapon/armor/talisman/aow/normal item) from the given
    /// storage (0 = held, 1 = storage box) by its unique gaitem handle.
    ///
    /// The underlying storage array is a fixed-size slot list mirroring the save file's format
    /// (in-use entries packed at the front, followed by empty padding), so removal shifts
    /// everything after it down by one and re-pads the tail with an empty slot to keep the
    /// array's length intact.
    pub fn remove_common_item(&mut self, storage_index: usize, ga_item_handle: u32) {
        let storage = match self.storage.get_mut(storage_index) {
            Some(storage) => storage,
            None => return,
        };
        let pos = match storage.common_items.iter().position(|i| i.ga_item_handle == ga_item_handle) {
            Some(pos) => pos,
            None => return,
        };
        if pos as u32 >= storage.common_item_count {
            return;
        }
        storage.common_items.remove(pos);
        storage.common_items.push(InventoryItemViewModel::default());
        storage.common_item_count = storage.common_item_count.saturating_sub(1);

        self.changed = true;
        self.filter();
    }

    /// Removes a single key item from the given storage (0 = held, 1 = storage box) by its
    /// unique gaitem handle. Same fixed-length-array handling as `remove_common_item`.
    pub fn remove_key_item(&mut self, storage_index: usize, ga_item_handle: u32) {
        let storage = match self.storage.get_mut(storage_index) {
            Some(storage) => storage,
            None => return,
        };
        let pos = match storage.key_items.iter().position(|i| i.ga_item_handle == ga_item_handle) {
            Some(pos) => pos,
            None => return,
        };
        if pos as u32 >= storage.key_item_count {
            return;
        }
        storage.key_items.remove(pos);
        storage.key_items.push(InventoryItemViewModel::default());
        storage.key_item_count = storage.key_item_count.saturating_sub(1);

        self.changed = true;
        self.filter();
    }

    /// Wipes every item from held inventory and the storage box (both common and key items),
    /// along with the underlying gaitem instance records and projectile list.
    ///
    /// Currently equipped gear is left untouched -- if a cleared item was equipped, its
    /// equipment slot will keep referencing a gaitem handle that no longer resolves to
    /// anything. Review the Equipment screen afterward and re-equip as needed.
    pub fn clear_all_inventory(&mut self) {
        for storage in self.storage.iter_mut() {
            let common_len = storage.common_items.len();
            storage.common_items = vec![InventoryItemViewModel::default(); common_len];
            storage.common_item_count = 0;

            let key_len = storage.key_items.len();
            storage.key_items = vec![InventoryItemViewModel::default(); key_len];
            storage.key_item_count = 0;
        }

        // gaitem_map mirrors a fixed-size array in the save format -- keep its length, just
        // reset every entry to empty rather than shrinking it.
        let gaitem_map_len = self.gaitem_map.len();
        self.gaitem_map = vec![GaItem::default(); gaitem_map_len];
        self.next_aow_index = 0;
        self.next_armament_or_armor_index = 0;

        self.gaitem_data = Default::default();
        self.projectile_list = Default::default();

        self.changed = true;
        self.log.insert(0, "Cleared all inventory items. Check the Equipment screen -- anything that was equipped now points to a removed item.".to_string());
        self.filter();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vm::inventory::{InventoryGaitemType, InventoryStorage};

    fn item(handle: u32, r#type: InventoryGaitemType) -> InventoryItemViewModel {
        InventoryItemViewModel {
            ga_item_handle: handle,
            r#type,
            ..Default::default()
        }
    }

    // Fixed size mirroring the save file's on-disk array (kept small here for test speed --
    // the invariant under test, that the array length never changes, holds regardless of size).
    const FIXED_LEN: usize = 8;

    fn storage_with_items(items: Vec<InventoryItemViewModel>) -> InventoryStorage {
        let mut storage = InventoryStorage::default();
        storage.common_item_count = items.len() as u32;
        storage.common_items = items;
        while storage.common_items.len() < FIXED_LEN {
            storage.common_items.push(InventoryItemViewModel::default());
        }
        storage
    }

    #[test]
    fn remove_common_item_preserves_fixed_array_length_and_shifts_remaining_items() {
        let mut vm = InventoryViewModel::default();
        vm.storage = vec![
            storage_with_items(vec![
                item(1, InventoryGaitemType::ITEM),
                item(2, InventoryGaitemType::ITEM),
                item(3, InventoryGaitemType::ITEM),
            ]),
            InventoryStorage::default(),
        ];

        vm.remove_common_item(0, 2);

        assert_eq!(vm.storage[0].common_items.len(), FIXED_LEN, "array must stay the fixed on-disk length");
        assert_eq!(vm.storage[0].common_item_count, 2);
        assert_eq!(vm.storage[0].common_items[0].ga_item_handle, 1);
        assert_eq!(vm.storage[0].common_items[1].ga_item_handle, 3, "item after the removed one must shift down");
        assert!(vm.changed);
    }

    #[test]
    fn clear_all_inventory_preserves_fixed_array_lengths_and_resets_counts() {
        let mut vm = InventoryViewModel::default();
        vm.storage = vec![
            storage_with_items(vec![item(1, InventoryGaitemType::WEAPON), item(2, InventoryGaitemType::ITEM)]),
            storage_with_items(vec![item(3, InventoryGaitemType::ARMOR)]),
        ];
        vm.gaitem_map = vec![GaItem::default(); 5];

        vm.clear_all_inventory();

        for storage in &vm.storage {
            assert_eq!(storage.common_items.len(), FIXED_LEN);
            assert_eq!(storage.common_item_count, 0);
            assert!(storage.common_items.iter().all(|i| i.ga_item_handle == 0));
        }
        assert_eq!(vm.gaitem_map.len(), 5, "gaitem_map must keep its fixed length");
    }
}
