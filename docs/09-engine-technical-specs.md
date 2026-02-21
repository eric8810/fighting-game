# 格斗游戏引擎技术规格书（Technical Specifications）

**项目名称：** Tickle Fighting Engine
**版本：** v0.1.0-alpha
**技术栈：** Rust + winit + wgpu + kira + GGRS
**目标平台：** Windows / Linux / macOS / Web (WASM)

---

## 1. 架构概览

### 1.1 整体架构

```
┌─────────────────────────────────────────────────────────┐
│                   Game Application                       │
├─────────────────────────────────────────────────────────┤
│  Game Logic Layer (ECS)                                 │
│  ├── Fighter System (状态机、输入处理)                    │
│  ├── Combat System (碰撞检测、伤害结算)                   │
│  ├── Animation System (帧动画播放)                       │
│  └── Camera System (摄像机跟随)                          │
├─────────────────────────────────────────────────────────┤
│  Rollback Network Layer (GGRS)                          │
│  ├── Save State / Load State                            │
│  ├── Input Synchronization                              │
│  └── Deterministic Simulation                           │
├─────────────────────────────────────────────────────────┤
│  Rendering Layer (wgpu)                                 │
│  ├── Sprite Batch Renderer                              │
│  ├── Texture Atlas Manager                              │
│  └── Debug Renderer (碰撞盒可视化)                       │
├─────────────────────────────────────────────────────────┤
│  Audio Layer (kira)                                     │
│  ├── Sound Effect Manager                               │
│  └── Music Player                                       │
├─────────────────────────────────────────────────────────┤
│  Platform Layer (winit)                                 │
│  ├── Window Management                                  │
│  ├── Input Handling                                     │
│  └── Event Loop                                         │
└─────────────────────────────────────────────────────────┘
```

### 1.2 核心设计原则

| 原则 | 说明 |
|------|------|
| **确定性优先** | 所有游戏逻辑使用整数坐标（i32），禁止浮点运算 |
| **数据驱动** | 帧数据、角色属性、招式配置全部外置为 RON 文件 |
| **ECS 架构** | 使用轻量 ECS（hecs 或 bevy_ecs），便于回滚网络 |
| **模块化** | 每个子系统独立 crate，便于测试和复用 |
| **零拷贝** | 纹理、音频资源使用引用计数（Arc），避免重复加载 |

---

## 2. 坐标系统与单位

### 2.1 逻辑坐标系（确定性）

```rust
/// 逻辑坐标，单位：1/100 像素（定点数）
/// 例如：100 表示 1 像素，10000 表示 100 像素
pub type LogicCoord = i32;

/// 二维逻辑坐标
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LogicVec2 {
    pub x: LogicCoord,
    pub y: LogicCoord,
}

/// 逻辑坐标转渲染坐标
impl LogicVec2 {
    pub fn to_render(&self) -> [f32; 2] {
        [self.x as f32 / 100.0, self.y as f32 / 100.0]
    }
}
```

### 2.2 世界坐标系

```
Y 轴向上为正（OpenGL 风格）
X 轴向右为正
原点 (0, 0) 在舞台中心地面

        Y↑
         |
    P1   |   P2
  ───────┼───────→ X
         |
      地面 (Y=0)
```

### 2.3 单位换算

| 逻辑单位 | 实际含义 |
|---------|---------|
| 100 | 1 像素 |
| 10000 | 100 像素（角色宽度约此值） |
| 速度 500/帧 | 每帧移动 5 像素 |
| 重力 -80/帧² | 每帧速度减少 0.8 像素/帧 |

---

## 3. ECS 组件设计

### 3.1 核心组件

```rust
/// 位置组件
#[derive(Component, Clone, Copy)]
pub struct Position {
    pub pos: LogicVec2,
}

/// 速度组件
#[derive(Component, Clone, Copy)]
pub struct Velocity {
    pub vel: LogicVec2,
}

/// 朝向组件（1 = 面向右，-1 = 面向左）
#[derive(Component, Clone, Copy)]
pub struct Facing {
    pub dir: i32, // 1 or -1
}

/// 角色状态组件
#[derive(Component, Clone)]
pub struct FighterState {
    pub current_state: StateType,
    pub state_frame: u32, // 当前状态已执行帧数
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StateType {
    Idle,
    WalkForward,
    WalkBackward,
    Run,
    Crouch,
    Jump,
    Attack(AttackId),
    Hitstun,
    Blockstun,
    Knockdown,
}

/// 生命值组件
#[derive(Component, Clone, Copy)]
pub struct Health {
    pub current: i32,
    pub max: i32,
}

/// 气槽组件
#[derive(Component, Clone, Copy)]
pub struct PowerGauge {
    pub current: i32, // 0-3000
    pub max: i32,     // 3000
}

/// 输入缓冲组件
#[derive(Component, Clone)]
pub struct InputBuffer {
    pub history: [InputState; 16], // 最近 16 帧输入
    pub head: usize,
}

#[derive(Clone, Copy, Debug)]
pub struct InputState {
    pub buttons: u8, // bit0: A, bit1: B, bit2: C, bit3: D
    pub direction: Direction,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    Neutral,
    Up, Down, Left, Right,
    UpLeft, UpRight, DownLeft, DownRight,
}
```

### 3.2 碰撞盒组件

```rust
/// 碰撞盒管理器组件
#[derive(Component, Clone)]
pub struct HitboxManager {
    pub hitboxes: Vec<Hitbox>,   // 攻击框
    pub hurtboxes: Vec<Hurtbox>, // 受伤框
    pub pushbox: Pushbox,        // 推挤框
}

#[derive(Clone, Copy, Debug)]
pub struct Hitbox {
    pub rect: LogicRect,
    pub damage: i32,
    pub hitstun: u32,
    pub blockstun: u32,
    pub knockback: LogicVec2,
    pub hit_type: HitType,
}

#[derive(Clone, Copy, Debug)]
pub enum HitType {
    High,   // 站立防御
    Mid,    // 站立/蹲下均可防御
    Low,    // 必须蹲下防御
    Throw,  // 无法防御
}

#[derive(Clone, Copy, Debug)]
pub struct Hurtbox {
    pub rect: LogicRect,
}

#[derive(Clone, Copy, Debug)]
pub struct Pushbox {
    pub rect: LogicRect,
}

#[derive(Clone, Copy, Debug)]
pub struct LogicRect {
    pub x: LogicCoord,
    pub y: LogicCoord,
    pub w: LogicCoord,
    pub h: LogicCoord,
}
```

### 3.3 动画组件

```rust
/// 精灵动画组件
#[derive(Component, Clone)]
pub struct SpriteAnimation {
    pub texture_id: AssetId,
    pub current_frame: u32,
    pub frame_duration: u32, // 每帧持续时间（游戏帧）
    pub frame_timer: u32,
    pub frames: Vec<SpriteFrame>,
    pub looping: bool,
}

#[derive(Clone, Debug)]
pub struct SpriteFrame {
    pub atlas_index: u32,
    pub offset: LogicVec2, // 相对于角色位置的偏移
}
```

---

## 4. 帧数据系统

### 4.1 数据格式（RON）

```ron
// data/moves/ryu_hadoken.ron
Move(
    id: "ryu_hadoken",
    name: "波动拳",
    input: QCF(A),
    startup: 6,
    active: 4,
    recovery: 18,
    total_frames: 28,
    damage: 1200,
    stun_damage: 80,
    gauge_gain_on_hit: 20,
    gauge_gain_on_block: 10,
    on_hit_advantage: 2,
    on_block_advantage: -4,
    cancel_windows: [
        CancelWindow(
            start: 7,
            end: 12,
            into: [Super],
        ),
    ],
    frames: [
        MoveFrame(
            frame_index: 6,
            hitboxes: [
                Hitbox(
                    rect: (x: 2000, y: -5000, w: 6000, h: 3000),
                    damage: 1200,
                    hitstun: 18,
                    blockstun: 12,
                    knockback: (x: 30000, y: 0),
                    hit_type: Mid,
                ),
            ],
            hurtboxes: [
                Hurtbox(rect: (x: -2000, y: -8000, w: 4000, h: 8000)),
            ],
            pushbox: Pushbox(rect: (x: -1500, y: -8000, w: 3000, h: 8000)),
        ),
        // ... 其他帧
    ],
    animation: "ryu_hadoken_anim",
    hit_effect: "spark_medium",
    hit_sound: "hit_medium_01",
    block_sound: "block_01",
)
```

### 4.2 数据加载

```rust
pub struct MoveDatabase {
    moves: HashMap<String, MoveData>,
}

impl MoveDatabase {
    pub fn load_from_dir(path: &Path) -> Result<Self>;
    pub fn get_move(&self, id: &str) -> Option<&MoveData>;
}
```

---

## 5. 状态机系统

### 5.1 状态转换表

```rust
pub struct StateMachine {
    current: StateType,
    frame: u32,
}

impl StateMachine {
    /// 尝试转换到新状态
    pub fn try_transition(
        &mut self,
        input: &InputState,
        fighter: &FighterComponents,
    ) -> Option<StateType> {
        match self.current {
            StateType::Idle => {
                if input.direction == Direction::Down {
                    return Some(StateType::Crouch);
                }
                if input.direction == Direction::Right {
                    return Some(StateType::WalkForward);
                }
                // ... 其他转换
            }
            // ... 其他状态
        }
        None
    }
}
```

### 5.2 状态优先级

| 优先级 | 状态类型 | 说明 |
|--------|---------|------|
| 1 | Hitstun / Blockstun | 受击/格挡硬直，无法取消 |
| 2 | Attack | 攻击中，只能在取消窗口转换 |
| 3 | Jump | 跳跃中，只能空中攻击 |
| 4 | Run | 跑步中，可取消为攻击 |
| 5 | Crouch / Walk | 蹲下/行走，可随时取消 |
| 6 | Idle | 待机，可转换为任何状态 |

---

## 6. 碰撞检测系统

### 6.1 检测流程

```rust
pub fn collision_system(world: &mut World) {
    // 1. 收集所有活跃的 Hitbox 和 Hurtbox
    let mut hits: Vec<HitEvent> = Vec::new();

    for (p1_entity, p1_hitboxes, p1_pos, p1_facing) in query_hitboxes() {
        for (p2_entity, p2_hurtboxes, p2_pos) in query_hurtboxes() {
            if p1_entity == p2_entity { continue; }

            for hitbox in p1_hitboxes {
                let world_hitbox = transform_to_world(hitbox, p1_pos, p1_facing);

                for hurtbox in p2_hurtboxes {
                    let world_hurtbox = transform_to_world(hurtbox, p2_pos, Facing(1));

                    if aabb_intersect(world_hitbox, world_hurtbox) {
                        hits.push(HitEvent {
                            attacker: p1_entity,
                            defender: p2_entity,
                            hitbox: *hitbox,
                        });
                    }
                }
            }
        }
    }

    // 2. 处理命中事件
    for hit in hits {
        apply_hit(world, hit);
    }

    // 3. Pushbox 分离
    resolve_pushbox_overlap(world);
}

fn aabb_intersect(a: LogicRect, b: LogicRect) -> bool {
    a.x < b.x + b.w &&
    a.x + a.w > b.x &&
    a.y < b.y + b.h &&
    a.y + a.h > b.y
}
```

---

## 7. 回滚网络系统

### 7.1 GGRS 集成

```rust
use ggrs::{GGRSRequest, GameState, PlayerType, SessionBuilder};

/// 游戏状态快照（必须实现 Clone）
#[derive(Clone)]
pub struct GameSnapshot {
    pub entities: Vec<EntitySnapshot>,
    pub frame: u32,
    pub rng_state: u64, // 伪随机数生成器状态
}

impl GameState for GameSnapshot {
    fn save(&self) -> Vec<u8> {
        bincode::serialize(self).unwrap()
    }

    fn load(data: &[u8]) -> Self {
        bincode::deserialize(data).unwrap()
    }
}

/// 实体快照
#[derive(Clone, Serialize, Deserialize)]
pub struct EntitySnapshot {
    pub position: Position,
    pub velocity: Velocity,
    pub facing: Facing,
    pub state: FighterState,
    pub health: Health,
    pub power_gauge: PowerGauge,
    pub input_buffer: InputBuffer,
}
```

### 7.2 确定性保证

| 要求 | 实现方式 |
|------|---------|
| 禁止浮点 | 所有逻辑坐标使用 `i32` |
| 禁止随机数 | 使用固定种子的 LCG，种子同步 |
| 禁止时间依赖 | 只使用帧计数，不使用 `Instant::now()` |
| 禁止异步 | 所有逻辑同步执行 |
| 输入同步 | GGRS 自动处理 |

### 7.3 伪随机数生成器

```rust
/// 线性同余生成器（LCG），保证确定性
pub struct DeterministicRng {
    state: u64,
}

impl DeterministicRng {
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    pub fn next(&mut self) -> u32 {
        self.state = self.state.wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (self.state >> 32) as u32
    }

    pub fn range(&mut self, min: i32, max: i32) -> i32 {
        let range = (max - min) as u32;
        min + (self.next() % range) as i32
    }
}
```

---

## 8. 渲染系统

### 8.1 Sprite Batch 渲染器

```rust
pub struct SpriteBatchRenderer {
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    texture_bind_group: wgpu::BindGroup,
    batch: Vec<SpriteInstance>,
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SpriteInstance {
    pub position: [f32; 2],
    pub size: [f32; 2],
    pub uv_offset: [f32; 2],
    pub uv_size: [f32; 2],
    pub color: [f32; 4],
}

impl SpriteBatchRenderer {
    pub fn add_sprite(&mut self, sprite: SpriteInstance);
    pub fn flush(&mut self, encoder: &mut wgpu::CommandEncoder);
}
```

### 8.2 纹理图集

```rust
pub struct TextureAtlas {
    texture: wgpu::Texture,
    regions: HashMap<String, AtlasRegion>,
}

#[derive(Clone, Copy)]
pub struct AtlasRegion {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}
```

---

## 9. 音频系统

### 9.1 音效管理

```rust
use kira::manager::{AudioManager, AudioManagerSettings};
use kira::sound::static_sound::{StaticSoundData, StaticSoundHandle};

pub struct AudioSystem {
    manager: AudioManager,
    sounds: HashMap<String, StaticSoundData>,
}

impl AudioSystem {
    pub fn play_sound(&mut self, id: &str) -> Result<StaticSoundHandle>;
    pub fn play_music(&mut self, id: &str, looping: bool);
    pub fn stop_music(&mut self);
}
```

---

## 10. 资源管理

### 10.1 资源加载器

```rust
pub struct AssetLoader {
    textures: HashMap<AssetId, Arc<wgpu::Texture>>,
    sounds: HashMap<AssetId, Arc<StaticSoundData>>,
    moves: HashMap<String, Arc<MoveData>>,
}

impl AssetLoader {
    pub async fn load_texture(&mut self, path: &Path) -> AssetId;
    pub async fn load_sound(&mut self, path: &Path) -> AssetId;
    pub fn load_move_data(&mut self, path: &Path) -> Result<()>;
}
```

---

## 11. 性能指标

### 11.1 目标性能

| 指标 | 目标值 |
|------|--------|
| 帧率 | 固定 60 FPS |
| 逻辑帧时间 | < 8ms |
| 渲染帧时间 | < 8ms |
| 内存占用 | < 500MB |
| 启动时间 | < 3s |
| 回滚深度 | 最多回滚 8 帧 |

### 11.2 性能分析工具

- `cargo flamegraph` - CPU 火焰图
- `tracing-tracy` - 帧分析器
- `cargo bench` - 基准测试

---

## 12. 测试策略

### 12.1 单元测试

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_aabb_collision() {
        let a = LogicRect { x: 0, y: 0, w: 100, h: 100 };
        let b = LogicRect { x: 50, y: 50, w: 100, h: 100 };
        assert!(aabb_intersect(a, b));
    }

    #[test]
    fn test_deterministic_rng() {
        let mut rng1 = DeterministicRng::new(12345);
        let mut rng2 = DeterministicRng::new(12345);
        assert_eq!(rng1.next(), rng2.next());
    }
}
```

### 12.2 集成测试

- 回滚网络确定性测试（双端状态对比）
- 输入缓冲指令识别测试
- 碰撞检测边界测试

---

## 13. 构建与发布

### 13.1 构建配置

```toml
# Cargo.toml
[profile.release]
opt-level = 3
lto = "fat"
codegen-units = 1
strip = true

[profile.dev]
opt-level = 1 # 开发时轻度优化，保持编译速度
```

### 13.2 平台目标

```bash
# Windows
cargo build --release --target x86_64-pc-windows-msvc

# Linux
cargo build --release --target x86_64-unknown-linux-gnu

# macOS
cargo build --release --target x86_64-apple-darwin

# Web (WASM)
cargo build --release --target wasm32-unknown-unknown
```

---

## 14. 依赖清单

```toml
[dependencies]
# 窗口 + 事件循环
winit = "0.30"

# 渲染
wgpu = "23.0"
bytemuck = { version = "1.14", features = ["derive"] }

# ECS
hecs = "0.10"  # 或 bevy_ecs = "0.15"

# 音频
kira = "0.9"

# 回滚网络
ggrs = "0.10"

# 序列化
serde = { version = "1.0", features = ["derive"] }
ron = "0.8"
bincode = "1.3"

# 数学
glam = "0.29"  # 仅用于渲染变换，逻辑层不使用

# 资源加载
image = "0.25"

# 调试
tracing = "0.1"
tracing-subscriber = "0.3"

# 性能分析（可选）
tracing-tracy = { version = "0.11", optional = true }
```

---

## 15. 项目结构

```
tickle/
├── Cargo.toml
├── assets/
│   ├── textures/
│   ├── sounds/
│   ├── data/
│   │   ├── moves/
│   │   └── characters/
│   └── shaders/
├── src/
│   ├── main.rs
│   ├── lib.rs
│   ├── core/
│   │   ├── mod.rs
│   │   ├── types.rs          # LogicVec2, LogicRect 等
│   │   ├── rng.rs             # DeterministicRng
│   │   └── input.rs           # InputState, Direction
│   ├── ecs/
│   │   ├── mod.rs
│   │   ├── components.rs      # 所有 ECS 组件
│   │   └── systems.rs         # 所有 ECS 系统
│   ├── combat/
│   │   ├── mod.rs
│   │   ├── collision.rs       # 碰撞检测
│   │   ├── state_machine.rs   # 状态机
│   │   └── move_data.rs       # 帧数据加载
│   ├── rendering/
│   │   ├── mod.rs
│   │   ├── sprite_batch.rs    # Sprite Batch 渲染器
│   │   ├── texture_atlas.rs   # 纹理图集
│   │   └── shaders.wgsl       # WGSL 着色器
│   ├── audio/
│   │   ├── mod.rs
│   │   └── audio_system.rs
│   ├── network/
│   │   ├── mod.rs
│   │   ├── ggrs_wrapper.rs    # GGRS 封装
│   │   └── snapshot.rs        # 游戏状态快照
│   └── assets/
│       ├── mod.rs
│       └── loader.rs          # 资源加载器
├── benches/                   # 基准测试
└── tests/                     # 集成测试
```

---

**文档版本：** v1.0
**最后更新：** 2026-02-21
**维护者：** Tickle Engine Team
