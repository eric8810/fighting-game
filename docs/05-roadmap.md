# 开发路线图

---

## 阶段 1：可玩原型

**目标：** 验证核心战斗手感

- [ ] 搭建 Godot 4 项目基础结构
- [ ] 实现基础移动（站立/跑步/跳跃/蹲下）
- [ ] 实现最简状态机（5 个状态）
- [ ] 实现 Hitbox / Hurtbox / Pushbox 三类碰撞盒
- [ ] 实现 3 个普通攻击（轻/中/重）
- [ ] 实现 1 个特殊技（QCF 指令）
- [ ] 实现基础血条 HUD
- [ ] 本地双人对战可玩

---

## 阶段 2：完整战斗系统

**目标：** 实现 KOF 风格核心机制

- [ ] 帧数据系统（JSON 驱动）
- [ ] 输入缓冲 + 完整指令识别（QCF/QCB/DP/HCF/HCB）
- [ ] 气槽系统（3 股，充能/消耗）
- [ ] 超必杀（DM）实现
- [ ] MAX 模式实现
- [ ] 连招取消链（普攻 → 特殊技 → 超必杀）
- [ ] 连招伤害衰减
- [ ] 翻滚/紧急回避（带无敌帧）
- [ ] 投技系统
- [ ] 1 个完整可玩角色（含完整帧数据）

---

## 阶段 3：网络与内容扩展

**目标：** 联机对战 + 多角色

- [ ] 集成 rollback-netcode 插件
- [ ] 实现 Save State / Load State
- [ ] 验证游戏逻辑确定性（双端状态对比测试）
- [ ] 实现 2-4 个角色
- [ ] 实现 2-3 个舞台场景（视差背景）
- [ ] 音效系统（命中音效、BGM）
- [ ] UI 打磨（血条动画、气槽特效）
- [ ] Striker 系统（KOF 2000 特色，可选）

---

## 阶段 4：发布准备

**目标：** 可发布版本

- [ ] AI 对手（基于行为树或 FSM）
- [ ] 角色选择界面
- [ ] 主菜单 / 设置界面
- [ ] 本地双人 + 在线对战模式
- [ ] 匹配服务器部署（Nakama）
- [ ] Steam / Itch.io 发布配置
- [ ] 性能优化与测试

---

## 技术选型决策

| 决策项 | 选择 | 备注 |
|--------|------|------|
| 引擎 | Godot 4.4+ | MIT 免费，原生 2D |
| 主要语言 | GDScript + C# | 快速开发 + 性能优化 |
| 网络方案 | godot-rollback-netcode | GGPO 风格回滚 |
| 匹配服务 | Nakama（自托管） | 开源，可控 |
| 像素动画 | Aseprite | 行业标准工具 |
| 帧数据格式 | JSON | 策划可独立编辑 |
| 坐标单位 | 整数（像素×100） | 保证确定性 |
| 版本控制 | Git + Git LFS | 二进制资源用 LFS |

---

## 参考资源

| 资源 | 类型 | 内容 |
|------|------|------|
| GDC Vault - "The Mechanics of Fighting Games" | 视频 | SNK/Capcom 开发者讲座 |
| Infil.net - Rollback Netcode 文章 | 文章 | 回滚网络深度解析 |
| Dustloop Wiki | 数据库 | 真实格斗游戏帧数据参考 |
| "Playing to Win" by David Sirlin | 书 | 格斗游戏设计哲学 |
| github.com/dsnopek/godot-rollback-netcode | 代码 | Godot 回滚网络插件 |
| github.com/gschup/ggrs | 代码 | GGRS Rust 回滚库 |
| github.com/johanhelsing/matchbox | 代码 | WebRTC P2P 匹配库 |
| Aseprite | 工具 | 像素动画制作 |
