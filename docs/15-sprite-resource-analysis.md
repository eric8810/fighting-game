# 精灵资源绑定与切分逻辑分析

**创建日期:** 2026-02-22
**问题:** 人物动画和精灵图依然没有被正确切分显示

---

## 1. 当前资源概况

### 1.1 可用资源

**精灵表 (Sprite Sheets):**
- `assets/sprites/fighters/ryu.png` - 600×654 px, 105KB
- `assets/sprites/fighters/ken.png` - 1024×1024 px, 211KB
- `assets/sprites/fighters/fireball.png` - 1161×305 px, 9.6KB

**配置文件:**
- `assets/sprites/ryu_atlas.ron` - 定义了 Ryu 的动画布局
- `assets/sprites/ken_atlas.ron` - 定义了 Ken 的动画布局

**音效:**
- `assets/sounds/hit_light.mp3`
- `assets/sounds/hit_medium.mp3`
- `assets/sounds/hit_heavy.mp3`
- `assets/sounds/ko.mp3`

### 1.2 精灵表布局（Ryu）

根据 `ryu_atlas.ron`：

```
精灵表尺寸: 600×654 px
单帧尺寸: 100×116 px
列数: 600 / 100 = 6 列
行数: 654 / 116 ≈ 5.6 行 (实际使用 9 行)

动画布局（按行）:
Row 0: idle (4 帧)
Row 1: walk_forward (5 帧)
Row 2: walk_backward (5 帧)
Row 3: jump (6 帧)
Row 4: crouch (2 帧)
Row 5: attack_a (5 帧)
Row 6: attack_b (6 帧)
Row 7: hitstun (3 帧)
Row 8: knockdown (6 帧)
```

---

## 2. 当前实现分析

### 2.1 纹理加载 (main.rs:241-263)

```rust
let ryu_path = "./assets/sprites/fighters/ryu.png";
let fighter_texture = match std::fs::read(ryu_path) {
    Ok(bytes) => {
        match Texture::load_from_bytes(&ctx.device, &ctx.queue, &bytes, "ryu_texture") {
            Ok(texture) => {
                log::info!("Fighter sprites loaded successfully");
                Some(texture)
            }
            Err(e) => {
                log::warn!("Failed to create fighter texture: {}", e);
                None
            }
        }
    }
    Err(e) => {
        log::warn!("Failed to load fighter sprites: {}", e);
        None
    }
};
self.fighter_texture = fighter_texture;
self.use_sprites = self.fighter_texture.is_some();
```

**问题 1:** 只加载了 Ryu 的纹理，所有角色都使用同一张精灵表
**问题 2:** 没有加载 atlas 配置文件

### 2.2 UV 坐标计算 (main.rs:753-797)

```rust
// Sprite sheet layout: 100x116 per frame, 6 columns
const FRAME_W: f32 = 100.0;
const FRAME_H: f32 = 116.0;
const TEXTURE_W: f32 = 600.0;
const TEXTURE_H: f32 = 654.0;

// Map state to animation row
let row = match sm.current_state() {
    StateType::Idle => 0,
    StateType::WalkForward => 1,
    // ...
};

// Number of frames in this animation
let frames_in_row = match sm.current_state() {
    StateType::Idle => 4,
    StateType::WalkForward => 5,
    // ...
};

// Calculate current frame (cycles through animation)
let frame = (sm.state.state_frame / 8) as i32 % frames_in_row;

// Calculate UV coordinates (normalized 0-1)
let u0 = (frame as f32 * FRAME_W) / TEXTURE_W;
let v0 = (row as f32 * FRAME_H) / TEXTURE_H;
let uw_norm = FRAME_W / TEXTURE_W;
let vh_norm = FRAME_H / TEXTURE_H;
```

**计算公式:**
- `u0 = frame * 100 / 600 = frame * 0.1667`
- `v0 = row * 116 / 654 = row * 0.1774`
- `uw = 100 / 600 = 0.1667`
- `vh = 116 / 654 = 0.1774`

**问题 3:** 硬编码了尺寸和布局，不灵活
**问题 4:** 没有从 RON 配置文件读取
**问题 5:** 所有角色使用相同的布局（Ryu 的）

### 2.3 渲染调用 (main.rs:798-806)

```rust
qr.draw_overlay(
    &ctx.device,
    &mut encoder,
    &view,
    &ctx.queue,
    ctx.size.width as f32,
    ctx.size.height as f32,
    &fighter_instances,
    self.fighter_texture.as_ref(),  // 所有角色使用同一纹理
);
```

**问题 6:** 只有一个纹理变量，所有角色共享

---

## 3. 问题根源分析

### 3.1 为什么精灵图没有被切分？

**假设:** UV 坐标计算看起来是正确的，但可能有以下问题：

1. **帧尺寸不匹配:** 假设每帧 100×116，但实际精灵表可能不是这个布局
2. **精灵表布局错误:** RON 文件可能是为其他精灵表创建的
3. **纹理坐标系统:** UV 原点可能在错误的位置（左上 vs 左下）

### 3.2 为什么动画不工作？

**可能原因:**

1. `state_frame` 计算错误
2. 帧率太快/太慢（当前 8 帧 = 12 FPS @ 60Hz）
3. 状态映射不正确

### 3.3 验证需求

需要验证：
1. ✅ 精灵表是否真的是 600×654？
2. ❓ 精灵表的实际帧布局是什么？
3. ❓ UV 坐标系统是否正确？
4. ❓ 实际运行时 UV 值是什么？

---

## 4. 正确的资源管理架构

### 4.1 应该有的结构

```rust
// 资源管理器
struct ResourceManager {
    textures: HashMap<String, Texture>,
    atlases: HashMap<String, AtlasConfig>,
}

// 图集配置
struct AtlasConfig {
    texture_path: String,
    frame_width: u32,
    frame_height: u32,
    animations: HashMap<String, AnimationDef>,
}

// 动画定义
struct AnimationDef {
    row: u32,
    frames: u32,
    duration: u32,
    looping: bool,
}

// 角色组件
struct FighterSprite {
    atlas_name: String,  // "ryu" or "ken"
    current_animation: String,
    current_frame: u32,
}
```

### 4.2 渲染流程

```
1. 加载阶段:
   - 加载所有纹理到 HashMap
   - 解析所有 RON 配置文件

2. 每帧更新:
   - 根据状态查询当前动画名称
   - 从 atlas 配置获取 row 和 frames
   - 计算当前帧 (基于 state_frame)
   - 计算 UV 坐标 (基于 atlas 尺寸)

3. 渲染:
   - 从 atlas_name 查询纹理
   - 使用计算好的 UV 坐标
```

---

## 5. 调试步骤

### 5.1 立即可以做的

1. **打印 UV 坐标**
   ```rust
   log::info!("UV: {:?}, state: {:?}, frame: {}, row: {}",
              uv, sm.current_state(), frame, row);
   ```

2. **可视化 UV 范围**
   - 绘制第一帧：UV = [0.0, 0.0, 0.1667, 0.1774]
   - 这应该显示左上角的 1/6 x 1/5.6 区域

3. **检查精灵表实际内容**
   - 打开 ryu.png 确认布局
   - 验证帧尺寸是否真的是 100×116

### 5.2 需要验证的假设

- ❓ Ryu 精灵表是否与 ryu_atlas.ron 匹配？
- ❓ UV 原点在哪个角落？
- ❓ 实际显示的是什么？（整张图？某个区域？）

---

## 6. 推荐的修复方案

### 方案 A: 快速验证（临时）

1. 在代码中添加日志，打印 UV 坐标
2. 手动设置 UV 显示特定帧
3. 确认切分逻辑是否工作

### 方案 B: 正确实现（推荐）

1. 创建 `ResourceManager` 系统
2. 加载 atlas 配置文件
3. 根据角色动态选择纹理和配置
4. 正确计算 UV 坐标

### 方案 C: 简化版（折中）

1. 保留当前硬编码，但修复尺寸
2. 添加日志验证 UV 计算
3. 确认为什么切分不工作

---

## 7. 下一步行动

### 7.1 诊断（必须先做）

1. ✅ 打开 ryu.png 查看实际布局
2. ✅ 添加日志输出 UV 坐标
3. ✅ 截图当前游戏画面
4. ✅ 验证 `use_sprites` 是否为 true

### 7.2 修复（根据诊断结果）

**如果显示整张图:**
- UV 计算错误，需要修复公式
- 或者纹理没有正确绑定

**如果显示错误区域:**
- 帧尺寸不对
- 行列计算错误

**如果显示黑色/空白:**
- 纹理加载失败
- UV 超出范围

---

## 8. 代码位置索引

| 功能 | 文件 | 行号 |
|------|------|------|
| 纹理加载 | game/src/main.rs | 241-263 |
| UV 计算 | game/src/main.rs | 753-797 |
| 渲染调用 | game/src/main.rs | 798-806 |
| Atlas 配置 | assets/sprites/ryu_atlas.ron | 全文件 |
| QuadInstance | game/src/quad_renderer.rs | 4-23 |
| Shader | game/src/quad_renderer.rs | 61-114 |

---

## 9. 待回答的问题

1. **实际精灵表布局是什么？**
   - 需要打开图片验证
   - RON 配置是否准确？

2. **为什么现在显示不正确？**
   - UV 计算错误？
   - 纹理绑定错误？
   - Shader 错误？

3. **如何支持多个角色？**
   - P1/P2 可能是不同角色
   - 需要动态纹理选择

4. **动画切换是否平滑？**
   - 状态切换时帧数是否重置？
   - 动画速度是否合适？

---

## 10. 总结

**核心问题:**
- 代码逻辑看起来正确，但实际显示不正确
- 需要先诊断问题根源

**建议顺序:**
1. 🔍 诊断 - 添加日志，验证数据
2. 📸 截图 - 记录当前状态
3. 🖼️ 检查 - 查看实际精灵表
4. 🔧 修复 - 根据诊断结果修复
5. 📚 重构 - 实现正确的资源管理系统

**不要急于重构代码，先搞清楚为什么当前代码不工作！**
