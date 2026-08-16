pub mod stats_view_model {
    use crate::{db::classes::classes::ArcheType, save::common::save_slot::SaveSlot};

    #[derive(Clone)]
    pub struct StatsViewModel  {
        pub arche_type: ArcheType,
        pub vigor: u32,
        pub mind: u32,
        pub endurance: u32,
        pub strength: u32,
        pub dexterity: u32,
        pub intelligence: u32,
        pub faith: u32,
        pub arcane: u32,
        pub level: u32,
        pub souls: u32,
        pub soulsmemory: u32,
        pub spirit_tuning_level: u8,
        pub great_rune_active: bool
    }

    impl Default for StatsViewModel {
        fn default() -> Self {
            Self { 
                arche_type: ArcheType::Unknown,
                vigor: Default::default(), 
                mind: Default::default(), 
                endurance: Default::default(), 
                strength: Default::default(), 
                dexterity: Default::default(), 
                intelligence: Default::default(), 
                faith: Default::default(), 
                arcane: Default::default(),
                level: Default::default(),
                souls: Default::default(),
                soulsmemory: Default::default(),
                spirit_tuning_level: Default::default(),
                great_rune_active: Default::default(),
            }
        }
    }

    impl StatsViewModel {
        pub fn from_save(slot:& SaveSlot) -> Self {
            let arche_type = ArcheType::try_from(slot.player_game_data.arche_type).expect("");
            let vigor = slot.player_game_data.vigor;
            let mind = slot.player_game_data.mind;
            let endurance = slot.player_game_data.endurance;
            let strength = slot.player_game_data.strength;
            let dexterity = slot.player_game_data.dexterity;
            let intelligence = slot.player_game_data.intelligence;
            let faith = slot.player_game_data.faith;
            let arcane = slot.player_game_data.arcane;
            let level = slot.player_game_data.level;
            let souls = slot.player_game_data.souls;
            let soulsmemory = slot.player_game_data.soulsmemory;
            let spirit_tuning_level = slot.player_game_data.spirit_tuning_level;
            let great_rune_active = slot.player_game_data.great_rune_active != 0;

            Self {
                arche_type,
                vigor,
                mind,
                endurance,
                strength,
                dexterity,
                intelligence,
                faith,
                arcane,
                level,
                souls,
                soulsmemory,
                spirit_tuning_level,
                great_rune_active
            }
        }
    }
}