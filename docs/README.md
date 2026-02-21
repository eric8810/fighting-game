# 调研文档索引

类拳皇 2000 风格 2D 横板格斗游戏 — 前期调研

| 文档 | 内容 |
|------|------|
| [01-game-design.md](./01-game-design.md) | 游戏策划设计：帧系统、碰撞盒、KOF 特色机制、角色分类、场景设计 |
| [02-engine-comparison.md](./02-engine-comparison.md) | 引擎与技术栈选型：Godot / Unity / Bevy / 自研 / 开源格斗框架对比 |
| [02-engine-comparison-extended.md](./02-engine-comparison-extended.md) | 补充引擎选项：Defold / GameMaker / Cocos2d-x / MonoGame / Phaser / Stride |
| [03-technical-architecture.md](./03-technical-architecture.md) | 技术架构设计：场景树、状态机、帧数据系统、输入缓冲、碰撞检测 |
| [04-network-architecture.md](./04-network-architecture.md) | 网络架构：回滚网络 vs 状态同步、GGPO/GGRS/godot-rollback 方案对比 |
| [05-roadmap.md](./05-roadmap.md) | 开发路线图：4 个阶段任务清单 + 技术选型决策表 + 参考资源 |
| [06-custom-engine-analysis.md](./06-custom-engine-analysis.md) | 自研引擎深度分析：好处/坏处、子系统清单、可用基础库、现实工作量评估 |
| [07-zig-tech-stack.md](./07-zig-tech-stack.md) | Zig 技术栈调研：raylib-zig / zsdl / zaudio / GGPO 互操作 / 与 Rust 对比 |
| [08-final-decision-analysis.md](./08-final-decision-analysis.md) | **最终决策分析**：Rust vs Zig vs C++，基于技术领先度和问题追踪能力的深度对比 |
| [09-engine-technical-specs.md](./09-engine-technical-specs.md) | **引擎技术规格书**：架构设计、ECS 组件、坐标系统、渲染/网络/音频系统详细规格 |
| [10-development-todo.md](./10-development-todo.md) | **开发任务清单**：分 6 个阶段，从项目初始化到可发布版本的完整任务追踪 |
| [11-high-refresh-rate-support.md](./11-high-refresh-rate-support.md) | **高刷新率支持设计**：固定 60 FPS 逻辑 + 可变渲染帧率（120/144/240 Hz）插值方案 |
