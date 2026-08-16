pub mod validator {
    use std::collections::{HashMap, HashSet};
    use crate::{save::{common::save_slot::{EquipInventoryItem, GaItem}, save::save::Save}, util::{param_structs::EQUIP_PARAM_GEM_ST, params::params::Row, regulation::Regulation}, vm::{inventory::{InventoryGaitemType, InventoryItemType}, regulation::regulation_view_model::{GoodsType, ProtectorCategory, WepType}}};

    pub struct Validator;

    impl Validator {
        pub fn validate(save: &Save) -> Result<(), String> {
            // Get active characters
            for (index, active) in save.save_type.active_slots().iter().enumerate() {
                if *active {
                    Self::is_weapons_valid(save, index).map_err(|e| format!("slot {index}: weapons: {e}"))?;
                    Self::is_items_valid(save, index).map_err(|e| format!("slot {index}: items: {e}"))?;
                    Self::is_armor_valid(save, index).map_err(|e| format!("slot {index}: armor: {e}"))?;
                    Self::is_physics_valid(save, index).map_err(|e| format!("slot {index}: physick: {e}"))?;
                    Self::is_equipped_items_valid(save, index).map_err(|e| format!("slot {index}: equipped items: {e}"))?;
                }
            }
            Ok(())
        }

        fn is_weapons_valid(save: &Save, index: usize) -> Result<(), String> {
            let slot = save.save_type.get_slot(index);
            let weapons = Regulation::equip_weapon_params_map();
            let gems = Regulation::equip_gem_param_map();

            // Map weapons
            let ga_item_weapons = &slot.ga_items.iter()
            .filter(|gaitem| gaitem.gaitem_handle.to_le_bytes()[3] == 0)
            .map(|g| g)
            .collect::<Vec<&GaItem>>();

            // Map gems
            let ga_item_gems = &slot.ga_items.iter()
            .filter(|gaitem| gaitem.gaitem_handle.to_le_bytes()[3] == 0xC0)
            .map(|g| (g.gaitem_handle, g))
            .collect::<HashMap<u32,&GaItem>>();

            for weapon_ga_item in ga_item_weapons {
                let res_weapon = weapons.get(&weapon_ga_item.item_id);

                if res_weapon.is_none() && weapon_ga_item.item_id != 0xFFFFFFFF {
                    return Err(format!("weapon item_id {} ({:#x}) not found in regulation params (weapon param map has {} entries)", weapon_ga_item.item_id, weapon_ga_item.item_id, weapons.len()));
                }

                // Get currently infused Ash Of War
                let current_weapon_gem = weapon_ga_item.aow_gaitem_handle;

                // Skip rest of validation if there's no ash of war infused
                if current_weapon_gem == u32::MAX || current_weapon_gem == 0 {
                    continue;
                }

                // Look up the gem_param
                let gem_ga_item = match ga_item_gems.get(&current_weapon_gem) {
                    Some(gem_ga_item) => gem_ga_item,
                    None => return Err(format!("weapon item_id {} references aow_gaitem_handle {:#x} which has no matching gem gaitem entry", weapon_ga_item.item_id, current_weapon_gem)),
                };
                let res_gem = match gems.get(&gem_ga_item.item_id) {
                    Some(res_gem) => res_gem,
                    None => return Err(format!("ash of war item_id {} ({:#x}) not found in regulation gem params", gem_ga_item.item_id, gem_ga_item.item_id)),
                };

                let weapon_param = res_weapon.unwrap();

                // Ash of war on an item that doesn't support it.
                if weapon_param.data.gemMountType == 0 {
                    return Err(format!("weapon item_id {} has an ash of war attached but gemMountType is 0 (doesn't support one)", weapon_ga_item.item_id));
                }

                // Ash of war not valid
                if !Self::validate_attached_gem(WepType::from(weapon_param.data.wepType), res_gem) {
                    return Err(format!("ash of war item_id {} is not compatible with weapon item_id {}'s weapon type", gem_ga_item.item_id, weapon_ga_item.item_id));
                }
            }

            Ok(())
        }

        fn is_items_valid(save: &Save, index: usize) -> Result<(), String> {
            let inventory_common_items = &save.save_type.get_slot(index).equip_inventory_data.common_items;
            let storage_common_items = &save.save_type.get_slot(index).storage_inventory_data.common_items;
            Self::check_for_duplicate_items(inventory_common_items).map_err(|e| format!("held inventory: {e}"))?;
            Self::check_for_duplicate_items(storage_common_items).map_err(|e| format!("storage box: {e}"))?;
            Ok(())
        }

        fn is_armor_valid(save: &Save, index: usize) -> Result<(), String> {
            let head_protector_id = save.save_type.get_slot(index).chr_asm.head;
            let body_protector_id = save.save_type.get_slot(index).chr_asm.chest;
            let arms_protector_id = save.save_type.get_slot(index).chr_asm.arms;
            let legs_protector_id = save.save_type.get_slot(index).chr_asm.legs;

            Self::validate_armor_piece(head_protector_id, ProtectorCategory::Head).map_err(|e| format!("chr_asm head: {e}"))?;
            Self::validate_armor_piece(body_protector_id, ProtectorCategory::Body).map_err(|e| format!("chr_asm chest: {e}"))?;
            Self::validate_armor_piece(arms_protector_id, ProtectorCategory::Arms).map_err(|e| format!("chr_asm arms: {e}"))?;
            Self::validate_armor_piece(legs_protector_id, ProtectorCategory::Legs).map_err(|e| format!("chr_asm legs: {e}"))?;

            let head_protector_id = save.save_type.get_slot(index).equipped_items.head ^ InventoryItemType::ARMOR as u32;
            let body_protector_id = save.save_type.get_slot(index).equipped_items.chest ^ InventoryItemType::ARMOR as u32;
            let arms_protector_id = save.save_type.get_slot(index).equipped_items.arms ^ InventoryItemType::ARMOR as u32;
            let legs_protector_id = save.save_type.get_slot(index).equipped_items.legs ^ InventoryItemType::ARMOR as u32;

            Self::validate_armor_piece(head_protector_id, ProtectorCategory::Head).map_err(|e| format!("equipped_items head: {e}"))?;
            Self::validate_armor_piece(body_protector_id, ProtectorCategory::Body).map_err(|e| format!("equipped_items chest: {e}"))?;
            Self::validate_armor_piece(arms_protector_id, ProtectorCategory::Arms).map_err(|e| format!("equipped_items arms: {e}"))?;
            Self::validate_armor_piece(legs_protector_id, ProtectorCategory::Legs).map_err(|e| format!("equipped_items legs: {e}"))?;

            Ok(())
        }

        fn is_physics_valid(save: &Save, index: usize) -> Result<(), String> {
            let physics_slot1 = save.save_type.get_slot(index).equip_physics_data.slot1;
            let physics_slot2 = save.save_type.get_slot(index).equip_physics_data.slot2;

            // Check if same tear is in the both slots
            if physics_slot1 != u32::MAX && physics_slot2 != u32::MAX && physics_slot1 == physics_slot2 {
                return Err(format!("physick slot 1 and slot 2 both hold the same tear ({physics_slot1:#x})"));
            }

            // Check if physic slot 1 is of type wondrous physics
            if physics_slot1 != u32::MAX {
                let res_physics1_good = Regulation::equip_goods_param_map().get(&(physics_slot1 ^ InventoryGaitemType::ITEM as u32));
                if res_physics1_good.is_some_and(|p| GoodsType::from(p.data.goodsType) != GoodsType::WonderousPhysicsTears) {
                    return Err(format!("physick slot 1 item {physics_slot1:#x} is not a Wondrous Physick Tear"));
                }
            }

            // Check if physic slot 2 is of type wondrous physics
            if physics_slot2 != u32::MAX {
                let res_physics2_good = Regulation::equip_goods_param_map().get(&(physics_slot2 ^ InventoryGaitemType::ITEM as u32));
                if res_physics2_good.is_some_and(|p| GoodsType::from(p.data.goodsType) != GoodsType::WonderousPhysicsTears) {
                    return Err(format!("physick slot 2 item {physics_slot2:#x} is not a Wondrous Physick Tear"));
                }
            }

            Ok(())
        }

        fn is_equipped_items_valid(save: &Save, index: usize) -> Result<(), String> {
            let quick_slot_items = &save.save_type.get_slot(index).equip_item_data.quick_slot_items;
            let pouch_items = &save.save_type.get_slot(index).equip_item_data.pouch_items;

            // Check for invalid or duplicate quickslot items
            let mut item_ids = HashSet::new();
            for item in quick_slot_items.iter() {
                if item.item_id == 0 { continue; }
                if Regulation::equip_goods_param_map().get(&(item.item_id ^ InventoryGaitemType::ITEM as u32)).is_none() {
                    return Err(format!("quickslot item_id {} not found in regulation goods params", item.item_id));
                }
                if item_ids.contains(&item.item_id) {
                    return Err(format!("duplicate quickslot item_id {}", item.item_id));
                }
                item_ids.insert(item.item_id);
            }

            // Check for invalid or duplicate pouch items
            let mut item_ids = HashSet::new();
            for item in pouch_items.iter() {
                if item.item_id == 0 { continue; }
                if Regulation::equip_goods_param_map().get(&(item.item_id ^ InventoryGaitemType::ITEM as u32)).is_none() {
                    return Err(format!("pouch item_id {} not found in regulation goods params", item.item_id));
                }
                if item_ids.contains(&item.item_id) {
                    return Err(format!("duplicate pouch item_id {}", item.item_id));
                }
                item_ids.insert(item.item_id);
            }
            Ok(())
        }

        // region: utils

        fn validate_armor_piece(id: u32, protector_category: ProtectorCategory) -> Result<(), String> {
            let armor_piece = Regulation::equip_protectors_param_map().get(&id)
                .ok_or_else(|| format!("armor item_id {id} ({id:#x}) not found in regulation protector params"))?;
            let armor_piece_pc = ProtectorCategory::try_from(armor_piece.data.protectorCategory)
                .map_err(|_| format!("armor item_id {id} has an unrecognized protectorCategory {}", armor_piece.data.protectorCategory))?;
            if armor_piece_pc != protector_category {
                return Err(format!("armor item_id {id} has protectorCategory {armor_piece_pc:?}, expected {protector_category:?}"));
            }
            Ok(())
        }

        // Validates the infsued gem against the weapon type by looking it up in the game params.
        fn validate_attached_gem(wep_type: WepType, gem_param: &Row<EQUIP_PARAM_GEM_ST>) -> bool {
            match wep_type {
                WepType::Dagger => gem_param.data.canMountWep_Dagger(),
                WepType::StraightSword => gem_param.data.canMountWep_SwordNormal(),
                WepType::Greatsword => gem_param.data.canMountWep_SwordLarge(),
                WepType::ColossalSword => gem_param.data.canMountWep_SwordGigantic(),
                WepType::CurvedSword => gem_param.data.canMountWep_SaberNormal(),
                WepType::CurvedGreatsword => gem_param.data.canMountWep_SaberLarge(),
                WepType::Katana => gem_param.data.canMountWep_katana(),
                WepType::Twinblade => gem_param.data.canMountWep_SwordDoubleEdge(),
                WepType::ThrustingSword => gem_param.data.canMountWep_SwordPierce(),
                WepType::HeavyThrustingSword => gem_param.data.canMountWep_RapierHeavy(),
                WepType::Axe => gem_param.data.canMountWep_AxeNormal(),
                WepType::Greataxe => gem_param.data.canMountWep_AxeLarge(),
                WepType::Hammer => gem_param.data.canMountWep_HammerNormal(),
                WepType::GreatHammer => gem_param.data.canMountWep_HammerLarge(),
                WepType::Flail => gem_param.data.canMountWep_Flail(),
                WepType::Spear => gem_param.data.canMountWep_SpearNormal(),
                WepType::HeavySpear => gem_param.data.canMountWep_SpearHeavy(),
                WepType::Halberd => gem_param.data.canMountWep_SpearAxe(),
                WepType::Scythe => gem_param.data.canMountWep_Sickle(),
                WepType::Fist => gem_param.data.canMountWep_Knuckle(),
                WepType::Claw => gem_param.data.canMountWep_Claw(),
                WepType::Whip => gem_param.data.canMountWep_Whip(),
                WepType::ColossalWeapon => gem_param.data.canMountWep_AxhammerLarge(),
                WepType::LightBow => gem_param.data.canMountWep_BowSmall(),
                WepType::Bow => gem_param.data.canMountWep_BowNormal(),
                WepType::Greatbow => gem_param.data.canMountWep_BowLarge(),
                WepType::Crossbow => gem_param.data.canMountWep_ClossBow(),
                WepType::Ballista => gem_param.data.canMountWep_Ballista(),
                WepType::Staff => gem_param.data.canMountWep_Staff(),
                WepType::Seal => gem_param.data.canMountWep_Talisman(),
                WepType::SmallShield => gem_param.data.canMountWep_ShieldSmall(),
                WepType::MediumShield => gem_param.data.canMountWep_ShieldNormal(),
                WepType::Greatshield => gem_param.data.canMountWep_ShieldLarge(),
                WepType::Torch => gem_param.data.canMountWep_Torch(),
                WepType::None |
                WepType::Arrow |
                WepType::Greatarrow |
                WepType::Bolt |
                WepType::BallistaBolt |
                WepType::Unknown => {
                    false
                },
            }
        }

        // Check if inventory_common_items only has EquipInventoryItem with unique ids
        fn check_for_duplicate_items(item_list: &Vec<EquipInventoryItem>) -> Result<(), String> {
            let mut item_ids = HashSet::new();

            for item in item_list.iter().filter(|i| i.ga_item_handle.to_le_bytes()[3] == 0xB0) {
                if item_ids.contains(&item.ga_item_handle) {
                    return Err(format!("duplicate ga_item_handle {:#x}", item.ga_item_handle));
                }
                item_ids.insert(item.ga_item_handle);
            }
            Ok(())
        }
        // endregion
    }
}
