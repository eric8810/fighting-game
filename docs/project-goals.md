# Tickle - Project Goals and Technical Direction

## Core Vision

**Build a modern, high-quality KOF2000-style 2D fighting game engine with academic rigor and open-source resources.**

---

## Primary Goals

### 1. Modern Fighting Game Experience

**High-Resolution Visuals**
- Support 4K sprite assets (720-1080px character height vs traditional 200-300px)
- Target 1080p-4K display resolutions
- Maintain pixel-perfect precision at high resolutions

**Best-in-Class Netcode**
- Rollback netcode (GGPO-style) with deterministic simulation
- Fixed 60 FPS logic layer (industry standard)
- Variable render rate (60/120/144/240 Hz) with interpolation
- Sub-frame input precision for local play

**Precise Combat System**
- Frame-perfect collision detection (60 FPS logic)
- Multiple hitbox/hurtbox per character (head, body, limbs)
- Pixel-accurate positioning (1/100 pixel precision in logic layer)
- Deterministic integer-based physics for rollback compatibility

**Modern Tooling**
- Comprehensive training mode (frame data display, hitbox visualization, replay)
- Match replay system with slow-motion and frame-by-frame analysis
- Online matchmaking with rank system and connection quality display
- Spectator mode with camera controls

### 2. KOF2000 Core Mechanics

The game design is based on KOF2000's proven mechanics:
- 4-character team system with striker mechanics
- MAX mode activation system
- Emergency evasion (dodge roll)
- Guard cancel system
- Counter wire and wall bounce mechanics
- Power gauge and super move system

These are **game design features**, not engine limitations. The engine should support these mechanics but remain flexible for variations.

### 3. Extensibility

**Content Pipeline**
- Support MUGEN SFF/AIR formats for sprite/animation loading (already implemented)
- Support MUGEN CNS/CMD formats as **reference** for character data (subset implementation)
- Custom RON-based character data format for original content
- AI-generated high-resolution sprite workflow

**Modular Architecture**
- Clean separation: core logic (tickle_core), rendering (tickle_render), audio (tickle_audio), network (tickle_network)
- ECS-based entity system (hecs) for flexibility
- Deterministic simulation for rollback compatibility

---

## Technical Specifications

### Resolution and Assets

| Aspect | Traditional (KOF2000) | Tickle Target |
|--------|----------------------|---------------|
| Display resolution | 320x224 (CRT) | 1920x1080 - 3840x2160 |
| Character sprite height | 200-300px | 720-1080px (4K-ready) |
| Sprite format | Indexed color (256 colors) | RGBA (full color) |
| Animation frames | 8-15 fps (hand-drawn) | 60 fps logic, variable render |

### Performance Targets

| System | Target | Notes |
|--------|--------|-------|
| Logic frame time | <16.67ms (60 FPS) | Fixed timestep, deterministic |
| Render frame time | <8.33ms (120 FPS) | Variable rate with interpolation |
| Rollback depth | 8 frames | ~133ms at 60 FPS |
| Network latency tolerance | <150ms | With rollback |
| Memory footprint | <2GB | For 2 characters + stage |

### Physics and Collision

- **Coordinate system**: Integer-based (1/100 pixel precision)
- **Collision detection**: AABB (axis-aligned bounding boxes)
- **Hitbox granularity**: Multiple boxes per character (head, body, limbs)
- **Physics tick rate**: 60 FPS fixed (industry standard)
- **Gravity**: -80 units/frame² (tunable per character)
- **Friction**: 50 units/frame (tunable)

---

## Content Strategy

### Phase 1: Development Assets (Current)

Use existing MUGEN community resources for rapid prototyping:
- Low-resolution sprites (320x224 era)
- MUGEN SFF/AIR format (already supported)
- Subset of MUGEN CNS/CMD for character logic

**Limitation**: MUGEN assets are 576p era, not 4K-ready.

### Phase 2: AI-Generated High-Resolution Assets

Leverage AI tools to generate high-resolution sprite sequences:
- Stable Diffusion / Midjourney for character design
- AI animation tools for frame generation
- Upscaling existing MUGEN sprites as interim solution

**Advantage**: Faster than 3D workflow, maintains 2D aesthetic.

### Phase 3: Original Content (Long-term)

Commission or create original high-resolution sprites:
- Hand-drawn or 3D-rendered sprite sequences
- Full 4K asset pipeline
- Original character designs

---

## What We Are NOT Doing

❌ **3D Skeletal Animation in Real-Time**
- Guilty Gear uses 3D models but renders to sprite sequences
- Real-time skeletal animation breaks frame-perfect control
- We may use 3D tools in the asset pipeline, but runtime is sprite-based

❌ **Variable Logic Tick Rate**
- 60 FPS logic is the fighting game industry standard
- Rollback netcode requires fixed tick rate
- Frame data (3f startup, 5f recovery) is core to game balance

❌ **Equipment/Loadout System**
- Contradicts KOF's pure skill-based design
- Adds balance complexity incompatible with competitive play
- May revisit for single-player modes only

❌ **Full MUGEN Compatibility**
- MUGEN has 20+ years of edge cases and quirks
- Full compatibility would constrain engine design
- We support MUGEN formats as a **content pipeline tool**, not a compatibility target

---

## Success Criteria

**Technical Milestones**
- [ ] Rollback netcode with <5 frame rollback on 100ms connections
- [ ] 4K sprite rendering at 120+ FPS
- [ ] Deterministic simulation (bit-identical across platforms)
- [ ] Training mode with frame data display and hitbox visualization

**Content Milestones**
- [ ] 2 playable characters with full movesets
- [ ] 3 stages with parallax backgrounds
- [ ] Complete UI (menus, HUD, match flow)
- [ ] Online matchmaking with rank system

**Community Milestones**
- [ ] Open-source release with documentation
- [ ] Modding tools and character creation guide
- [ ] Active community creating custom characters
- [ ] Tournament-ready netcode quality

---

## Comparison to Existing Projects

| Project | Approach | Our Difference |
|---------|----------|----------------|
| MUGEN | Full 2D engine, no netcode | We have rollback, modern resolution |
| IKEMEN GO | MUGEN + rollback | We're not bound by MUGEN compatibility |
| Skullgirls | Custom engine, hand-drawn | We use AI/community assets |
| Guilty Gear Strive | 3D→2D rendering | We're pure 2D sprites |
| Fightcade | Emulator + rollback | We're a native modern engine |

---

## Development Philosophy

**Academic Rigor**
- Open-source development with full documentation
- Reference implementations of fighting game systems
- Educational value for game developers

**Pragmatic Scope**
- Focus on core fighting game experience first
- Use existing assets/formats to accelerate development
- Avoid over-engineering for hypothetical future features

**Community-Driven**
- Leverage open-source MUGEN community resources
- Build modding tools for community content creation
- Transparent development process

---

Last updated: 2026-02-28
