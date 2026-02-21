# Tickle Fighting Engine - 开发任务清单

**版本：** v0.1.0-alpha
**更新日期：** 2026-02-21

---

## 阶段 0：项目初始化

### 0.1 项目结构搭建
- [ ] 创建 Cargo workspace 结构
  ```
  tickle/
  ├── Cargo.toml (workspace)
  ├── crates/
  │   ├── tickle_core/      # 核心逻辑（ECS、状态机）
  │   ├── tickle_render/    # 渲染层（wgpu）
  │   ├── tickle_audio/     # 音频层（kira）
  │   ├── tickle_network/   # 网络层（GGRS）
  │   └── tickle_assets/    # 资源管理
  ├── game/                 # 游戏主程序
  └── tools/                # 工具（帧数据编辑器）
  ```
- [ ] 配置 `Cargo.toml` 依赖
  ```toml
  [workspace]
  members = ["crates/*", "game"]

  [workspace.dependencies]
  winit = "0.30"
  wgpu = "23.0"
  kira = "0.9"
  ggrs = "0.10"
  hecs = "0.10"
  serde = { version = "1.0", features = ["derive"] }
  ron = "0.8"
  bytemuck = { version = "1.14", features = ["derive"] }
  ```
- [ ] 设置 `.gitignore`（target/, assets/temp/, *.log）
- [ ] 初始化 Git 仓库

### 0.2 开发环境配置
- [ ] 安装 Rust 工具链（rustup, cargo-watch, cargo-flamegraph）
- [ ] 配置 VSCode / RustRover IDE
- [ ] 安装 wgpu 依赖（Vulkan SDK / Metal）
- [ ] 验证跨平台编译（cargo build --target wasm32-unknown-unknown）

---

## 阶段 1：核心基础设施（预计 3-4 周）

### 1.1 游戏主循环与帧率系统
**文件：** `game/src/game_loop.rs`

- [ ] 实现固定时间步长（Fixed Timestep）
  - [ ] 逻辑帧率固定 60 FPS
  - [ ] 渲染帧率可变（60/120/144/240 Hz）
  - [ ] 累加器（Accumulator）防止时间漂移
- [ ] 实现渲染插值系统
  - [ ] `PreviousPosition` 组件
  - [ ] `interpolate()` 插值函数
  - [ ] `save_previous_position_system()`
- [ ] 实现帧率计数器（FPS Counter）
- [ ] 配置 wgpu `PresentMode`（VSync / NoVsync）
- [ ] 单元测试（验证 60 FPS 逻辑稳定性）

### 1.2 坐标系统与数学库
**文件：** `crates/tickle_core/src/math.rs`
**文件：** `crates/tickle_core/src/math.rs`

- [ ] 实现 `LogicCoord` 类型（i32 定点数）
- [ ] 实现 `LogicVec2` 结构体
  - [ ] 加减乘除运算符重载
  - [ ] `to_render()` 转换为 f32
  - [ ] `magnitude()` / `normalize()` 方法
- [ ] 实现 `LogicRect` 结构体
  - [ ] `intersects()` AABB 碰撞检测
  - [ ] `contains_point()` 点包含检测
- [ ] 单元测试（至少 20 个测试用例）

### 1.3 ECS 组件定义
**文件：** `crates/tickle_core/src/components.rs`

- [ ] 定义核心组件
  - [ ] `Position`
  - [ ] `PreviousPosition`（新增，用于插值）
  - [ ] `Velocity`
  - [ ] `Facing`
  - [ ] `Health`
  - [ ] `PowerGauge`
- [ ] 定义战斗组件
  - [ ] `FighterState`（状态机）
  - [ ] `InputBuffer`（输入缓冲）
  - [ ] `HitboxManager`（碰撞盒）
- [ ] 定义渲染组件
  - [ ] `SpriteAnimation`
  - [ ] `Transform`
- [ ] 为所有组件实现 `Clone` trait（回滚网络需要）

### 1.4 输入系统
**文件：** `crates/tickle_core/src/input.rs`

- [ ] 定义 `InputState` 结构体（按钮位图 + 方向）
- [ ] 实现 `InputBuffer`（16 帧循环队列）
  - [ ] `push()` 添加新输入
  - [ ] `get_history()` 获取历史输入
- [ ] 实现指令识别器 `CommandRecognizer`
  - [ ] QCF（↓↘→）识别
  - [ ] QCB（↓↙←）识别
  - [ ] DP（→↓↘）识别
  - [ ] HCF/HCB（半圆）识别
  - [ ] 双击方向（→→）识别
- [ ] 单元测试（模拟输入序列，验证指令识别）

### 1.5 资源管理系统
**文件：** `crates/tickle_assets/src/lib.rs`

- [ ] 定义 `AssetId` 类型（UUID 或字符串）
- [ ] 实现 `AssetManager<T>`
  - [ ] `load<T: Asset>()` 加载资源
  - [ ] `get<T>()` 获取资源引用（Arc）
  - [ ] 资源缓存（HashMap）
- [ ] 实现 RON 文件加载器
  - [ ] `MoveData` 招式数据加载
  - [ ] `CharacterData` 角色数据加载
- [ ] 错误处理（自定义 `AssetError` 类型）

---

## 阶段 2：渲染系统（预计 2-3 周）

### 2.1 wgpu 初始化
**文件：** `crates/tickle_render/src/lib.rs`

- [ ] 创建 `RenderContext` 结构体
  - [ ] 初始化 wgpu `Instance` / `Adapter` / `Device` / `Queue`
  - [ ] 创建 `Surface`（绑定 winit 窗口）
  - [ ] 配置 `SurfaceConfiguration`（分辨率、格式）
- [ ] 实现窗口 resize 处理
- [ ] 验证基础渲染（清屏为纯色）

### 2.2 纹理加载与 Atlas
**文件：** `crates/tickle_render/src/texture.rs`

- [ ] 实现 `Texture` 结构体（wgpu::Texture 封装）
- [ ] 实现 PNG 加载器（使用 `image` crate）
- [ ] 实现 `TextureAtlas`
  - [ ] 从 JSON 加载 Atlas 元数据（Aseprite 导出格式）
  - [ ] `get_sprite_uv()` 获取精灵 UV 坐标
- [ ] 纹理绑定组（wgpu::BindGroup）

### 2.3 Sprite Batch 渲染器
**文件：** `crates/tickle_render/src/sprite_batch.rs`

- [ ] 定义 `SpriteInstance` 结构体（顶点数据）
- [ ] 编写 WGSL 着色器
  - [ ] Vertex Shader（MVP 变换 + UV 传递）
  - [ ] Fragment Shader（纹理采样 + Alpha 混合）
- [ ] 实现 `SpriteBatchRenderer`
  - [ ] `begin()` 开始批次
  - [ ] `draw_sprite()` 添加精灵到批次
  - [ ] `flush()` 提交批次到 GPU
- [ ] 动态顶点缓冲（每帧更新）
- [ ] 测试：渲染 100 个精灵，验证批量绘制

### 2.4 调试渲染器
**文件：** `crates/tickle_render/src/debug_renderer.rs`

- [ ] 实现 `LineRenderer`（绘制线段）
- [ ] 实现 `DebugRenderer`
  - [ ] `draw_rect()` 绘制矩形框（碰撞盒可视化）
  - [ ] `draw_cross()` 绘制十字（位置标记）
  - [ ] `draw_text()` 绘制文本（帧数、坐标）
- [ ] 可通过按键切换显示/隐藏（F1 键）

---

## 阶段 3：游戏逻辑核心（预计 4-5 周）

### 3.1 状态机系统
**文件：** `crates/tickle_core/src/state_machine.rs`

- [ ] 定义 `StateType` 枚举（Idle / Walk / Jump / Attack 等）
- [ ] 实现 `StateMachine` 结构体
  - [ ] `update()` 状态转换逻辑
  - [ ] `enter_state()` 进入状态回调
  - [ ] `exit_state()` 退出状态回调
- [ ] 实现状态转换表
  - [ ] Idle → Walk / Jump / Crouch / Attack
  - [ ] Attack → Idle（动作结束）
  - [ ] Attack → Attack（取消链）
  - [ ] Hitstun → Idle（硬直结束）
- [ ] 单元测试（模拟输入，验证状态转换）

### 3.2 碰撞检测系统
**文件：** `crates/tickle_core/src/systems/collision.rs`

- [ ] 实现 `collision_system()`
  - [ ] 查询所有角色的 `Position` + `HitboxManager`
  - [ ] Hitbox vs Hurtbox AABB 检测
  - [ ] 生成 `HitEvent`（命中事件）
- [ ] 实现 `pushbox_separation_system()`
  - [ ] Pushbox 重叠时向外推开
  - [ ] 边界限制（不能推出舞台）
- [ ] 实现碰撞盒世界坐标转换
  - [ ] 考虑角色位置 + 朝向（镜像翻转）

### 3.3 战斗系统
**文件：** `crates/tickle_core/src/systems/combat.rs`

- [ ] 实现 `hit_resolution_system()`
  - [ ] 处理 `HitEvent`
  - [ ] 扣除生命值（考虑伤害衰减）
  - [ ] 增加气槽
  - [ ] 设置受击状态（Hitstun / Blockstun）
  - [ ] 应用击退（Knockback）
- [ ] 实现连招衰减表
  ```rust
  const COMBO_SCALING: [f32; 10] = [
      1.0, 1.0, 0.9, 0.8, 0.7, 0.6, 0.5, 0.5, 0.5, 0.5
  ];
  ```
- [ ] 实现气槽充能逻辑
  - [ ] 攻击命中 +20
  - [ ] 攻击被格挡 +10
  - [ ] 被攻击命中 +30

### 3.4 动画系统
**文件：** `crates/tickle_core/src/systems/animation.rs`

- [ ] 实现 `animation_system()`
  - [ ] 推进 `SpriteAnimation.frame_timer`
  - [ ] 切换到下一帧
  - [ ] 循环播放 / 单次播放
- [ ] 实现动画事件回调
  - [ ] 特定帧触发音效
  - [ ] 特定帧生成碰撞盒
- [ ] 根据 `FighterState` 自动切换动画

### 3.5 物理系统
**文件：** `crates/tickle_core/src/systems/physics.rs`

- [ ] 实现 `velocity_system()`
  - [ ] 应用速度到位置：`pos += vel`
  - [ ] 应用重力：`vel.y -= GRAVITY`（仅空中）
- [ ] 实现地面检测
  - [ ] `pos.y <= 0` 时落地
  - [ ] 落地时 `vel.y = 0`，状态转换为 `Idle`
- [ ] 实现摩擦力（地面减速）

---

## 阶段 4：音频系统（预计 1 周）

### 4.1 音频管理器
**文件：** `crates/tickle_audio/src/lib.rs`

- [ ] 初始化 kira `AudioManager`
- [ ] 实现 `SoundEffectPlayer`
  - [ ] `play_sound(id)` 播放音效
  - [ ] 音效缓存（预加载常用音效）
- [ ] 实现 `MusicPlayer`
  - [ ] `play_music(id)` 播放 BGM
  - [ ] `stop_music()` 停止 BGM
  - [ ] 循环播放
- [ ] 音量控制（主音量 / 音效 / 音乐分离）

### 4.2 音频事件系统
**文件：** `crates/tickle_core/src/systems/audio_events.rs`

- [ ] 监听 `HitEvent`，播放命中音效
- [ ] 监听 `StateChange`，播放动作音效
- [ ] 监听 `RoundStart`，播放 BGM

---

## 阶段 5：回滚网络（预计 3-4 周）

### 5.1 游戏状态快照
**文件：** `crates/tickle_network/src/snapshot.rs`

- [ ] 定义 `GameSnapshot` 结构体
  - [ ] 包含所有 ECS 组件数据
  - [ ] 实现 `Clone` trait
- [ ] 实现 `save_state(world) -> GameSnapshot`
  - [ ] 遍历所有实体，克隆组件
- [ ] 实现 `load_state(world, snapshot)`
  - [ ] 清空当前世界
  - [ ] 从快照恢复所有实体和组件

### 5.2 GGRS 集成
**文件：** `crates/tickle_network/src/lib.rs`

- [ ] 实现 `GGRSConfig` trait
  - [ ] 定义 `State = GameSnapshot`
  - [ ] 定义 `Input = InputState`
- [ ] 实现 `advance_frame()`（确定性逻辑更新）
  - [ ] 应用输入
  - [ ] 执行所有游戏系统
  - [ ] 禁止浮点运算（编译时检查）
- [ ] 实现 `save_game_state()` / `load_game_state()`
- [ ] 实现 `on_event()`（处理 GGRS 事件）

### 5.3 确定性验证
**文件：** `crates/tickle_network/tests/determinism_test.rs`

- [ ] 编写确定性测试
  ```rust
  #[test]
  fn test_determinism() {
      let input_sequence = [...];
      let state1 = run_simulation(input_sequence.clone());
      let state2 = run_simulation(input_sequence.clone());
      assert_eq!(state1, state2); // 必须完全一致
  }
  ```
- [ ] 测试 1000 帧模拟，验证状态一致性
- [ ] 测试随机数生成器（固定种子）

### 5.4 网络传输层
**文件：** `crates/tickle_network/src/transport.rs`

- [ ] 实现 UDP Socket（使用 `tokio` + `quinn`）
- [ ] 实现 P2P 连接（使用 `matchbox` 或自建）
- [ ] 实现房间匹配逻辑（简单版：房间码）
- [ ] 延迟测试工具（模拟 50ms / 100ms / 200ms 延迟）

---

## 阶段 6：游戏内容（预计 4-6 周）

### 6.1 第一个可玩角色
**文件：** `assets/characters/ryu/`

- [ ] 设计角色帧数据
  - [ ] 基础移动（站立 / 前进 / 后退 / 跳跃 / 蹲下）
  - [ ] 3 个普通攻击（轻拳 / 中拳 / 重拳）
  - [ ] 1 个特殊技（波动拳，QCF+A）
  - [ ] 1 个超必杀（真空波动拳，QCF×2+A）
- [ ] 绘制精灵动画（或使用占位符）
- [ ] 编写 RON 配置文件
- [ ] 测试所有招式的碰撞盒和帧数据

### 6.2 第一个舞台
**文件：** `assets/stages/dojo/`

- [ ] 设计舞台背景（视差 3 层）
- [ ] 设置舞台边界（宽度约 1200 像素）
- [ ] 添加 BGM
- [ ] 编写舞台配置文件

### 6.3 UI 系统
**文件：** `game/src/ui/`

- [ ] 实现血条 HUD
  - [ ] 平滑动画（受击时延迟减少）
- [ ] 实现气槽 HUD
  - [ ] 3 股显示，充能动画
- [ ] 实现回合计时器
- [ ] 实现连击数显示（Combo Counter）
- [ ] 实现胜负判定界面

### 6.4 游戏模式
**文件：** `game/src/modes/`

- [ ] 实现本地对战模式（2P）
  - [ ] 角色选择界面
  - [ ] 3 局 2 胜制
  - [ ] 回合结算
- [ ] 实现训练模式
  - [ ] 无限血量
  - [ ] 显示帧数据
  - [ ] 碰撞盒可视化
  - [ ] 输入历史显示

---

## 阶段 7：优化与发布（预计 2-3 周）

### 7.1 性能优化
- [ ] 使用 `cargo flamegraph` 分析 CPU 热点
- [ ] 优化渲染批次（减少 Draw Call）
- [ ] 优化 ECS 查询（使用 `hecs` 的 `Query` 缓存）
- [ ] 减少内存分配（使用对象池）
- [ ] 目标：稳定 60 FPS（1000 帧无掉帧）

### 7.2 跨平台测试
- [ ] Windows 测试（DX12 / Vulkan）
- [ ] Linux 测试（Vulkan）
- [ ] macOS 测试（Metal）
- [ ] Web 测试（WebGPU / WebGL2 fallback）
- [ ] 修复平台特定 bug

### 7.3 打包与发布
- [ ] 配置 `cargo-bundle`（生成可执行文件）
- [ ] 打包资源文件（assets/ 目录）
- [ ] 编写 README.md（安装说明、操作指南）
- [ ] 发布到 itch.io（alpha 版本）

---

## 里程碑时间线

| 阶段 | 预计时间 | 交付物 |
|------|---------|--------|
| 阶段 0 | 3 天 | 项目结构 + 开发环境 |
| 阶段 1 | 3-4 周 | 核心基础设施（坐标系统、ECS、输入、资源管理） |
| 阶段 2 | 2-3 周 | 渲染系统（wgpu、Sprite Batch、调试渲染） |
| 阶段 3 | 4-5 周 | 游戏逻辑（状态机、碰撞、战斗、动画、物理） |
| 阶段 4 | 1 周 | 音频系统 |
| 阶段 5 | 3-4 周 | 回滚网络（GGRS、确定性验证） |
| 阶段 6 | 4-6 周 | 游戏内容（角色、舞台、UI、游戏模式） |
| 阶段 7 | 2-3 周 | 优化与发布 |
| **总计** | **约 5-6 个月** | **可发布的 alpha 版本** |

---

## 优先级标记

- 🔴 **P0（阻塞）**：必须完成才能继续后续开发
- 🟡 **P1（重要）**：核心功能，影响可玩性
- 🟢 **P2（可选）**：锦上添花，可延后

当前所有任务默认为 **P1**，具体优先级在开发过程中调整。

---

## 进度追踪

使用 GitHub Issues / Projects 或本地 Markdown 文件追踪：
- 每完成一个任务，在前面的 `[ ]` 中打勾 `[x]`
- 每周更新进度报告
- 遇到阻塞问题时，标记 🚧 并记录原因
