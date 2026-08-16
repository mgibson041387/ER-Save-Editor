# ER Save Editor

<p align="center">
  <a href="https://github.com/ClayAmore/ER-Save-Editor/blob/master/icon/readme.md"><img src="https://github.com/ClayAmore/ER-Save-Editor/assets/131625063/e1d40274-066a-4b72-b010-de9170090b60" /></a>
</p>

Tool for editing Elden Ring save files. Compatible with PC saves, raw Playstation saves, and Playstation Save Wizard exported saves.

**Use with caution:** This tool should be used responsibly. Always backup your save file before making any edits. This is a hobby project developed during spare time and is intended for easier character build making. I do not endorse or condone cheating online.

## Web version
Runs entirely in your browser via WebAssembly -- nothing is uploaded anywhere, your save file stays local.

- https://mgibson041387.github.io/ER-Save-Editor/
- https://selfhosted.vip/er-save-editor

## Downloads
Compiled Windows and Linux builds are available on the [Releases page](https://github.com/mgibson041387/ER-Save-Editor/releases).

## Features
- Import characters from other save files
- Change pc save file SteamID
- Modify player name
- Change player gender
- Edit player stats
- Modify soul count
- Add items, weapons, armors, ashes of war, and talismans to inventory
- Add items in bulk to speed up build making process.
- Browse inventory
- Edit item quantity and weapon upgrade level, or remove items, directly from the Browse screen
- Change player equipment
- Activate/deactivate Sites of Grace, summoning pools, colosseums, etc.
- Revive or kill bosses
- Activate/deactivate invasion regions
- Direct editing of raw PlayStation (PS4) saves, no Save Wizard conversion required
- Quick Cheats screen:
  - Souls: quick-add presets (+1k up to +10M) or jump straight to the cap
  - Stats: max all 8 attributes to 99 in one click
  - Rune Arcs: quick-add presets
  - Flasks: top off held Crimson/Cerulean Tears to their current upgrade level's max charges
  - Unlocks: bulk-grant an entire category at once -- Weapons, Armor, Talismans, Ashes of War, Spells, Crafting Materials, Consumables, Bell Bearings, Crystal Tears, Spirit Ashes, and Sites of Grace
- Stats page: press-and-hold `<`/`>` buttons on each attribute for in-game-style +/-1 stepping with auto-repeat
- More features will be added in future updates

## Bug fixes
- Fixed several crash/panic paths (out-of-bounds stat table lookups, unmapped weapon/gem IDs, malformed save-format parsing) that previously brought down the whole app on edge-case input
- Fixed raw PlayStation saves being rejected outright as "not a recognized Elden Ring save" -- the magic bytes were never checked for
- Removed an incorrect assumption that a certain save field's first byte was always `0`, which caused legitimate real-world saves to fail to load

## Permissions
Feel free to use this save editor for learning or development purposes. However, I do not authorize its use for creating tools or modifications that enable actions online outside the bounds of what the game allows.

## FAQ
Q: Will this ban me?<br/>
A: There's no guarantee that you won't be banned. None of these features have been tested online.

## Reporting Issues
If you encounter any bugs or issues while using the save editor, please report them. When reporting bugs, try to provide reproducible steps so I can debug effectively.

## Credits
<a href="https://github.com/nordgaren/"><img src="https://github.com/ClayAmore/ER-Save-Editor/assets/131625063/710c9ee6-c3df-4665-be6b-d96bce1ebf46"/></a>
