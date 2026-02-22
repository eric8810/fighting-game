# Tickle Fighting Engine

一个使用 Rust + wgpu 开发的 2D 格斗游戏引擎，灵感来自拳皇 2000。

## 快速开始

### 运行主 Demo

```bash
cargo run
```

**控制：**
- **Player 1 (蓝色)**: WASD 移动 + Space 攻击
- **Player 2 (红色)**: 方向键移动 + Enter 攻击

### 运行示例

```bash
# 基础渲染示例（清屏）
cargo run --example clear_screen -p tickle_render

# Sprite Batch 示例（144 个动画精灵）
cargo run --example sprite_batch -p tickle_render
```

### 运行测试

```bash
# 运行所有测试（81 个测试）
cargo test --workspace

# 运行特定 crate 的测试
cargo test -p tickle_core
cargo test -p tickle_assets
```

### 代码质量检查

```bash
# Clippy 检查
cargo clippy --workspace

# 代码格式化
cargo fmt --all

# 构建所有示例
cargo build --workspace --examples
```

## 项目结构

```
tickle/
├── crates/
│   ├── tickle_core/      # 核心逻辑（数学、ECS、输入、系统）
│   ├── tickle_render/    # 渲染层（wgpu、sprite batch、debug）
│   ├── tickle_audio/     # 音频层（待实现）
│   ├── tickle_network/   # 网络层（待实现）
│   └── tickle_assets/    # 资源管理（RON 加载）
├── game/                 # 游戏主程序
├── assets/               # 游戏资源（精灵、数据）
└── docs/                 # 技术文档
```

## 技术特性

- ✅ **固定 60 FPS 逻辑** + 可变渲染帧率（60/120/144/240 Hz）
- ✅ **渲染插值** - 流畅的高刷新率体验
- ✅ **确定性坐标系统** - 整数定点数，为回滚网络做准备
- ✅ **ECS 架构** - 使用 hecs，易于扩展
- ✅ **现代图形 API** - wgpu（WebGPU 标准）
- ✅ **完整测试覆盖** - 81 个单元测试
- ✅ **零警告构建** - clippy + rustfmt 全部通过

## 已实现系统

### 核心系统
- 数学库（LogicVec2, LogicRect）
- ECS 组件（Position, Velocity, Health, PowerGauge 等）
- 输入系统（指令识别：QCF, QCB, DP, HCF, HCB, Dash）
- 游戏主循环（Fixed Timestep + 渲染插值）

### 渲染系统
- RenderContext（wgpu 初始化）
- SpriteBatchRenderer（实例化渲染，最多 4096 sprites）
- Texture & TextureAtlas（PNG 加载 + sprite sheet）
- DebugRenderer（碰撞盒可视化）

### 游戏逻辑
- 物理系统（重力、地面检测、摩擦力）
- 碰撞检测（Hitbox vs Hurtbox AABB）
- 战斗系统（伤害、连招衰减、气槽）
- 动画系统（帧推进、循环播放）

### 资源管理
- AssetManager（泛型资源管理）
- RON 文件加载（角色数据、招式数据）
- Arc 缓存（自动去重）

## 开发状态

**当前版本：** v0.1.0-alpha

**完成度：**
- ✅ 阶段 0：项目初始化（100%）
- ✅ 阶段 1：核心基础设施（100%）
- ✅ 阶段 2：渲染系统（100%）
- ✅ 阶段 3：游戏逻辑核心（80%）
- ⚠️ 阶段 4：音频系统（0%）
- ⚠️ 阶段 5：回滚网络（0%）

## 文档

- [SUMMARY.md](./SUMMARY.md) - 开发完成总结
- [PROGRESS.md](./PROGRESS.md) - 详细进度报告
- [docs/README.md](./docs/README.md) - 文档索引
- [docs/09-engine-technical-specs.md](./docs/09-engine-technical-specs.md) - 技术规格书
- [docs/10-development-todo.md](./docs/10-development-todo.md) - 开发任务清单

## 系统要求

- **Rust**: 1.93.1 或更高版本
- **操作系统**: Windows 10/11, Linux, macOS
- **图形 API**: 支持 Vulkan, DirectX 12, 或 Metal

## 许可证

MIT License

## 致谢

本项目使用 Agent Teams 协作开发模式完成，感谢所有团队成员的出色工作。
