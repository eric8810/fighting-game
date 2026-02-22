# 游戏装修任务清单 (v0.2.0)

**版本：** v0.2.0
**更新日期：** 2026-02-22
**设计参考：** `docs/12-visual-design-specs.md`

---

## ✅ Phase 0：素材准备（已完成）

### 0.1 角色精灵（✅ 完成）

- [x] 从 GitHub 下载 Ryu/Ken 精灵表
  - Source: https://github.com/Andrei-Lapusteanu/2D-Fighting-Game
  - ✅ `assets/sprites/fighters/ryu.png` (600×654, 105KB)
  - ✅ `assets/sprites/fighters/ken.png` (1024×1024, 211KB)
  - ✅ `assets/sprites/fighters/fireball.png` (1161×305, 9.6KB)
- [x] 创建 RON 图集配置
  - ✅ `assets/sprites/ryu_atlas.ron`
  - ✅ `assets/sprites/ken_atlas.ron`

### 0.2 舞台背景（⏸️ 暂时跳过）

- [ ] 使用现有纯色背景

### 0.3 背景音乐（⏸️ 暂时跳过）

- [ ] 使用现有测试音乐

### 0.4 音效（✅ 完成）

- [x] 从 GitHub 下载音效
  - Source: https://github.com/harbiabderrhamane/fighting-game
  - ✅ `assets/sounds/hit_light.mp3` (9.7KB)
  - ✅ `assets/sounds/hit_medium.mp3` (6.7KB)
  - ✅ `assets/sounds/hit_heavy.mp3` (6.7KB)
  - ✅ `assets/sounds/ko.mp3` (30KB)
- [x] 集成到游戏
  - ✅ 更新音频系统支持 MP3 格式
  - ✅ 添加 KO 音效触发事件
  - ✅ 在回合结束时播放 KO 音效

---

## ✅ Phase 1：渲染系统集成（完成 - 100%）

> 将现有但未接入的精灵渲染基础设施连接到主游戏循环。

### 1.1 纹理管理器（✅ 完成）

- [x] 在 `App` 中添加 `fighter_texture: Option<Texture>`
- [x] 在 `resumed()` 中加载纹理
- [x] 纹理加载代码已验证工作

### 1.2 精灵渲染器集成（✅ 100%完成）

- [x] 修改 `QuadInstance` 添加 UV 坐标字段
  ```rust
  pub struct QuadInstance {
      pub rect: [f32; 4],
      pub color: [f32; 4],
      pub uv: [f32; 4],  // ✅ 已添加 (u, v, w, h)
  }
  ```
- [x] 修改 `QuadRenderer` 创建 textured + solid 两个pipeline
- [x] 修改 shader 支持纹理采样
- [x] 修复所有 QuadInstance 初始化错误（使用 `..Default::default()`）
- [x] 游戏编译成功并运行
- [x] 修改 `tick_and_render` 使用精灵而非方块（✅ 完成）
- [x] 根据 `Facing` 组件设置 UV 镜像（✅ 完成）

**当前状态：** ✅ 战士使用精灵纹理渲染，不再是彩色方块！

### 1.3 精灵翻转（✅ 完成）

- [x] 在渲染时根据 `Facing::LEFT` 翻转 UV.x

### 1.4 动画系统集成（⏸️ 待完成）

- [ ] 为 Fighter 实体添加 `SpriteAnimation` 组件
- [ ] 实现 `state_to_animation(StateType)` 映射
- [ ] 在状态切换时重置动画

### 1.5 图集配置格式（✅ 已定义）

- [x] 定义 RON 格式图集配置
- [x] 创建示例配置文件（ryu_atlas.ron, ken_atlas.ron）

---

## Phase 2：UI 重设计（KOF2000 风格）

> 不依赖外部素材，可立即开始。详细规格见 `docs/12-visual-design-specs.md`。

### 2.1 血条重设计

- [x] 实现颜色渐变（绿→黄→红，基于 HP 百分比）
- [x] 添加金色边框（1px，四边）
- [x] P2 血条从右向左填充（镜像）
- [x] 低血量闪烁效果（HP < 25%，6帧周期）

### 2.2 角色名称显示

- [x] 在血条上方显示角色名（P1 左对齐，P2 右对齐）
- [x] 字体 12px，白色，黑色描边

### 2.3 回合胜利标志

- [x] 血条下方添加 2 个圆点（实心=赢，空心=未赢）
- [x] 金色 `#c8a000`，直径 8px

### 2.4 能量槽重设计

- [x] 3 格设计，蓝色渐变
- [x] 满格发光效果（外扩 2px 半透明蓝色）

### 2.5 计时器重设计

- [x] 字体放大至 36px
- [x] 添加背景框（深色，金色边框）
- [x] ≤ 10 秒时变红色并闪烁

### 2.6 主菜单重设计

- [x] 背景改为深蓝黑 `#08080f`
- [x] 添加细微斜线纹理（半透明线条 quad）
- [x] 标题改为金色渐变横条 + 深色文字
- [x] 菜单项：选中时金色边框，未选中时灰色文字

### 2.7 回合开始/结束动画

- [x] 在 `MenuSystem` 中添加动画状态（`RoundIntro`）
- [x] 实现 "ROUND X" 文字放大动画（scale 插值）
- [x] 实现 "FIGHT!" 文字出现效果
- [x] 实现 "K.O." / "TIME OVER" 文字效果（已有）

### 2.8 暂停菜单重设计

- [x] 弹窗添加金色边框（已在现有代码中）
- [x] 添加 "PAUSE" 标题文字（已有）

---

## Phase 3：背景替换

> 依赖 Phase 0.2 完成。

### 3.1 背景图片渲染

- [ ] 扩展 `Stage` 系统支持图片层（当前只有纯色层）
- [ ] 在 `SpriteBatchRenderer` 中支持全屏背景精灵
- [ ] 实现视差滚动（各层不同速度）

### 3.2 舞台配置更新

- [ ] 更新 `assets/stages/dojo.ron` 引用图片背景
- [ ] 添加新舞台配置文件

---

## Phase 4：音频替换

> 依赖 Phase 0.3、0.4 完成。

### 4.1 音乐替换

- [ ] 将 `stage_theme.wav` 替换为 `stage_theme.ogg`
- [ ] 更新 `tickle_audio` 加载逻辑（优先 OGG）

### 4.2 音效扩充

- [ ] 添加格挡音效（`block.ogg`）
- [ ] 添加 KO 音效（`ko.ogg`），在比赛结束时播放
- [ ] 添加回合开始音效（`round_start.ogg`）
- [ ] 添加菜单音效（`menu_select.ogg`、`menu_confirm.ogg`）
- [ ] 在 `MenuSystem` 状态切换时触发对应音效

---

## 优先级与依赖关系

```
Phase 0 (手动) ──┬──► Phase 1 (渲染集成) ──► Phase 3 (背景替换)
                 └──► Phase 4 (音频替换)

Phase 2 (UI重设计) ← 无依赖，可立即开始
```

**建议开发顺序：**
1. **立即开始** Phase 2（UI重设计，无外部依赖）
2. 用户完成 Phase 0 后，并行推进 Phase 1、3、4

---

## 验收标准

- [ ] 游戏运行时无彩色方块，角色有精灵动画
- [ ] HUD 视觉风格接近 KOF2000
- [ ] 血条颜色随 HP 变化
- [ ] 计时器在低时间时有视觉反馈
- [ ] 主菜单有专业感，不像占位界面
- [ ] 音效和音乐有格斗游戏氛围
- [ ] 舞台背景有视差滚动效果
