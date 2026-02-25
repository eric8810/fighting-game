# 修复任务追踪

> 创建日期：2026-02-24
> 来源：`01-issues.md`、`02-issues-continued.md`、`03-sprite-rendering-issues.md`

状态说明：`[ ]` 待处理 · `[~]` 进行中 · `[x]` 已完成

---

## 第一优先级：精灵渲染（当前可见问题）

| 状态 | ID | 任务 | 文件 | 参考 |
|------|----|------|------|------|
| [x] | S-1 | 修正帧高度常量：`FRAME_H` 从 `116.0` 改为 `109.0`，`TEXTURE_H` 保持 `654.0` | `game/src/main.rs:760` | SPRITE-01 |
| [x] | S-2 | 同步修正 atlas 文件：`ryu_atlas.ron` 中 `frame_height: 116` 改为 `109` | `assets/sprites/ryu_atlas.ron:7` | SPRITE-01 |
| [x] | S-3 | 修正渲染画框尺寸：`FIGHTER_W/H` 改为与精灵帧等比的值（`100.0 × 109.0`） | `game/src/main.rs:47-48` | SPRITE-02 |
| [x] | S-4 | 修复动画循环逻辑：`looping: false` 的状态（Attack、Hitstun、Knockdown）播完后钳制到最后一帧，不取模 | `game/src/main.rs:783` | SPRITE-03 |
| [x] | S-5 | 各状态使用 atlas 中定义的独立帧时长，而非统一硬编码为 `/ 8` | `game/src/main.rs:779` | SPRITE-03、RENDER-02 |

---

## 第二优先级：核心战斗逻辑 Bug

| 状态 | ID | 任务 | 文件 | 参考 |
|------|----|------|------|------|
| [ ] | B-1 | 修复硬直不生效：`hit_resolution_system` 改用 `force_enter(Hitstun, hitstun_duration)` | `tickle_core/src/systems/combat.rs:57` | BUG-01 |
| [ ] | B-2 | 修复命中处理绕过战斗系统：主循环命中后调用 `hit_resolution_system`，写回 knockback、gauge、stun | `game/src/main.rs:484-495` | BUG-02 |
| [ ] | B-3 | 修复连击计数器不重置：硬直结束回到 Idle 时归零 `combo_count` | `tickle_core/src/systems/combat.rs:65` | BUG-03 |
| [ ] | B-4 | 修复面朝方向不更新：每帧根据双方 X 坐标更新 `Facing` 组件 | `game/src/main.rs:logic_update` | BUG-04 |
| [ ] | B-5 | 修复击退方向硬编码：根据攻击者与防御者相对位置决定击退方向符号 | `game/src/main.rs:458, 477` | BUG-05 |
| [ ] | B-6 | 修复攻击活跃帧只有 1 帧：支持多帧活跃窗口（`state_frame` 在 startup 到 startup+active 范围内均判定） | `game/src/main.rs:448, 467` | LOGIC-06 |
| [ ] | B-7 | 修复回合重置后朝向不恢复：`reset_fighters` 中重置 `Facing` 为初始方向 | `game/src/menu.rs:362-395` | LOGIC-05 |

---

## 第三优先级：逻辑正确性

| 状态 | ID | 任务 | 文件 | 参考 |
|------|----|------|------|------|
| [ ] | L-1 | 修复跳跃时水平速度不清零：进入 Jump 状态时将 `vel.vel.x` 归零 | `game/src/main.rs:509-512` | LOGIC-02 |
| [ ] | L-2 | 修复 RoundIntro 期间可以行动：`should_run_logic` 在 RoundIntro 时返回 `false` | `game/src/menu.rs:151-153` | LOGIC-04 |
| [ ] | L-3 | 修复训练模式无限 HP 查询方式：改用 `query_mut` | `game/src/menu.rs:270-275` | LOGIC-03 |
| [ ] | L-4 | 统一物理系统：主循环改用 `tickle_core` 的 `physics` 系统，删除内联重复实现 | `game/src/main.rs:400-428` | ARCH-01、LOGIC-01 |

---

## 第四优先级：接入已有系统（死代码激活）

| 状态 | ID | 任务 | 文件 | 参考 |
|------|----|------|------|------|
| [ ] | D-1 | 接入 pushbox 分离系统：主循环调用 `pushbox_separation_system`，防止角色重叠 | `tickle_core/src/systems/collision.rs:64` | DEAD-02 |
| [ ] | D-2 | 接入真实碰撞检测：用 `collision_system`（hitbox/hurtbox AABB）替换距离阈值判定 | `tickle_core/src/systems/collision.rs:23` | DEAD-01 |
| [ ] | D-3 | 接入 atlas 加载：读取 `ryu_atlas.ron`，用其数据驱动 UV 计算，删除 `main.rs` 中的硬编码 `match` | `game/src/main.rs:756-808` | SPRITE-04、DEAD-03 |
| [ ] | D-4 | 接入指令识别：`try_transition` 中调用 `CommandRecognizer`，支持 QCF/DP 等必杀技输入 | `tickle_core/src/input.rs:136` | DEAD-04 |
| [ ] | D-5 | 区分攻击按键：A/B/C 映射到不同 `Attack(id)`，各自有独立帧数据 | `tickle_core/src/state_machine.rs:192` | HARD-01、HARD-02 |

---

## 第五优先级：测试补全

| 状态 | ID | 任务 | 文件 | 参考 |
|------|----|------|------|------|
| [ ] | T-1 | 补全确定性测试：在 `determinism_test.rs` 中实现实际的 `#[test]` 函数 | `tickle_network/tests/determinism_test.rs` | TEST-01、TODO-02 |
| [ ] | T-2 | 修复 combat 测试：`test_hit_resolution_applies_hitstun` 增加对 `stun_duration` 的断言 | `tickle_core/src/systems/combat.rs` | TEST-02 |
| [ ] | T-3 | 修复 `simulate_fighter_frame` 跳跃判断：用更健壮的方式检测"刚进入跳跃"，不依赖 `state_frame == 0` | `tickle_network/tests/determinism_test.rs:84` | MISC-04 |

---

## 第六优先级：代码质量清理

| 状态 | ID | 任务 | 文件 | 参考 |
|------|----|------|------|------|
| [ ] | Q-1 | 热路径日志降级：`log::info!` 改为 `log::debug!` | `game/src/main.rs:801-806` | QUALITY-01 |
| [ ] | Q-2 | 删除死字段 `TextRenderer::cache` | `game/src/text_renderer.rs:25` | QUALITY-02 |
| [ ] | Q-3 | 删除或实现 `UIRenderer::set_names` | `game/src/ui.rs:174` | QUALITY-03 |
| [ ] | Q-4 | 封装 `QuadRenderer::draw` 参数为结构体 | `game/src/quad_renderer.rs:378` | QUALITY-04 |
| [ ] | Q-5 | 删除冗余导入 `use tickle_audio` | `game/src/menu.rs:2` | QUALITY-05 |
| [ ] | Q-6 | 修复渲染 Pass 注释：文字渲染标注为 "Pass 3" | `game/src/main.rs:884` | RENDER-03 |
| [ ] | Q-7 | 修复地面线宽度计算 | `game/src/main.rs:725` | RENDER-01 |
| [ ] | Q-8 | 修复背景层 Y 坐标不随窗口高度自适应 | `game/src/stage.rs:162` | RENDER-04 |
| [ ] | Q-9 | 修复音频事件 fallback 分支：`_ => AudioEvent::StopMusic` 改为正确处理或 `unreachable!()` | `game/src/main.rs:656` | MISC-02 |
| [ ] | Q-10 | 为 LCG 魔法数字添加来源注释（Knuth） | `tickle_network/src/lib.rs:107` | MISC-03 |
| [ ] | Q-11 | 清理 `#[allow(dead_code)]`：明确区分"待实现"与"应删除"的字段 | `game/src/stage.rs`、`game/src/menu.rs` | MISC-01 |
| [ ] | Q-12 | `NetworkMode` 未使用时给出明确提示，或移除 `--online` 参数解析 | `game/src/main.rs:949` | ARCH-04 |

---

## 架构重设计：精灵与碰撞盒系统（阻塞性问题）

> 当前的 grid sprite sheet 假设（等宽等高、行列索引）与真实格斗游戏精灵的本质矛盾。
> 这组任务是后续所有精灵/碰撞相关工作的前提，需要优先规划。

| 状态 | ID | 任务 | 说明 |
|------|----|------|------|
| [x] | A-1 | 调研 MUGEN SFF/AIR 格式，输出格式规范文档 | 见 `docs/review/04-mugen-format-research.md` |
| [x] | A-2 | 设计帧级碰撞盒数据结构：每个动画帧携带独立的 hitbox/hurtbox/pushbox 列表 | 见 `docs/review/05-frame-based-sprite-design.md` |
| [x] | A-3 | 设计每帧独立尺寸 + 原点偏移的精灵渲染方案 | 见 `docs/review/05-frame-based-sprite-design.md` |
| [x] | A-4 | 实现 MUGEN SFF v1/v2 解析器（读取精灵图 + 每帧偏移） | `tickle_mugen` crate，KFM 文件在 `assets/mugen/kfm/` |
| [x] | A-5 | 实现 MUGEN AIR 解析器（读取动画定义 + 每帧碰撞盒） | AIR 格式：每帧含 group/image/duration/clsn 数据 |
| [x] | A-6 | 将 `SpriteFrame` 扩展为包含 hitbox/hurtbox/pushbox，状态机切帧时同步更新 `HitboxManager` | `tickle_core/src/systems/animation.rs` |
| [x] | A-7 | 渲染层改为按帧的实际尺寸和原点偏移绘制，替换固定矩形 | `game/src/main.rs` 渲染循环 |
| [x] | A-8 | 用 KFM（Kung Fu Man）作为第一个测试角色，验证完整管线 | 资源：已下载到 `assets/mugen/kfm/` |

---

## 暂不处理（待规划）

| ID | 任务 | 参考 |
|----|------|------|
| F-1 | 实现在线联机网络层（UDP socket、P2P、匹配） | TODO-01 |
| F-2 | 实现训练模式帧数据显示 | TODO-03 |
| F-3 | 接入角色 RON 数据加载系统（`character_select.rs`） | DEAD-05 |
| F-4 | `FighterColor` 从 ECS 中分离，改为渲染层独立管理 | ARCH-02 |
| F-5 | `tick_with_dt` 中移除 `Instant::now()` 调用，保证确定性 | ARCH-03 |
