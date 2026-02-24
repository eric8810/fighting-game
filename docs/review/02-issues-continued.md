# 代码审查问题清单（续）

> 审查日期：2026-02-24

---

## 逻辑正确性问题

### LOGIC-01：物理系统与 tickle_core 实现存在细微差异

- **位置**：`game/src/main.rs:409` vs `crates/tickle_core/src/systems/physics.rs:52`
- **描述**：主循环的摩擦力条件是 `pos.pos.y <= GROUND_Y`（包含地面），而 `tickle_core` 的 `friction_system` 条件是 `pos.pos.y == GROUND_Y`（仅地面）。两者行为不同：主循环在角色 Y 坐标为负时也会施加摩擦力（理论上不应发生，但边界情况下可能出现）。
- **影响**：若将来切换到使用 `tickle_core` 的物理系统，行为会悄然改变。

---

### LOGIC-02：跳跃时速度 X 不清零

- **位置**：`game/src/main.rs:509-512`
- **描述**：进入 Jump 状态时只设置 `vel.vel.y = JUMP_VEL`，不清零 `vel.vel.x`。地面摩擦力在空中不生效，所以起跳前的水平速度会完整保留到空中。
- **影响**：从奔跑状态起跳时，角色会以全速水平飞行，物理感觉不自然。

---

### LOGIC-03：训练模式无限 HP 用 `query` 而非 `query_mut`

- **位置**：`game/src/menu.rs:270-275`
- **描述**：训练模式每帧回满血量时，代码写的是 `world.query::<(&Player1, &mut Health)>().iter()`，但 `query` 返回的是共享引用，`&mut Health` 在这里实际上是通过 `UnsafeCell` 内部可变性实现的（hecs 的 `query` 允许这样做，但语义上应该用 `query_mut`）。
- **影响**：代码意图不清晰，且在 hecs 的某些版本中可能触发 panic（若同时有其他查询持有同一组件的引用）。

---

### LOGIC-04：`should_run_logic` 在 RoundIntro 期间也运行游戏逻辑

- **位置**：`game/src/menu.rs:151-153`
- **描述**：`should_run_logic()` 在 `GameState::RoundIntro` 时返回 `true`，意味着玩家在回合介绍动画播放期间可以移动和攻击。
- **影响**：玩家可以在 "ROUND 1 FIGHT!" 动画期间提前行动，破坏格斗游戏的节奏感。

---

### LOGIC-05：回合结束后未清零速度

- **位置**：`game/src/menu.rs:362-395`（`reset_fighters`）
- **描述**：`reset_fighters` 正确重置了位置、状态机、血量、能量槽，但 `Velocity` 也被重置为 `ZERO`（这是正确的）。然而 `Facing` 组件没有被重置——P1 和 P2 的朝向在回合重置后保持上一回合结束时的状态，而不是回到初始的"面对对手"方向。
- **影响**：新回合开始时角色可能背对对手。

---

### LOGIC-06：命中检测只在 `state_frame == 1` 触发

- **位置**：`game/src/main.rs:448, 467`
- **描述**：攻击命中判定只在 `state_frame == 1`（进入攻击状态后的第一帧）触发，之后的帧完全没有判定。
- **影响**：攻击的"活跃帧"（active frames）概念完全丢失——真实格斗游戏中攻击有多帧活跃窗口，而这里只有一帧瞬间判定。

---

## 架构设计问题

### ARCH-01：物理系统在主循环中重复实现

- **位置**：`game/src/main.rs:400-428` vs `crates/tickle_core/src/systems/physics.rs`
- **描述**：`tickle_core` 中有完整的 `velocity_system`、`gravity_system`、`ground_detection_system`、`friction_system`，但主循环把这四个系统合并成一个内联循环重新实现了一遍。
- **影响**：两套物理实现并存，任何修改都需要同步两处；`tickle_core` 的物理系统测试覆盖良好，但主循环的内联版本没有测试。

---

### ARCH-02：`FighterColor` 是游戏层私有组件，但混入了 ECS 查询

- **位置**：`game/src/main.rs:915, 734`
- **描述**：`FighterColor` 是 `game` crate 内部的渲染辅助组件，但它被 spawn 到 ECS 世界中，与核心游戏组件混在一起。渲染信息（颜色）和游戏逻辑信息（位置、状态）耦合在同一个实体上。
- **影响**：违反了逻辑/渲染分离的设计原则；若将来需要序列化游戏状态用于回滚，`FighterColor` 也会被包含进去（尽管它不影响游戏逻辑）。

---

### ARCH-03：`GameLoop::tick` 的 FPS 计数器在 `tick_with_dt` 中也被调用

- **位置**：`game/src/game_loop.rs:153, 186`
- **描述**：`tick_with_dt` 是为确定性测试设计的，但它也调用了 `self.frame_counter.tick()`，这会修改 `FrameCounter` 的内部状态（`last_second` 使用 `Instant::now()`）。在测试中调用 `tick_with_dt` 会污染 FPS 计数器的时间基准。
- **影响**：测试中的 FPS 计数器行为不可预测；`tick_with_dt` 的"确定性"承诺被 `Instant::now()` 破坏。

---

### ARCH-04：`NetworkMode` 解析但从不使用

- **位置**：`game/src/main.rs:949-950`
- **描述**：`NetworkMode::from_args()` 被调用并记录日志，但返回值被丢弃，`network_mode` 变量从未被读取。
- **影响**：`--online` 参数被静默忽略，用户无法得到任何提示说明联机功能未实现。

---

## 渲染问题

### RENDER-01：地面线渲染宽度计算有误

- **位置**：`game/src/main.rs:725`
- **描述**：地面线的宽度计算为 `screen_w + camera_x * 2.0`。当 `camera_x` 为正值（摄像机向右移动）时，地面线会比屏幕宽，这是多余的；当 `camera_x` 为负值时（理论上不应发生，但 clamp 前可能短暂出现），宽度会缩小。正确做法应该是固定为 `screen_w`，或者用舞台总宽度。
- **影响**：视觉上无明显问题（多余的部分被裁剪），但逻辑不正确。

---

### RENDER-02：精灵动画帧率硬编码为每 8 逻辑帧

- **位置**：`game/src/main.rs:792`
- **描述**：`let frame = (sm.state.state_frame / 8) as i32 % frames_in_row;` 将所有动画的帧率固定为每 8 逻辑帧（即 7.5 FPS）。不同动作（如攻击、行走、受击）应有不同的动画速度。
- **影响**：所有动画播放速度相同，无法针对不同状态调整动画节奏。

---

### RENDER-03：Pass 2 注释标签重复

- **位置**：`game/src/main.rs:872, 884`
- **描述**：战士渲染和文字渲染都被标注为 "Pass 2"，实际上文字渲染应该是 "Pass 3"。
- **影响**：代码可读性问题，不影响功能。

---

### RENDER-04：`stage.render_layers` 忽略 `screen_h` 和 `ground_screen_y` 参数

- **位置**：`game/src/stage.rs:162-165`
- **描述**：`render_layers` 接受 `screen_h` 和 `ground_screen_y` 参数，但函数签名中用 `_screen_h` 和 `_ground_screen_y` 标记为未使用。背景层的 Y 坐标是绝对像素值，没有相对于地面线进行定位。
- **影响**：背景层位置在不同窗口高度下不会自适应；地面线与背景层之间的视觉对齐是巧合而非设计。

---

## 测试覆盖问题

### TEST-01：确定性测试文件有占位符但无实际测试

- **位置**：`crates/tickle_network/tests/determinism_test.rs:41, 125`
- **描述**：文件中有 `// DETERMINISM_TEST_PLACEHOLDER` 和 `// DETERMINISM_TESTS_PLACEHOLDER` 注释，以及完整的辅助函数（`simulate_fighter_frame`、`simulate_frame`、`generate_input_sequence`），但没有任何 `#[test]` 函数。
- **影响**：`cargo test -p tickle_network --test determinism_test` 会成功但不执行任何断言，给人一种"已测试"的错误印象。

---

### TEST-02：`combat.rs` 测试未覆盖 `force_enter` 路径

- **位置**：`crates/tickle_core/src/systems/combat.rs` 测试部分
- **描述**：`test_hit_resolution_applies_hitstun` 只断言状态变为 `Hitstun`，但没有验证 `stun_duration` 是否被正确设置（因为当前实现根本没有设置它）。测试通过了，但掩盖了 BUG-01 的存在。
- **影响**：测试给出了错误的安全感；需要增加对 `stun_duration` 的断言。

---

### TEST-03：`game_loop.rs` 的 `FrameCounter` 测试不完整

- **位置**：`game/src/game_loop.rs:400-404`
- **描述**：`frame_counter_initial_state` 只测试了初始 FPS 为 0，没有测试 `tick()` 在一秒后返回正确 FPS 的行为（因为这依赖 `Instant::now()`，难以在单元测试中控制）。
- **影响**：FPS 计数逻辑缺乏测试覆盖。

---

## 其他细节问题

### MISC-01：`#[allow(dead_code)]` 掩盖了真实问题

- **位置**：`game/src/stage.rs:5, 43, 51`；`game/src/menu.rs:84, 108`
- **描述**：多处使用 `#[allow(dead_code)]` 来压制警告，而不是真正使用或删除这些代码。`StageData`、`CameraLimits`、`ParallaxLayer` 的字段，以及 `MenuSystem` 的多个字段（`training_show_hitboxes`、`training_show_framedata`）都被标记为允许死代码。
- **影响**：这些字段要么是真正的死代码（应删除），要么是计划中的功能（应实现）；`#[allow]` 掩盖了这一区别。

---

### MISC-02：`GameAudioEvent::StateChangeSound` 从未被消费

- **位置**：`game/src/main.rs:647-658`
- **描述**：`audio_events_from_hits` 可能返回 `GameAudioEvent::StateChangeSound`，但主循环的 `match` 分支中 `_ => AudioEvent::StopMusic` 会将其映射为停止音乐——这是一个错误的 fallback，注释也写着 `// unreachable for hit-derived events`，但实际上并非不可达。
- **影响**：若 `audio_events_from_hits` 将来返回非 `HitSound` 事件，会意外触发停止音乐。

---

### MISC-03：`DeterministicRng` 的 LCG 参数来自 Knuth，但未注明来源

- **位置**：`crates/tickle_network/src/lib.rs:107-111`
- **描述**：LCG 使用的乘数 `6364136223846793005` 和加数 `1442695040888963407` 是 Knuth 的经典参数，但代码中没有注释说明来源或为何选择这组参数。
- **影响**：可维护性问题；未来维护者可能不知道这些"魔法数字"的来源。

---

### MISC-04：`simulate_fighter_frame` 中跳跃速度判断条件有误

- **位置**：`crates/tickle_network/tests/determinism_test.rs:84`
- **描述**：`if fighter.state_machine.state_frame() == 0` 用于判断"刚进入跳跃状态"，但 `state_frame` 在 `change_state` 时被重置为 0，然后在 `update()` 中递增。由于 `try_transition` 在 `update()` 之前调用，`state_frame` 在进入跳跃的第一帧确实为 0，但这个判断依赖了内部实现细节，脆弱性较高。
- **影响**：若 `state_frame` 的初始值或递增时机改变，跳跃速度将不再被设置。
