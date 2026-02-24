# 代码审查问题清单

> 审查日期：2026-02-24

---

## 严重 Bug（影响核心玩法）

### BUG-01：受击硬直（Hitstun）不生效

- **位置**：`crates/tickle_core/src/systems/combat.rs:57`
- **描述**：`hit_resolution_system` 调用 `change_state(StateType::Hitstun)` 但没有设置持续时长。`StateMachine` 提供了 `force_enter(state, duration)` 方法专门处理此场景，但未被调用。
- **后果**：被打后角色立刻可以行动，硬直帧完全无效。
- **修复方向**：将 `defender.state.change_state(StateType::Hitstun)` 替换为 `sm.force_enter(StateType::Hitstun, event.hitbox.hitstun)`。

---

### BUG-02：主循环命中处理绕过战斗系统

- **位置**：`game/src/main.rs:484-495`
- **描述**：`logic_update` 检测到命中后直接调用 `hp.take_damage()`，完全跳过了 `combat.rs` 的 `hit_resolution_system`。
- **后果**：以下功能全部失效：
  - 连击伤害衰减（combo scaling）
  - 击退（knockback）不应用
  - 能量槽（power gauge）不增加
  - 硬直时长不设置
- **修复方向**：命中后构造 `CombatEntity`，调用 `hit_resolution_system`，再将结果写回 ECS 组件。

---

### BUG-03：连击计数器永不重置

- **位置**：`crates/tickle_core/src/systems/combat.rs:65`
- **描述**：每次命中递增 `combo_count`，但没有任何地方在连击断开（角色回到 Idle/Walk 等状态）时将其归零。
- **后果**：伤害衰减跨多个独立连击持续累积，后续连击伤害越来越低。
- **修复方向**：在 `StateMachine::update()` 检测到硬直结束、状态回到 Idle 时，通知外部重置 `combo_count`；或将 `combo_count` 移入 `StateMachine` 统一管理。

---

### BUG-04：面朝方向永不更新

- **位置**：`game/src/main.rs:169-205`（`spawn_fighters`）
- **描述**：P1 初始朝右、P2 初始朝左，但游戏逻辑中没有任何代码根据双方相对位置更新 `Facing` 组件。
- **后果**：角色可以背对对手攻击；击退方向也因此出错（见 BUG-05）。
- **修复方向**：每帧 `logic_update` 末尾，比较双方 X 坐标，更新各自 `Facing`。

---

### BUG-05：击退方向硬编码，不考虑面朝方向

- **位置**：`game/src/main.rs:458, 477`
- **描述**：P1 攻击 P2 的击退固定为 `LogicVec2::new(300, 0)`（向右），P2 攻击 P1 固定为 `(-300, 0)`（向左），与实际站位无关。
- **后果**：若双方位置互换，击退方向会把被打者推向攻击者。
- **修复方向**：根据攻击者与防御者的相对 X 坐标决定击退方向符号。

---

## 未集成的完整系统（死代码）

### DEAD-01：碰撞检测系统（hitbox/hurtbox）未接入

- **位置**：`crates/tickle_core/src/systems/collision.rs:23`
- **描述**：`collision_system()` 实现完整，支持 hitbox 与 hurtbox 的 AABB 检测，但主循环从未调用。
- **现状**：主循环用简单距离阈值（80像素）代替真实碰撞检测。

---

### DEAD-02：推箱分离（Pushbox）未接入

- **位置**：`crates/tickle_core/src/systems/collision.rs:64-104`
- **描述**：`pushbox_separation_system()` 实现完整，但从未在游戏循环中调用。
- **后果**：两名角色可以完全重叠，无物理分离。

---

### DEAD-03：动画系统未接入

- **位置**：`crates/tickle_core/src/systems/animation.rs`
- **描述**：`SpriteAnimation` 组件和 `animation_system()` 实现完整，但主循环手动在 `main.rs:756-808` 计算 UV 坐标。
- **后果**：动画逻辑分散，无法复用，且帧速率硬编码（每8逻辑帧切一张）。

---

### DEAD-04：指令识别系统未接入

- **位置**：`crates/tickle_core/src/input.rs:136-342`
- **描述**：QCF（236）、QCB（214）、DP（623）、HCF（41236）等指令识别完整实现，但 `try_transition` 只检查单帧按键，从未调用指令识别器。
- **后果**：无法输入任何必杀技指令。

---

### DEAD-05：角色数据（RON）加载系统未接入

- **位置**：`crates/tickle_core/src/character_select.rs`
- **描述**：完整的 RON 文件加载、角色/招式数据解析系统，但主游戏从未使用。
- **后果**：所有攻击帧数据硬编码在 `state_machine.rs:112-120`，无法按角色定制。

---

## 硬编码问题

### HARD-01：所有攻击共用同一套帧数据

- **位置**：`crates/tickle_core/src/state_machine.rs:112-120`
- **描述**：`try_transition` 中所有攻击都使用固定的 `AttackData { total_frames: 30, cancel_windows: [5-20] }`。
- **后果**：无法区分轻/中/重攻击的帧数差异。

---

### HARD-02：所有按键攻击映射到 Attack(0)

- **位置**：`crates/tickle_core/src/state_machine.rs:192-193, 211, 227, 240, 251, 264`
- **描述**：A/B/C 三个按键全部触发 `StateType::Attack(0)`，攻击 ID 永远为 0。
- **后果**：无法区分不同攻击动作，动画也只能播放同一行。

---

## 代码质量问题

### QUALITY-01：热路径中的 log::info! 调用

- **位置**：`game/src/main.rs:801-806`
- **描述**：每次 `state_frame == 0`（即每次进入新状态）都调用 `log::info!()`，在 release 构建中若日志级别未关闭会有性能开销。
- **修复方向**：改为 `log::debug!()` 或用 `#[cfg(debug_assertions)]` 包裹。

---

### QUALITY-02：死字段 `TextRenderer::cache`

- **位置**：`game/src/text_renderer.rs:25`
- **描述**：`cache` 字段声明但从未读取，触发 clippy 警告。

---

### QUALITY-03：未使用的方法 `UIRenderer::set_names`

- **位置**：`game/src/ui.rs:174`
- **描述**：`set_names()` 方法实现但从未调用，触发 clippy 警告。

---

### QUALITY-04：函数参数过多

- **位置**：`game/src/quad_renderer.rs:378, 403, 427`
- **描述**：`draw`、`draw_overlay`、`draw_internal` 参数数量分别为 9、9、10，超过 clippy 建议的 7 个上限。
- **修复方向**：将相关参数封装为结构体（如 `DrawParams`）。

---

### QUALITY-05：冗余导入

- **位置**：`game/src/menu.rs:2`
- **描述**：`use tickle_audio;` 是冗余导入，触发 clippy 警告。

---

## 待实现功能（文档中已规划）

### TODO-01：在线联机网络层

- **文档**：`docs/10-development-todo.md` Phase 5.4
- **描述**：GGRS 基础设施已就绪，但 UDP socket、P2P 连接、匹配系统均未实现。`NetworkMode::Online` 解析了但从未使用。

### TODO-02：确定性验证测试

- **文档**：`crates/tickle_network/tests/determinism_test.rs`
- **描述**：测试文件存在但测试体为空，无法验证回滚网络代码的确定性。

### TODO-03：训练模式帧数据显示

- **文档**：`docs/10-development-todo.md` Phase 6.4
- **描述**：训练模式已有基础，但帧优劣势数据、输入历史显示均未实现。
