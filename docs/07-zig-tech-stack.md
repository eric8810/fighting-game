# Zig 技术栈调研

---

## 1. Zig 用于游戏开发的核心优势

### 为什么 Zig 适合自研引擎？

| 特性 | 说明 |
|------|------|
| **C 互操作零成本** | `@cImport` 直接调用任何 C 库，无需手写绑定，无运行时开销 |
| **编译时计算** | `comptime` 可在编译期生成帧数据表、状态机等，零运行时成本 |
| **无隐式内存分配** | 所有分配器显式传入，内存行为完全可控，适合确定性要求 |
| **无运行时** | 无 GC、无异常、无隐藏控制流，帧时间稳定 |
| **跨平台编译** | 内置交叉编译，一台机器编译所有平台 |
| **错误处理** | `error union` 强制处理错误，无空指针崩溃 |
| **比 C++ 简单** | 无头文件、无模板地狱、无 UB 陷阱 |

---

## 2. 可用库清单

### 2.1 窗口 + 渲染

| 库 | 来源 | 说明 | 成熟度 |
|----|------|------|--------|
| **raylib-zig** | github.com/raylib-zig/raylib-zig | raylib 5.6 完整绑定，422 commits，70 贡献者 | ★★★★★ |
| **zsdl** | zig-gamedev/zsdl | SDL2 + SDL3 官方绑定，zig-gamedev 维护 | ★★★★ |
| **zopengl** | zig-gamedev/zopengl | OpenGL loader + 类型安全封装 | ★★★★ |
| **zgpu** | zig-gamedev/zgpu | WebGPU（Dawn）跨平台图形，支持 DX12/Vulkan/Metal | ★★★ |
| **zglfw** | zig-gamedev/zglfw | GLFW 绑定（窗口 + 输入） | ★★★★ |

**2D 格斗游戏推荐：raylib-zig 或 zsdl + zopengl**

---

### 2.2 音频

| 库 | 来源 | 说明 | 成熟度 |
|----|------|------|--------|
| **zaudio** | zig-gamedev/zaudio | miniaudio 封装，跨平台，零依赖 | ★★★★ |
| **miniaudio（直接 C 互操作）** | miniaud.io | 单头文件，`@cImport` 直接使用 | ★★★★★ |

---

### 2.3 数学

| 库 | 来源 | 说明 | 成熟度 |
|----|------|------|--------|
| **zmath** | zig-gamedev/zmath | SIMD 优化数学库，向量/矩阵/四元数 | ★★★★★ |
| **zalgebra** | github.com/kooparse/zalgebra | 纯 Zig 线性代数，API 更简洁 | ★★★★ |

> 格斗游戏逻辑层用整数，zmath 主要用于渲染变换矩阵

---

### 2.4 ECS（可选）

| 库 | 来源 | 说明 | 成熟度 |
|----|------|------|--------|
| **zflecs** | zig-gamedev/zflecs | flecs ECS 绑定（C 库，业界最快 ECS 之一） | ★★★★ |
| **zig-ecs** | github.com/prime31/zig-ecs | EnTT 的 Zig 重写 | ★★★ |

> 格斗游戏角色数量少（2 个），ECS 不是必须的，可选

---

### 2.5 网络（回滚）

| 库 | 来源 | 说明 | 成熟度 |
|----|------|------|--------|
| **GGPO（C 互操作）** | github.com/pond3r/ggpo | 直接 `@cImport` 调用 C++ GGPO，Zig 互操作支持 | ★★★★ |
| **自建回滚逻辑** | — | Zig 实现 GGPO 算法，工作量约 4-6 周 | — |

> ⚠️ 目前没有原生 Zig 回滚网络库（不像 Rust 有 GGRS）。需要通过 C 互操作调用 GGPO，或自建。

---

### 2.6 调试工具

| 库 | 来源 | 说明 | 成熟度 |
|----|------|------|--------|
| **zgui** | zig-gamedev/zgui | Dear ImGui 绑定，碰撞盒编辑器/帧数据查看器 | ★★★★ |
| **ztracy** | zig-gamedev/ztracy | Tracy 性能分析器集成 | ★★★★ |

---

### 2.7 其他工具

| 库 | 来源 | 说明 |
|----|------|------|
| **zpool** | zig-gamedev/zpool | 对象池，管理大量碰撞盒/特效对象 |
| **zjobs** | zig-gamedev/zjobs | 多线程 Job 队列（渲染/逻辑分线程） |
| **znoise** | zig-gamedev/znoise | FastNoiseLite 绑定（背景生成可用） |

---

## 3. 推荐技术栈组合

### 方案 A：raylib-zig（最简单，快速上手）

```
raylib-zig     → 窗口 + 渲染 + 输入 + 音频（一体化）
zmath          → 渲染变换矩阵
GGPO（C 互操作）→ 回滚网络
zgui           → 调试工具（碰撞盒可视化）
std.json       → 帧数据解析（Zig 标准库内置）
```

**优点：** raylib API 极简，上手最快，适合快速验证原型
**缺点：** raylib 渲染能力有限，大量特效时性能瓶颈

---

### 方案 B：zsdl + zopengl（最灵活）

```
zsdl           → 窗口 + 输入 + 基础渲染
zopengl        → 自定义 OpenGL 渲染管线（Sprite Batch）
zaudio         → 音频（miniaudio 封装）
zmath          → 矩阵变换
GGPO（C 互操作）→ 回滚网络
zgui           → 调试工具
std.json       → 帧数据解析
```

**优点：** 渲染完全可控，性能最优，SDL3 跨平台支持最好
**缺点：** 需要自己写 Sprite Batch 渲染器（约 1-2 周）

---

### 方案 C：zgpu（WebGPU，最现代）

```
zgpu           → WebGPU 渲染（支持 DX12/Vulkan/Metal）
zglfw          → 窗口 + 输入
zaudio         → 音频
zmath          → 矩阵变换
GGPO（C 互操作）→ 回滚网络
zgui           → 调试工具
```

**优点：** 现代图形 API，未来兼容性好，支持高级特效
**缺点：** WebGPU 学习曲线较高，2D 精灵渲染需要自建管线

---

## 4. Zig C 互操作示例

Zig 调用 C 库几乎零成本，以 miniaudio 为例：

```zig
const c = @cImport({
    @cDefine("MINIAUDIO_IMPLEMENTATION", {});
    @cInclude("miniaudio.h");
});

var engine: c.ma_engine = undefined;

pub fn init() !void {
    if (c.ma_engine_init(null, &engine) != c.MA_SUCCESS) {
        return error.AudioInitFailed;
    }
}

pub fn playSound(path: [*:0]const u8) void {
    _ = c.ma_engine_play_sound(&engine, path, null);
}
```

调用 GGPO 同理，直接 `@cImport("ggpo_public.h")` 即可。

---

## 5. Mach Engine（备选，不推荐当前使用）

- **状态：** v0.3/0.4，仍在开发中
- **定位：** 类 Bevy 的模块化 Zig 引擎
- **问题：** API 变动频繁，文档不完整，2D 支持不明确
- **建议：** 等 v1.0 稳定后再评估

---

## 6. Zig vs Rust（Bevy）对比

| 维度 | Zig + 自建 | Rust + Bevy |
|------|-----------|-------------|
| 学习曲线 | 中（比 Rust 低） | 高（所有权系统） |
| C 库互操作 | 极佳（零成本） | 良好（需要 unsafe） |
| 回滚网络 | 需 C 互操作 GGPO | 原生 GGRS（最佳） |
| 编译速度 | 快 | 慢（增量改善） |
| 生态成熟度 | 中（快速增长） | 中（快速增长） |
| ECS 支持 | zflecs（C 绑定） | Bevy 原生 ECS |
| 确定性控制 | 完全自控 | ECS 天然支持 |
| 工具链 | 内置交叉编译 | cargo + cross |

---

## 7. 总结

**Zig 做格斗游戏引擎是可行的**，核心优势是：
- C 互操作让所有成熟 C 库（SDL3、GGPO、miniaudio）直接可用
- 无 GC、无隐式分配，帧时间稳定，确定性好控制
- 比 C++ 简单，比 Rust 学习曲线低

**主要短板：**
- 没有原生回滚网络库（需 C 互操作 GGPO 或自建）
- 生态不如 Rust/Godot 成熟
- Zig 语言本身仍在演进（API 偶有变动）

**推荐组合：** `raylib-zig`（快速原型）或 `zsdl + zopengl`（生产级），配合 `zaudio + GGPO + zgui`
