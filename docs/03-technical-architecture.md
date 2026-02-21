# 技术架构设计

---

## 1. 整体架构概览

```
┌─────────────────────────────────────────────────────┐
│                    Game Loop (60 FPS)                │
├──────────┬──────────┬──────────┬────────────────────┤
│  Input   │ Network  │  Logic   │      Render        │
│  System  │  Sync    │  Update  │      Layer         │
└──────────┴──────────┴──────────┴────────────────────┘
```

**核心原则：逻辑与渲染分离，逻辑层全程整数运算（保证确定性）**

---

## 2. 场景树结构（Godot 4）

```
Main (Node)
├── GameManager          # 回合管理、计时器、胜负判定
├── Stage (Node2D)
│   ├── Background (ParallaxBackground)   # 视差背景
│   └── Foreground (Sprite2D)
├── Player1 (CharacterBody2D)
│   ├── AnimatedSprite2D                  # 精灵动画
│   ├── StateMachine (Node)               # 层次有限状态机
│   ├── HitboxManager (Node)              # 碰撞盒管理
│   │   ├── Hitbox (Area2D)               # 攻击框
│   │   ├── Hurtbox (Area2D)              # 受伤框
│   │   └── Pushbox (CollisionShape2D)    # 推挤框
│   ├── InputBuffer (Node)                # 输入缓冲 + 指令识别
│   └── PowerGauge (Node)                 # 气槽（整数，0-3000）
├── Player2 (CharacterBody2D)             # 同上结构
└── UI (CanvasLayer)
    ├── HealthBar_P1 / HealthBar_P2
    ├── PowerGauge_P1 / PowerGauge_P2
    └── RoundTimer
```

---

## 3. 状态机设计

### 3.1 层次有限状态机（HFSM）

```
RootState
├── GroundState（地面）
│   ├── IdleState
│   ├── WalkState（前/后）
│   ├── RunState
│   ├── CrouchState
│   ├── AttackState（普攻/特殊技/超必杀）
│   ├── BlockState（站立防御/蹲防）
│   └── HitstunState（受击硬直）
├── AirState（空中）
│   ├── JumpState（起跳/上升/下降）
│   ├── AirAttackState
│   └── AirHitstunState
└── SpecialState
    ├── KnockdownState（倒地）
    ├── WakeupState（起身）
    └── ThrowState（投技/被投）
```

### 3.2 状态转换条件

每个状态定义：
- `enter()`：进入时执行（播放动画、设置速度）
- `update(frame)`：每帧执行（推进帧计数、检测取消窗口）
- `exit()`：退出时清理
- `can_transition_to(next_state)`：转换条件检查

---

## 4. 帧数据系统（数据驱动）

所有招式属性外置为 JSON，策划可独立调整平衡性：

```json
{
  "move_id": "qcf_punch",
  "name": "波动拳",
  "input": "QCF+A",
  "startup": 6,
  "active": 4,
  "recovery": 18,
  "total_frames": 28,
  "damage": 1200,
  "stun_damage": 80,
  "gauge_gain_on_hit": 20,
  "gauge_gain_on_block": 10,
  "on_hit_advantage": 2,
  "on_block_advantage": -4,
  "cancel_windows": [
    { "start": 7, "end": 12, "into": ["super"] }
  ],
  "frames": [
    {
      "frame_index": 6,
      "hitboxes": [{ "x": 20, "y": -50, "w": 60, "h": 30 }],
      "hurtboxes": [{ "x": -20, "y": -80, "w": 40, "h": 80 }],
      "pushbox": { "x": -15, "y": -80, "w": 30, "h": 80 }
    }
  ],
  "hit_effect": "spark_medium",
  "hit_sound": "hit_medium_01",
  "block_sound": "block_01",
  "hitstun": 18,
  "blockstun": 12,
  "knockback": { "x": 300, "y": 0 }
}
```

---

## 5. 输入缓冲与指令识别

```
InputBuffer（16帧历史队列）
│
├── 每帧写入当前按键位图（u8/u16）
│   bit0: ← bit1: → bit2: ↑ bit3: ↓
│   bit4: A  bit5: B  bit6: C  bit7: D
│
└── 指令识别器（CommandRecognizer）
    ├── 遍历最近 N 帧历史
    ├── 匹配预定义指令序列（QCF/QCB/DP 等）
    ├── 宽容窗口：允许中间帧有额外输入
    └── 优先级：超必杀 > 特殊技 > 普攻
```

**指令宽容规则：**
- QCF 允许 ↓ 和 → 之间插入 ↘（斜向）
- 指令总时间窗口约 12-16 帧
- 同时按下多个攻击键时取最高优先级

---

## 6. 碰撞检测系统

```
每帧碰撞检测流程：

1. 根据当前动画帧，从帧数据表加载碰撞盒坐标
2. 将相对坐标转换为世界坐标（考虑朝向翻转）
3. Hitbox vs Hurtbox AABB 检测
   ├── P1 Hitbox ∩ P2 Hurtbox → P2 受击
   └── P2 Hitbox ∩ P1 Hurtbox → P1 受击
4. 同帧双方均命中 → 互相受击（双方硬直）
5. Pushbox 分离：两个 Pushbox 重叠时向外推开
6. 地面检测：Y 坐标 ≤ 地面高度时落地
```

**关键：所有坐标使用整数（像素×100），禁止浮点运算**

---

## 7. 气槽与连招系统

### 气槽充能来源

| 来源 | 充能量 |
|------|--------|
| 攻击命中对手 | +20 |
| 攻击被格挡 | +10 |
| 被攻击命中 | +30 |
| 格挡攻击 | +5 |
| MAX 模式激活后 | 持续消耗 -5/帧 |

### 连招伤害衰减表

| 连击数 | 伤害倍率 |
|--------|----------|
| 第 1-2 击 | 100% |
| 第 3 击 | 90% |
| 第 4 击 | 80% |
| 第 5 击 | 70% |
| 第 6 击 | 60% |
| 第 7 击起 | 50%（最低 20%） |

---

## 8. 摄像机系统

- 跟随两个角色的中点
- 水平范围：不超出舞台边界
- 缩放：当两角色距离过远时轻微缩小（可选）
- 无垂直跟随（固定高度，KOF 风格）

