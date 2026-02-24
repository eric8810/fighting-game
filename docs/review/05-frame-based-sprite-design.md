# 帧级精灵与碰撞盒系统设计

> 设计日期：2026-02-24
> 对应任务：A-2（帧级碰撞盒数据结构）、A-3（每帧独立尺寸渲染方案）

---

## 1. 问题陈述

**当前架构的根本缺陷**：

```rust
// 现在（错误）：假设所有帧等宽等高
const FIGHTER_W: f32 = 100.0;
const FIGHTER_H: f32 = 109.0;

// 渲染时用固定矩形
rect: [screen_x, screen_y, FIGHTER_W, FIGHTER_H]

// 碰撞盒是静态 ECS 组件，不随动画帧变化
pub struct HitboxManager {
    pub hitboxes: Vec<Hitbox>,      // 全局静态
    pub hurtboxes: Vec<Hurtbox>,    // 全局静态
    pub pushbox: Pushbox,           // 全局静态
}
```

**真实格斗游戏的需求**：

```
Idle 第 1 帧：宽 60px 高 90px，轴点在脚底，hurtbox 覆盖身体
Idle 第 2 帧：宽 62px 高 91px，轴点在脚底，hurtbox 稍微扩大
Attack 第 3 帧：宽 110px 高 88px，轴点偏移 +30px，hitbox 在拳头位置，hurtbox 缩小
```

每帧的**尺寸、原点偏移、碰撞盒**都不同。

---

## 2. 新数据结构设计

### 2.1 核心类型定义

```rust
// crates/tickle_core/src/sprite.rs（新文件）

use crate::components::{Hitbox, Hurtbox, Pushbox};
use crate::math::LogicVec2;
use serde::{Deserialize, Serialize};

/// 单个精灵帧的完整定义
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpriteFrame {
    /// MUGEN group/image 编号（用于索引 SFF 文件）
    pub group: u16,
    pub image: u16,

    /// 精灵尺寸（像素）
    pub width: u16,
    pub height: u16,

    /// 轴点偏移（相对于精灵左上角）
    pub axis_offset: LogicVec2,

    /// 持续时长（逻辑帧数，60 tick = 1秒）
    pub duration: u32,

    /// 水平翻转标志
    pub flip_h: bool,

    /// 攻击框（Clsn1）- 相对于轴点的坐标
    pub hitboxes: Vec<Hitbox>,

    /// 受击框（Clsn2）- 相对于轴点的坐标
    pub hurtboxes: Vec<Hurtbox>,

    /// 推挤框 - 相对于轴点的坐标
    pub pushbox: Pushbox,
}

/// 动画定义（对应 MUGEN 的一个 Action）
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Animation {
    /// Action 编号（如 0=Idle, 20=Walk, 200=Attack）
    pub action_number: u32,

    /// 帧序列
    pub frames: Vec<SpriteFrame>,

    /// 是否循环
    pub looping: bool,

    /// 循环起点（帧索引）
    pub loop_start: usize,
}

/// 角色精灵数据（从 SFF 文件加载）
pub struct SpriteAtlas {
    /// 精灵图集（group, image） -> 纹理区域
    sprites: HashMap<(u16, u16), SpriteData>,
}

pub struct SpriteData {
    /// 在纹理图集中的 UV 坐标
    pub uv_rect: [f32; 4],  // [u, v, w, h]

    /// 精灵实际尺寸（像素）
    pub width: u16,
    pub height: u16,

    /// 轴点偏移（相对于左上角）
    pub axis_x: i16,
    pub axis_y: i16,
}
```

### 2.2 状态机集成

```rust
// crates/tickle_core/src/state_machine.rs（修改）

pub struct StateMachine {
    pub state: FighterState,

    /// 当前播放的动画
    current_animation: Option<Arc<Animation>>,

    /// 当前帧索引
    current_frame_index: usize,

    /// 当前帧内的 tick 计数
    frame_tick: u32,

    // ... 其他字段
}

impl StateMachine {
    /// 切换到新动画
    pub fn play_animation(&mut self, animation: Arc<Animation>) {
        self.current_animation = Some(animation);
        self.current_frame_index = 0;
        self.frame_tick = 0;
    }

    /// 更新动画，返回当前帧数据
    pub fn update(&mut self) -> Option<&SpriteFrame> {
        let anim = self.current_animation.as_ref()?;

        // 推进帧计数
        self.frame_tick += 1;

        let current_frame = &anim.frames[self.current_frame_index];

        // 检查是否需要切换到下一帧
        if self.frame_tick >= current_frame.duration {
            self.frame_tick = 0;
            self.current_frame_index += 1;

            // 处理循环
            if self.current_frame_index >= anim.frames.len() {
                if anim.looping {
                    self.current_frame_index = anim.loop_start;
                } else {
                    self.current_frame_index = anim.frames.len() - 1;
                }
            }
        }

        Some(&anim.frames[self.current_frame_index])
    }

    /// 获取当前帧（不推进）
    pub fn current_frame(&self) -> Option<&SpriteFrame> {
        let anim = self.current_animation.as_ref()?;
        anim.frames.get(self.current_frame_index)
    }
}
```

### 2.3 碰撞盒同步

```rust
// crates/tickle_core/src/systems/animation.rs（新增）

/// 根据当前动画帧更新实体的碰撞盒
pub fn sync_hitboxes_system(
    world: &mut hecs::World,
) {
    for (_, (sm, hitbox_mgr, pos, facing)) in world
        .query_mut::<(&StateMachine, &mut HitboxManager, &Position, &Facing)>()
    {
        if let Some(frame) = sm.current_frame() {
            // 清空旧碰撞盒
            hitbox_mgr.hitboxes.clear();
            hitbox_mgr.hurtboxes.clear();

            // 应用当前帧的碰撞盒（考虑面朝方向）
            for hitbox in &frame.hitboxes {
                let mut h = hitbox.clone();
                if facing.dir == Facing::LEFT {
                    h.rect = h.rect.flip_x(0);  // 水平翻转
                }
                hitbox_mgr.hitboxes.push(h);
            }

            for hurtbox in &frame.hurtboxes {
                let mut h = hurtbox.clone();
                if facing.dir == Facing::LEFT {
                    h.rect = h.rect.flip_x(0);
                }
                hitbox_mgr.hurtboxes.push(h);
            }

            // 更新 pushbox
            let mut pb = frame.pushbox;
            if facing.dir == Facing::LEFT {
                pb.rect = pb.rect.flip_x(0);
            }
            hitbox_mgr.pushbox = pb;
        }
    }
}
```

---

## 3. 渲染层设计

### 3.1 渲染流程

```rust
// game/src/main.rs（修改 render 函数）

fn render(&self, alpha: f32) {
    // ... 背景渲染 ...

    let mut fighter_instances = Vec::new();

    for (_, (pos, prev_pos, facing, sm, sprite_atlas)) in self.world
        .query::<(&Position, &PreviousPosition, &Facing, &StateMachine, &SpriteAtlas)>()
        .iter()
    {
        // 获取当前帧数据
        let frame = match sm.current_frame() {
            Some(f) => f,
            None => continue,  // 没有动画，跳过
        };

        // 从 SFF atlas 查询精灵数据
        let sprite_data = match sprite_atlas.get_sprite(frame.group, frame.image) {
            Some(s) => s,
            None => {
                log::warn!("Sprite not found: group={}, image={}", frame.group, frame.image);
                continue;
            }
        };

        // 插值位置（逻辑坐标 -> 渲染坐标）
        let prev_render = prev_pos.pos.to_render();
        let cur_render = pos.pos.to_render();
        let interp_x = prev_render[0] + (cur_render[0] - prev_render[0]) * alpha;
        let interp_y = prev_render[1] + (cur_render[1] - prev_render[1]) * alpha;

        // 应用轴点偏移（轴点是精灵的"锚点"，通常在脚底）
        let axis_offset_x = sprite_data.axis_x as f32;
        let axis_offset_y = sprite_data.axis_y as f32;

        let screen_x = interp_x - axis_offset_x;
        let screen_y = ground_screen_y - interp_y - axis_offset_y;

        // 使用精灵的实际尺寸
        let sprite_w = sprite_data.width as f32;
        let sprite_h = sprite_data.height as f32;

        // UV 坐标（考虑水平翻转）
        let uv = if facing.dir == Facing::RIGHT {
            sprite_data.uv_rect  // [u, v, w, h]
        } else {
            // 水平翻转：u 起点右移，宽度取负
            let [u, v, w, h] = sprite_data.uv_rect;
            [u + w, v, -w, h]
        };

        fighter_instances.push(QuadInstance {
            rect: [screen_x, screen_y, sprite_w, sprite_h],
            color: [1.0, 1.0, 1.0, 1.0],
            uv,
        });
    }

    // 渲染所有战士
    quad_renderer.draw(&fighter_instances, &fighter_texture);
}
```

### 3.2 关键改动点

**删除**：
- `const FIGHTER_W/H` 固定尺寸常量
- 硬编码的 UV 计算 `match` 语句

**新增**：
- `SpriteAtlas` 组件（挂在实体上，存储 SFF 数据）
- 每帧查询 `sprite_data` 获取实际尺寸和 UV
- 应用 `axis_offset` 调整渲染位置

---

## 4. 游戏循环集成

```rust
// game/src/main.rs（logic_update 修改）

fn logic_update(&mut self) {
    // 1. 输入处理
    // ...

    // 2. 状态机更新（推进动画帧）
    for (_, sm) in self.world.query_mut::<&mut StateMachine>() {
        sm.update();  // 返回当前帧，但这里不需要
    }

    // 3. 同步碰撞盒（新增）
    sync_hitboxes_system(&mut self.world);

    // 4. 物理系统
    // ...

    // 5. 碰撞检测（现在使用实时更新的碰撞盒）
    let hit_events = collision_system(&mut self.world);

    // 6. 战斗系统
    // ...
}
```

---

## 5. 数据加载流程

```
启动时：
1. 加载 KFM 的 kfm.sff（SFF 解析器）
   └─> 构建 SpriteAtlas（group/image -> SpriteData 映射）

2. 加载 KFM 的 kfm.air（AIR 解析器）
   └─> 解析所有 Action，构建 Animation 列表
   └─> 每个 Action 的每帧包含：
       - group/image（引用 SFF）
       - duration
       - Clsn1/Clsn2 碰撞盒

3. 创建角色实体时：
   - 挂载 SpriteAtlas 组件（引用 SFF 数据）
   - StateMachine 初始化，播放 Action 0（Idle）
```

---

## 6. 与现有系统的兼容性

**保留不变**：
- `LogicVec2`、`LogicRect` 等整数坐标系统
- `Position`、`Velocity`、`Facing` 等 ECS 组件
- `collision_system`、`combat_system` 的逻辑（只是碰撞盒数据来源改变）

**需要修改**：
- `StateMachine`：增加动画播放逻辑
- `HitboxManager`：从静态组件变为动态更新
- 渲染循环：删除固定尺寸假设，查询 SpriteAtlas

**新增**：
- `SpriteFrame`、`Animation` 数据结构
- `SpriteAtlas` 组件
- `sync_hitboxes_system` 系统
- SFF/AIR 解析器（A-4、A-5）

---

## 7. 实现顺序

1. **A-4**：实现 SFF 解析器
   - 输入：`kfm.sff` 文件
   - 输出：`SpriteAtlas`（group/image -> SpriteData）

2. **A-5**：实现 AIR 解析器
   - 输入：`kfm.air` 文件
   - 输出：`Vec<Animation>`（Action 列表）

3. **A-6**：扩展 `SpriteFrame` 和 `StateMachine`
   - 定义新数据结构
   - 实现 `sync_hitboxes_system`

4. **A-7**：改造渲染层
   - 删除 `FIGHTER_W/H`
   - 查询 `SpriteAtlas` 获取实际尺寸
   - 应用轴点偏移

5. **A-8**：KFM 验证
   - 加载 `kfm.sff` + `kfm.air`
   - 播放 Action 0（Idle）
   - 验证碰撞盒显示

---

## 8. 测试计划

**单元测试**：
- SFF 解析器：读取 `kfm.sff`，验证精灵数量、尺寸、轴点
- AIR 解析器：读取 `kfm.air`，验证 Action 0 的帧数、碰撞盒

**集成测试**：
- 加载 KFM，播放 Idle 动画，验证帧切换正确
- 播放 Attack 动画，验证碰撞盒在正确帧激活
- 验证水平翻转时碰撞盒也翻转

**视觉验证**：
- F1 切换 debug 渲染，显示碰撞盒
- 观察 Idle 动画是否流畅
- 观察 Attack 时 hitbox 是否在拳头位置
