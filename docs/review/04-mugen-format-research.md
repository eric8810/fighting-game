# MUGEN 格式调研报告

> 调研日期：2026-02-24
> 目的：为 Tickle 引擎设计支持非规则精灵和帧级碰撞盒的架构

---

## 1. AIR 格式（动画定义）

### 1.1 文件结构

```
[Begin Action n]
; 注释用分号
group, image, x_offset, y_offset, duration [, flip] [, blend] [, scale_x, scale_y] [, angle]
```

**Action 编号约定**：
- `0-99`：基础移动（0=站立Idle, 20=前进, 40=起跳, 47=落地）
- `120-152`：防御动作
- `170-195`：特殊状态（胜利、失败、挑衅）
- `200-699`：自定义攻击（200-299站立攻击, 400-499蹲攻击, 600-699空中攻击）
- `1000-2999`：必杀技
- `3000-4999`：超必杀
- `5000-5999`：受击反应（5000-5002站立轻/中/重, 5010-5012低位, 5050空中落下, 5110-5120倒地/起身）

### 1.2 每帧参数

**必需参数（1-5）**：
1. `group` - SFF 文件中的精灵组号
2. `image` - SFF 文件中的精灵图号
3. `x_offset` - 水平偏移（像素）
4. `y_offset` - 垂直偏移（像素）
5. `duration` - 持续时长（tick，60 tick = 1秒），`-1` 表示无限/不循环

**可选参数（6-10）**：
6. `flip` - `H`（水平翻转）、`V`（垂直）、`VH`（双向）
7. `blend` - `A`（加法混合）、`S`（减法）、`A1`（50%透明）
8-9. `scale_x/y` - 缩放系数（MUGEN 1.1+）
10. `angle` - 旋转角度（MUGEN 1.1+）

**示例**：
```
[Begin Action 0]
; 站立 Idle 动画
Clsn2Default: 2
 Clsn2[0] = -10, 0, 10,-79    ; 身体受击框
 Clsn2[1] = -4,-92, 6,-79     ; 头部受击框
0,1, 0,0, 7
0,2, 0,0, 7
0,3, 0,0, 50
```

### 1.3 碰撞盒（CLSN）

**两种类型**：
- **Clsn1**（红色）：攻击框（hitbox），与对方 Clsn2 相交时触发命中
- **Clsn2**（蓝色）：受击框（hurtbox），被对方 Clsn1 击中时受伤

**语法**：
```
Clsn2Default: n
 Clsn2[0] = left, top, right, bottom
 Clsn2[1] = left, top, right, bottom
Clsn1: 1
 Clsn1[0] = left, top, right, bottom
```

- `ClsnXDefault: n` - 定义 n 个盒子，应用于该 Action 的所有帧
- 单帧可以覆盖默认值，定义自己的碰撞盒

**坐标系**：
- **X轴**：正值=前方（角色面朝方向），负值=后方
- **Y轴**：负值=向上，正值=向下（朝地面）
- **原点**：精灵轴点（axis），通常在脚底中心

**标准轴点位置**：
- 站立/行走：双脚之间，地面高度
- 蹲下：双脚之间，地面高度
- 跳跃：假设站立时脚的位置
- 空中状态：假设站立时脚的位置

### 1.4 循环控制

```
0,1, 0,0, 7
Loopstart
0,2, 0,0, 7
0,3, 0,0, 50
```

- `Loopstart` 标记循环起点
- 默认循环整个 Action
- 最后一帧 `duration = -1` 创建不循环动画

---

## 2. SFF 格式（精灵文件）

### 2.1 SFF v1 文件结构

**主文件头（512 字节）**：
```
偏移    大小  字段
0-11    12    "ElecbyteSpr\0" 签名
12-15   4     版本号（verhi, verlo, verlo2, verlo3）
16-19   4     组数量（uint32）
20-23   4     图像数量（uint32）
24-27   4     第一个子文件偏移（uint32）
28-31   4     子文件头大小（uint32）
32      1     调色板类型：1=共享，0=独立
33-511  479   注释/保留空间
```

**子文件头（每个精灵 32 字节）**：
```
偏移    大小  字段
0-3     4     下一个子文件偏移（0=最后一个）
4-7     4     子文件数据长度（链接精灵为0）
8-9     2     轴点 X 坐标（int16）
10-11   2     轴点 Y 坐标（int16）
12-13   2     组号（uint16）
14-15   2     组内图像号（uint16）
16-17   2     前一个精灵索引（用于链接精灵）
18      1     调色板标志（1=使用前一个调色板）
19-31   13    保留/注释
32+     var   PCX 图像数据 + 可选调色板（最后 768 字节）
```

**特点**：
- 链表结构存储子文件
- 精灵存储为未压缩 PCX 格式
- 每个精灵可嵌入 256 色调色板（768 字节 RGB 三元组）
- 通过标志共享调色板，复用前一个精灵的调色板
- 轴点坐标定义精灵原点相对于左上角的位置

### 2.2 SFF v2 文件结构

**主文件头（扩展）**：
```
偏移    大小  字段
0-11    12    "ElecbyteSpr\0" 签名
12-15   4     版本（v2.01 为 0x00, 0x01, 0x00, 0x0C）
16-19   4     保留/兼容性字段
20-23   4     精灵数量（uint32）
24-27   4     调色板数量（uint32）
28-31   4     ldata 偏移（链接数据段）
32-35   4     ldata 长度
36-39   4     tdata 偏移（纹理数据段）
40-43   4     tdata 长度
44-511  468   填充/保留
```

**精灵节点结构**：
```
字段            类型    说明
group           uint16  组号
item            uint16  组内图像号
width           uint16  精灵宽度（像素）
height          uint16  精灵高度（像素）
axisx           int16   X 轴偏移
axisy           int16   Y 轴偏移
format          uint8   压缩：0=raw, 2=rle8, 3=rle5, 4=lz5, 10=png
colordepth      uint8   色深（5、8 或 24 位）
dataoffset      uint32  像素数据偏移
datalen         uint32  压缩数据长度
paletteindex    uint16  调色板节点索引
flags           uint16  决定偏移计算方法
```

**调色板节点结构**：
```
字段            类型    说明
group           uint16  调色板组（1=角色调色板）
item            uint16  调色板号（角色为 1-12）
numcols         uint16  颜色数量（最多 256）
paletteindex    uint16  调色板数组索引
dataoffset      uint32  调色板数据偏移
datalen         uint32  调色板数据长度
```

**调色板数据格式**：RGB 三元组 + 填充（每色 4 字节：R, G, B, 填充/alpha）

### 2.3 压缩方法

**RLE8（8 位行程编码）**：
- 快速压缩和解压
- 低压缩率
- 适合 33-256 色图像
- 当大多数颜色索引在 0-127 和 192-255 范围时最优
- SFF v1（PCX）和 v2 均支持

**RLE5（5 位行程编码）**：
- 快速压缩和解压
- 中等压缩率
- 用于 32 色（5 位）精灵
- SFF v2 支持

**LZ5（Lempel-Ziv 变体）**：
- 压缩慢，解压快
- 良好压缩率
- 用于 32 色精灵
- SFF v2 支持

**PCX RLE（SFF v1）**：
- 标准 PCX 行程编码
- 高位设置的字节（0xC0-0xFF）表示行程计数
- 格式：若字节 >= 0xC0，低 6 位=计数，下一字节=颜色值；否则字节为字面颜色值

**Raw/None**：
- 无压缩
- 生成超大文件
- SFF v2 支持

**PNG**：
- 标准 PNG 压缩
- SFF v2 支持 24 位精灵

### 2.4 标准 Group/Image 编号约定

**移动与姿态**：
- 0: 站立 Idle
- 5: 转身
- 10: 站立到蹲下过渡
- 11: 蹲下
- 12: 蹲下到站立过渡
- 20: 前进
- 21: 后退

**跳跃**：
- 40: 起跳（地面）
- 41: 跳跃中立（上升）
- 42: 前跳（上升）
- 43: 后跳（上升）
- 44: 跳跃中立（下降）
- 45: 前跳（下降）
- 46: 后跳（下降）
- 47: 落地

**奔跑**：
- 100: 前冲
- 105: 后跳

**防御状态**：
- 120-122: 开始防御（站立/蹲下/空中）
- 130-132: 防御中（站立/蹲下/空中）
- 140-142: 停止防御（站立/蹲下/空中）
- 150-152: 防御受击反应（站立/蹲下/空中）

**必需的受击反应精灵（5000-5072 系列）**：
- 5000,0-5000,9: 站立高位受击（轴点在脚）
- 5001,0-5001,9: 同上，轴点在腰部
- 5002,0-5002,9: 同上，轴点在头部
- 5010-5012: 站立低位受击（三种轴点变体）
- 5020: 蹲下受击
- 5030-5032: 后仰倒地
- 5040-5042: 地面受击和躺地
- 5060-5062: 垂直上击
- 5070-5072: 投技动画

**特殊范围**：
- 0-199: 保留给特殊状态
- 5000-5999: 保留给必需受击精灵
- 9000,0: 大肖像（常与 0,0 一起用于调色板参考）

**自定义攻击**：通常使用 200-4999 和 6000+ 组号存放角色特有招式

### 2.5 调色板系统

**SFF v1 调色板处理**：
- **共享调色板模式（SPRPALTYPE_SHARED=1）**：
  - 第一个精灵必须嵌入调色板
  - 后续精灵可设置调色板标志复用前一个调色板
  - 引擎对 9000,0 和 0,0 精灵应用外部 ACT 调色板
  - 调色板按顺序传播到其他精灵
  - 所有 12 个角色调色板都是外部 ACT 文件（1,1 到 1,12）

- **独立调色板模式（SPRPALTYPE_INDIV=0）**：
  - 每个精灵有自己的嵌入调色板
  - 效率较低但更灵活

**SFF v2 调色板处理**：
- 调色板与位图分离
- 组 1 保留给角色调色板（1,1 到 1,12）
- 其他调色板组可使用除 1 外的任意编号
- 每个调色板最多支持 256 RGBA 颜色
- SFF v2.01 增加 32 位颜色分量和 alpha 通道
- 用户在创建文件时控制调色板分配

**ACT 文件格式**：
- Adobe Color Table 格式
- 768 字节存储 256 色（RGB 三元组）
- 颜色索引 0 通常用作透明/遮罩色
- 角色调色板在 SFF 中编号为 1,1 到 1,12

**调色板约束**：
- 所有角色精灵必须共享相同的调色板结构
- 颜色索引 0 必须唯一（用于透明）
- 8 位精灵限 256 色
- 5 位精灵限 32 色
- 24 位精灵不使用调色板（真彩色）

---

## 3. CNS 文件（状态定义）

CNS 文件通过 `anim` 参数引用 AIR 中的 Action：

```
[StateDef 200]
type = S
movetype = A
physics = S
anim = 200        ; 引用 AIR 文件中的 Action 200
```

**ChangeAnim 控制器**：
```
[State 200, ChangeAnim]
type = ChangeAnim
trigger1 = AnimTime = 0
value = 0         ; 切换到 Action 0
elem = 1          ; 可选：从第几帧开始
```

**HitDef 与碰撞盒**：
```
[State 200, HitDef]
type = HitDef
trigger1 = AnimElem = 2
attr = S, NA
damage = 30
```

当 HitDef 触发时，AIR 动画中的 Clsn1 盒子变为活跃攻击框。Clsn2 盒子始终定义角色可被击中的区域。

---

## 4. 开源解析器参考

### 4.1 Rust

**rugen** - [github.com/reu/rugen](https://github.com/reu/rugen)
- 包含 `mugen-air`、`mugen-sff`、`mugen-def`、`mugen-snd` crate
- 100% Rust 实现
- 开发中，暂无正式 release

**mugen-sff** - [crates.io/crates/mugen-sff](https://crates.io/crates/mugen-sff)
- Rust SFF 解析器 crate
- 可查阅 docs.rs 获取 API 详情

### 4.2 Python

**mugen-tools** - [github.com/bitcraft/mugen-tools](https://github.com/bitcraft/mugen-tools)
- 使用 `construct` 库进行二进制解析
- 支持 SFF v1 和 v2
- 定义完整的文件头和精灵节点结构
- 理解二进制布局的良好参考
- Public domain（parse.py 除外）

### 4.3 Java

**Mugen.-SFF-Parser** - [github.com/Yauhescha/Mugen.-SFF-Parser](https://github.com/Yauhescha/Mugen.-SFF-Parser)
- SFF v1 和 v2 独立解析器
- 包含 `compressionAlgorithm` 字段识别
- Maven 项目

### 4.4 Go

**go-sffcli** - [github.com/leonkasovan/go-sffcli](https://github.com/leonkasovan/go-sffcli)
- CLI 工具，提取精灵为 PNG，调色板为 ACT
- 支持 SFF v1（PCX）和 v2（RLE5、RLE8、LZ5、PNG）

**Ikemen-GO** - [github.com/ikemen-engine/Ikemen-GO](https://github.com/ikemen-engine/Ikemen-GO)
- 完整开源格斗游戏引擎
- 完全兼容 MUGEN 资源（包括 SFF）
- Go 编写，MIT 许可证
- 活跃开发中

### 4.5 C

**sffdecompiler** - [github.com/PopovEvgeniy/sffdecompiler](https://github.com/PopovEvgeniy/sffdecompiler)
- DOS MUGEN 图像提取工具
- 基于 Osuna Richert Christophe 的 Sffextract
- 已归档但仍可访问

### 4.6 D 语言

**AIR Parser Gist** - [gist.github.com/aliyome/2cf62e2ecf944aea7d1e](https://gist.github.com/aliyome/2cf62e2ecf944aea7d1e)
- 使用 Pegged PEG 解析器
- 数据模型：Animation 类（elements、collision defaults、loop index）
- AnimationElement 类：sprite ref、position、duration、flip、alpha、collision data

---

## 5. 对 Tickle 引擎的启示

### 5.1 需要的数据结构

```rust
// 精灵帧定义（扩展现有 SpriteFrame）
pub struct SpriteFrame {
    pub group: u16,
    pub image: u16,
    pub offset: LogicVec2,        // 原点偏移（已有）
    pub duration: u32,             // 帧时长（tick）
    pub flip: FlipFlags,
    pub hitboxes: Vec<Hitbox>,     // 攻击框（Clsn1）
    pub hurtboxes: Vec<Hurtbox>,   // 受击框（Clsn2）
    pub pushbox: Pushbox,          // 推挤框
}

// 动画定义
pub struct Animation {
    pub action_number: u32,
    pub frames: Vec<SpriteFrame>,
    pub looping: bool,
    pub loop_start: usize,
}
```

### 5.2 渲染层改动

当前固定 `FIGHTER_W × FIGHTER_H` 矩形 → 改为每帧独立尺寸：

```rust
// 当前（错误）
rect: [screen_x, screen_y, FIGHTER_W, FIGHTER_H]

// 应该（正确）
let frame = animation.frames[current_frame];
let sprite_size = sff.get_sprite_size(frame.group, frame.image);
let render_pos = screen_pos + frame.offset.to_render();
rect: [render_pos.x, render_pos.y, sprite_size.w, sprite_size.h]
```

### 5.3 碰撞盒同步

状态机切帧时，同步更新 `HitboxManager`：

```rust
impl StateMachine {
    pub fn update(&mut self, hitbox_mgr: &mut HitboxManager) {
        self.state.advance_frame();

        // 获取当前帧的碰撞盒数据
        if let Some(frame) = self.current_animation.frames.get(self.state_frame) {
            hitbox_mgr.hitboxes = frame.hitboxes.clone();
            hitbox_mgr.hurtboxes = frame.hurtboxes.clone();
            hitbox_mgr.pushbox = frame.pushbox;
        }

        // ... 自动转换逻辑
    }
}
```

---

## 6. 实现路线图

1. **A-4**：实现 SFF v1/v2 解析器（`tickle_mugen` crate）
   - 读取精灵图像数据
   - 提取每帧尺寸和轴点偏移
   - 转换为 wgpu 纹理

2. **A-5**：实现 AIR 解析器
   - 解析 Action 定义
   - 解析每帧参数（group/image/offset/duration）
   - 解析 Clsn1/Clsn2 碰撞盒

3. **A-6**：扩展 `SpriteFrame` 结构
   - 添加 `hitboxes`、`hurtboxes`、`pushbox` 字段
   - 状态机 `update()` 时同步更新 `HitboxManager`

4. **A-7**：改造渲染层
   - 支持每帧独立尺寸
   - 应用原点偏移
   - 删除固定 `FIGHTER_W/H` 假设

5. **A-8**：用 KFM 验证完整管线
   - 加载 KFM 的 SFF + AIR 文件
   - 播放站立、行走、攻击、受击动画
   - 验证碰撞盒正确显示和工作

---

## 参考资料

**官方文档**：
- [The AIR format and standard - elecbyte.com](https://www.elecbyte.com/mugendocs-11b1/air.html)
- [The sprite standard - elecbyte.com](https://www.elecbyte.com/mugendocs-11b1/spr.html)
- [SpriteMaker Tool Documentation - elecbyte.com](https://www.elecbyte.com/mugendocs-11b1/sprmake2.html)
- [The CNS format - elecbyte.com](https://elecbyte.com/mugendocs/cns.html)
- [State Controller Reference - elecbyte.com](https://www.elecbyte.com/mugendocs-11b1/sctrls.html)
- [M.U.G.E.N Tutorial 1 - elecbyte.com](https://www.elecbyte.com/mugendocs-11b1/tutorial1.html)
- [M.U.G.E.N Tutorial 2 - elecbyte.com](https://www.elecbyte.com/mugendocs-11b1/tutorial2.html)
- [M.U.G.E.N Tutorial 3 - elecbyte.com](https://www.elecbyte.com/mugendocs-11b1/tutorial3.html)

**社区文档**：
- [M.U.G.E.N Documentation:AIR Format - mugen-net](https://www.mugen-net.work/wiki/index.php/M.U.G.E.N_Documentation:AIR_Format)
- [M.U.G.E.N Documentation:The Sprite Standard - mugen-net](https://www.mugen-net.work/wiki/index.php/M.U.G.E.N_Documentation:The_Sprite_Standard)
- [Sprites Documentation - virtualltek.com](https://virtualltek.com/community/m-docs-tutorials/sprites/)

**开源实现**：
- [GitHub - reu/rugen (Rust)](https://github.com/reu/rugen)
- [GitHub - bitcraft/mugen-tools (Python)](https://github.com/bitcraft/mugen-tools)
- [GitHub - Yauhescha/Mugen.-SFF-Parser (Java)](https://github.com/Yauhescha/Mugen.-SFF-Parser)
- [GitHub - leonkasovan/go-sffcli (Go)](https://github.com/leonkasovan/go-sffcli)
- [GitHub - ikemen-engine/Ikemen-GO (Go)](https://github.com/ikemen-engine/Ikemen-GO)
- [MUGEN air parser (D language)](https://gist.github.com/aliyome/2cf62e2ecf944aea7d1e)
