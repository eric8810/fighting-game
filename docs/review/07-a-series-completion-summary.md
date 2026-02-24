# A 系列任务完成总结

> 完成日期：2026-02-24
> 任务组：架构重设计 - 精灵与碰撞盒系统

---

## 已完成任务

### ✅ A-1: 调研 MUGEN SFF/AIR 格式
- **输出**：`docs/review/04-mugen-format-research.md`
- **内容**：完整的 SFF v1/v2 和 AIR 格式规范，包括压缩算法、调色板系统、碰撞盒定义

### ✅ A-2: 设计帧级碰撞盒数据结构
- **输出**：`docs/review/05-frame-based-sprite-design.md`
- **内容**：`SpriteFrame`、`Animation`、`SpriteAtlas` 数据结构设计，支持每帧独立的 hitbox/hurtbox/pushbox

### ✅ A-3: 设计每帧独立尺寸 + 原点偏移的精灵渲染方案
- **输出**：`docs/review/05-frame-based-sprite-design.md`
- **内容**：渲染层改造方案，支持每帧独立尺寸和原点偏移

### ✅ A-4: 实现 MUGEN SFF v1/v2 解析器
- **实现**：`crates/tickle_mugen/src/sff_v1.rs`
- **功能**：
  - 自动检测 SFF v1/v2 版本
  - SFF v1：使用 `mugen-sff` crate + PCX 解码
  - SFF v2：完整实现（header、palette nodes、sprite nodes）
  - 压缩格式：raw、RLE8、RLE5、LZ5
- **测试**：成功加载 KFM 的 281 个精灵，像素数据正确解压
- **文档**：`docs/review/06-sff-v2-implementation.md`

### ✅ A-5: 实现 MUGEN AIR 解析器
- **实现**：`crates/tickle_mugen/src/air.rs`
- **功能**：
  - 解析 Action 定义（action number、frames）
  - 解析 Clsn1/Clsn2 碰撞盒（default 和 per-frame）
  - 解析帧参数（group、image、offset、duration、flip）
- **测试**：成功加载 KFM 的 117 个 actions
- **数据结构**：
  ```rust
  pub struct Air {
      actions: HashMap<u32, Action>,
  }

  pub struct Action {
      pub number: u32,
      pub frames: Vec<Frame>,
      pub clsn2_default: Vec<Clsn>,  // hurtbox
      pub clsn1_default: Vec<Clsn>,  // hitbox
  }

  pub struct Frame {
      pub group: u16,
      pub image: u16,
      pub x_offset: i16,
      pub y_offset: i16,
      pub duration: i32,
      pub flip: FlipFlags,
      pub clsn2: Option<Vec<Clsn>>,  // frame-specific hurtbox
      pub clsn1: Option<Vec<Clsn>>,  // frame-specific hitbox
  }

  pub struct Clsn {
      pub left: i16,
      pub top: i16,
      pub right: i16,
      pub bottom: i16,
  }
  ```

---

## 待完成任务（需要后续迭代）

### ⏳ A-6: 扩展 SpriteFrame 并同步 HitboxManager
**目标**：将 AIR 数据集成到 ECS 系统中

**实现计划**：
1. 在 `tickle_core/src/components.rs` 中扩展 `AnimationState`：
   ```rust
   pub struct AnimationState {
       pub current_action: u32,
       pub current_frame: usize,
       pub frame_timer: i32,
       pub air_data: Arc<Air>,  // 引用 AIR 数据
   }
   ```

2. 在 `tickle_core/src/systems/animation.rs` 中实现 `sync_hitboxes_system`：
   ```rust
   pub fn sync_hitboxes_system(world: &mut World) {
       for (_, (anim, hitbox_mgr)) in world.query_mut::<(&AnimationState, &mut HitboxManager)>() {
           if let Some(action) = anim.air_data.get_action(anim.current_action) {
               if let Some(frame) = action.frames.get(anim.current_frame) {
                   // 更新 hitbox_mgr 的 hitboxes/hurtboxes
                   hitbox_mgr.hitboxes = frame.clsn1.clone()
                       .unwrap_or_else(|| action.clsn1_default.clone());
                   hitbox_mgr.hurtboxes = frame.clsn2.clone()
                       .unwrap_or_else(|| action.clsn2_default.clone());
               }
           }
       }
   }
   ```

3. 在主循环中调用 `sync_hitboxes_system`（在 `animation_system` 之后）

**依赖**：需要先实现 A-7（渲染层改造）才能完整测试

---

### ⏳ A-7: 渲染层改造
**目标**：支持每帧独立尺寸和原点偏移

**实现计划**：
1. 在 `game/src/main.rs` 渲染循环中：
   ```rust
   // 当前（错误）：
   let rect = [screen_x, screen_y, FIGHTER_W, FIGHTER_H];

   // 改为（正确）：
   if let Some(action) = air.get_action(anim.current_action) {
       if let Some(frame) = action.frames.get(anim.current_frame) {
           if let Some(sprite) = sff.get_sprite(frame.group, frame.image) {
               let w = sprite.width as f32;
               let h = sprite.height as f32;
               let ax = sprite.axis_x as f32;
               let ay = sprite.axis_y as f32;

               // 应用原点偏移和帧偏移
               let render_x = screen_x + frame.x_offset as f32 - ax;
               let render_y = screen_y + frame.y_offset as f32 - ay;

               let rect = [render_x, render_y, w, h];
               // 绘制精灵...
           }
       }
   }
   ```

2. 删除固定的 `FIGHTER_W/H` 常量

3. 支持 `FlipFlags`（水平/垂直翻转）

**依赖**：需要将 SFF 和 AIR 数据加载到游戏中

---

### ⏳ A-8: 用 KFM 验证完整管线
**目标**：端到端测试 SFF + AIR 集成

**实现计划**：
1. 创建 `KfmCharacter` 加载器：
   ```rust
   pub struct KfmCharacter {
       pub sff: SffV1,
       pub air: Air,
   }

   impl KfmCharacter {
       pub fn load(base_path: &str) -> Result<Self> {
           let sff = SffV1::load(format!("{}/kfm.sff", base_path))?;
           let air = Air::load(format!("{}/kfm.air", base_path))?;
           Ok(Self { sff, air })
       }
   }
   ```

2. 在 `game/src/main.rs` 中加载 KFM：
   ```rust
   let kfm = KfmCharacter::load("assets/mugen/kfm")?;
   ```

3. 测试场景：
   - 站立动画（Action 0）：11 帧循环
   - 行走动画（Action 20/21）：16 帧循环
   - 跳跃动画（Action 40-47）：多个 action 切换
   - 攻击动画（Action 200+）：带 Clsn1 hitbox

4. 验证项：
   - ✅ 精灵正确显示（尺寸、位置）
   - ✅ 动画流畅播放（duration 正确）
   - ✅ 碰撞盒正确显示（F1 调试模式）
   - ✅ 碰撞检测正常工作

---

## 技术债务和后续优化

### 1. 纹理 Atlas 优化
当前每个精灵都是独立的纹理，需要实现：
- 将多个精灵打包到单个纹理 atlas
- 减少 draw call 数量
- 提升渲染性能

### 2. 动画状态机集成
当前 `StateMachine` 使用硬编码的状态，需要：
- 将 AIR action number 映射到状态机状态
- 支持动态加载角色数据
- 实现 CNS 文件解析（状态定义）

### 3. 链接精灵优化
SFF v2 的 linked sprites（format=1）当前实现较简单，需要：
- 优化内存使用（共享像素数据）
- 支持链接调色板

### 4. 压缩格式补全
当前未实现的格式：
- PNG（format=10/11/12）：可使用 `image` crate
- 需要时再添加

---

## 测试覆盖率

### SFF 解析器
- ✅ SFF v1：PCX 解码
- ✅ SFF v2：header、palette nodes、sprite nodes
- ✅ 压缩格式：raw、RLE8、RLE5、LZ5
- ✅ 链接精灵：format=1
- ✅ KFM 测试：281 个精灵全部加载

### AIR 解析器
- ✅ Action 定义解析
- ✅ Clsn1/Clsn2 解析（default 和 per-frame）
- ✅ 帧参数解析（group、image、offset、duration、flip）
- ✅ KFM 测试：117 个 actions 全部加载
- ⚠️ Loopstart 标记：已识别但未处理

---

## 提交清单

### 新增文件
- `crates/tickle_mugen/src/sff_v1.rs`：SFF v1/v2 解析器
- `crates/tickle_mugen/src/air.rs`：AIR 解析器
- `crates/tickle_mugen/src/error.rs`：错误类型
- `crates/tickle_mugen/src/lib.rs`：模块导出
- `crates/tickle_mugen/Cargo.toml`：依赖配置
- `docs/review/06-sff-v2-implementation.md`：SFF v2 实现笔记

### 修改文件
- `docs/review/tasks.md`：更新 A-1 到 A-5 状态为已完成

### 测试
```bash
cargo test -p tickle_mugen test_load_kfm_sff -- --nocapture
cargo test -p tickle_mugen test_load_kfm_air -- --nocapture
```

---

## 下一步建议

1. **优先级 1**：完成 A-6 到 A-8（集成到游戏中）
2. **优先级 2**：修复 B 系列 bug（核心战斗逻辑）
3. **优先级 3**：接入已有系统（D 系列）
4. **优先级 4**：代码质量清理（Q 系列）

当前 A 系列的基础设施（SFF/AIR 解析器）已经完成，可以开始集成到游戏引擎中。
