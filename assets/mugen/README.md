# MUGEN Assets

This directory is intentionally empty in the repository.

## Required Files

Place the following files here before running the game:

### `common1.cns`

Contains the standard MUGEN common states (stand, walk, jump, guard, hitstun, knockdown, etc.).

**Source**: Copy from any MUGEN installation:
```
<MUGEN install>/data/common1.cns
```

If this file is missing, the game will log a warning and characters will lack common states
(standing, walking, jumping will not work correctly).

### `chars/`

Place MUGEN character directories here:
```
assets/mugen/chars/KyoKusanagi/
    KyoKusanagi.def
    kyo.cns
    kyo.cmd
    kyo.sff
    kyo.air
    kyo.snd
    kyo1.act ... kyo6.act
```

The game currently loads from `assets/mugen_resources/chars/` — update the path in
`game/src/main.rs` if you use a different location.

## Legal Note

MUGEN files are © Elecbyte and are NOT included in this repository.
Users must supply their own copies.
