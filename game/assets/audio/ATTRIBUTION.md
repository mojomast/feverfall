# FeverFall Audio Attribution

All runtime audio files in this directory are short fallback synthesis cues generated for FeverFall Checkpoint 5.

## License / provenance

- Files: `*.wav`
- Author: FeverFall contributors
- Source: procedurally generated square/sine/chord fallback tones committed with the project
- License dedication: CC0 1.0 Universal / public domain dedication
- Notes: No third-party samples, Kenney packs, OpenGameArt downloads, or Freesound downloads are included in this checkpoint. The generated cues are intentionally short placeholders so later production audio can replace them without licensing ambiguity.

## Runtime manifest

The code manifest in `game/src/plugins/audio/mod.rs` maps each audible `AudioLayer` to one of these files. `SilenceGate` is intentional silence and has no asset.
