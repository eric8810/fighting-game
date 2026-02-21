# 网络架构调研

---

## 1. 两种网络同步方案对比

### 1.1 状态同步（State Sync / Lockstep）

```
原理：
  服务器/主机收集所有玩家输入
  → 等待所有输入到齐（或超时）
  → 统一推进一帧
  → 广播新状态给所有客户端

优点：
  - 实现简单
  - 不需要游戏逻辑确定性

缺点：
  - 延迟等于最慢玩家的延迟（木桶效应）
  - 网络抖动直接导致游戏卡顿
  - 格斗游戏中 1 帧延迟都会影响手感
```

**结论：不适合格斗游戏**

---

### 1.2 回滚网络（Rollback Netcode）

```
原理：
  本地立即执行（不等待对手输入）
  → 预测对手输入（通常预测"保持上一帧输入"）
  → 收到真实输入后，若预测错误：
     1. 回滚到分歧帧的游戏状态（Save State）
     2. 用真实输入重新模拟到当前帧（Fast Forward）
     3. 渲染最新帧

优点：
  - 本地操作零延迟（手感与单机一致）
  - 网络抖动不影响游戏流畅度
  - 业界标准（街霸6、拳皇15、罪恶装备等均采用）

缺点：
  - 需要游戏逻辑完全确定性（相同输入必须产生相同结果）
  - 需要实现 Save State / Load State（快照整个游戏状态）
  - 预测错误时画面可能有轻微"闪烁"（通常不明显）
  - 实现复杂度较高
```

**结论：格斗游戏标准方案，必须采用**

---

## 2. 回滚网络实现方案

### 2.1 GGPO（经典 C++ 库）

- **作者：** Tony Cannon（街霸系列网络架构师）
- **语言：** C++
- **授权：** MIT
- **特点：** 业界最成熟的回滚网络库，商业游戏广泛使用
- **集成难度：** 中（需要 C++ 绑定）
- **适用：** 自研引擎或 C++ 项目

### 2.2 GGRS（Rust 实现）

- **语言：** Rust
- **授权：** MIT/Apache
- **特点：** GGPO 的 Rust 重写，与 Bevy ECS 完美集成
- **配套：** bevy_ggrs（Bevy 插件）、Matchbox（WebRTC P2P）
- **适用：** Bevy 项目，支持 WASM 浏览器联机

### 2.3 godot-rollback-netcode

- **语言：** GDScript + C++（底层）
- **授权：** MIT
- **特点：** 专为 Godot 设计的 GGPO 风格回滚插件
- **仓库：** `dsnopek/godot-rollback-netcode`
- **适用：** Godot 4 项目（推荐方案）

### 2.4 Backdash（C# SDK）

- **语言：** C#
- **特点：** 不绑定引擎，可用于 Unity/Godot/MonoGame
- **适用：** 已有 C# 项目，想单独添加回滚网络

---

## 3. 确定性要求

回滚网络要求游戏逻辑完全确定性：

| 要求 | 说明 |
|------|------|
| 禁止浮点运算 | 不同平台浮点结果可能不同，改用整数或定点数 |
| 禁止随机数 | 使用固定种子的伪随机（LCG），双端同步种子 |
| 禁止依赖时间 | 不使用 `Time.get_ticks_msec()`，只用帧计数 |
| 禁止异步操作 | 所有逻辑同步执行 |
| 坐标单位 | 像素 × 100（整数），避免浮点误差 |

---

## 4. Save State / Load State 设计

回滚需要保存和恢复完整游戏状态：

```
GameSnapshot（需序列化的数据）：
├── Player1State
│   ├── position: IVec2（整数坐标）
│   ├── velocity: IVec2
│   ├── current_state: StateType（枚举）
│   ├── state_frame: int
│   ├── health: int
│   ├── power_gauge: int
│   ├── facing: int（1 或 -1）
│   └── input_buffer: [u16; 16]
├── Player2State（同上）
├── GameManagerState
│   ├── round_timer: int
│   ├── round_number: int
│   └── rng_seed: int
└── 特效/音效队列（可选，通常不回滚）
```

**原则：只保存影响游戏逻辑的数据，特效/音效不需要回滚**

---

## 5. 匹配服务方案

| 方案 | 类型 | 费用 | 说明 |
|------|------|------|------|
| Nakama | 开源，可自托管 | 免费（自托管）| 功能完整，支持房间/匹配/排行榜 |
| Steam Relay | Steam 平台 | 免费（Steam 游戏）| 仅限 Steam 发布，延迟优化好 |
| Matchbox | WebRTC P2P | 免费 | 配合 GGRS，支持浏览器 |
| Photon Fusion | 商业服务 | 按用量计费 | 功能强大，有免费额度 |
| 自建 WebSocket | 自研 | 服务器成本 | 灵活，维护成本高 |

---

## 6. 网络架构推荐组合

### Godot 4 方案
```
godot-rollback-netcode（回滚逻辑）
+ WebRTC（P2P 连接，Godot 内置支持）
+ Nakama（匹配服务器，开源自托管）
```

### Bevy 方案
```
GGRS + bevy_ggrs（回滚逻辑）
+ Matchbox（WebRTC P2P，支持 WASM）
+ 自建 Matchbox Server 或 Nakama
```
