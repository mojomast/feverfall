# Roguelite Meta Save Migration

FeverFall's roguelite meta progression save lives at `saves/roguelite/meta.json` and is owned by `run_mode::MetaProgressionSave`.

## Current format

The current wire format is version `1`, exposed in code as `META_PROGRESSION_SAVE_VERSION` and `MetaProgressionSave::CURRENT_VERSION`. Runtime code keeps using the clean `MetaProgressionSave` type; serde writes through a private v1 wire struct so saved JSON always includes:

```json
{
  "version": 1,
  "total_runs": 0,
  "total_oranges_cleared": 0,
  "relics_seen": [],
  "unlocked_starter_balls": ["balls/act1/basic_orb"],
  "unlocked_starting_relics": [],
  "unlocked_board_archetype_weights": [],
  "mastery_records": []
}
```

## Legacy v0 migration

Older saves did not include a `version` field. Missing `version` is treated as legacy v0 and migrated during deserialize. Known legacy-missing fields are defaulted only where safe:

- `unlocked_starter_balls` defaults to `["balls/act1/basic_orb"]`.
- `unlocked_starting_relics`, `unlocked_board_archetype_weights`, and `mastery_records` default to empty arrays.
- `relics_seen` also defaults to an empty array for early meta-save snapshots.

## Unknown versions

Saves with a `version` greater than `MetaProgressionSave::CURRENT_VERSION` are rejected instead of being silently downgraded. Future migrations should add a new private wire struct, convert it into `MetaProgressionSave`, and keep v0/v1 tests intact.
