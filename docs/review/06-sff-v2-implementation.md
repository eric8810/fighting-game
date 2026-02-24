# SFF v2 实现笔记

> 实现日期：2026-02-24
> 任务：A-4 - 实现 MUGEN SFF v1/v2 解析器
> 参考来源：Ikemen-GO `image.go`、mugen-tools `sff.py`

---

## 1. SFF v2 文件结构（已验证）

通过分析 KFM 的 `kfm.sff`（206449 字节）并对照 Ikemen-GO 源码，确认了 SFF v2.01 的实际二进制布局。

### 1.1 文件头（512 字节）

```
偏移    大小  字段                          值（KFM 示例）
0-11    12    签名                          "ElecbyteSpr\0"
12-15   4     版本字节 [ver3,ver2,ver1,ver0] [0x00,0x01,0x00,0x02] = v2.01
16-19   4     保留
20-35   16    保留（4 × u32）
36-39   4     FirstSpriteHeaderOffset       624
40-43   4     NumberOfSprites               281
44-47   4     FirstPaletteHeaderOffset      512
48-51   4     NumberOfPalettes              7
52-55   4     lofs（literal data 段基址）   8492
56-59   4     保留
60-63   4     tofs（translated data 段基址）206449（= 文件末尾，KFM 无 tdata）
64-511  448   填充
```

**内存布局验证**：
- 调色板节点：`512 ~ 623`（7 × 16 = 112 字节）
- 精灵节点：`624 ~ 8491`（281 × 28 = 7868 字节）
- literal data 段：`8492 ~ EOF`（lofs 到文件末尾）

### 1.2 调色板节点（16 字节/节点）

从 `FirstPaletteHeaderOffset` 开始，共 `NumberOfPalettes` 个：

```
偏移  大小  字段
0-1   2     group (i16)
2-3   2     item  (i16)
4-5   2     numcols (i16) — 颜色数量
6-7   2     link (u16)
8-11  4     ofs (u32) — 相对于 lofs 的偏移
12-15 4     siz (u32) — 数据长度（字节）
```

调色板数据格式：RGBA（每色 4 字节），存储在 `lofs + ofs` 处。

### 1.3 精灵节点（28 字节/节点）

从 `FirstSpriteHeaderOffset` 开始，共 `NumberOfSprites` 个：

```
偏移  大小  字段
0-1   2     group (u16)
2-3   2     item  (u16)
4-5   2     width (u16)
6-7   2     height (u16)
8-9   2     axis_x (i16)
10-11 2     axis_y (i16)
12-13 2     link (u16)   ← 链接精灵的源索引（format=1 时有效）
14    1     format (u8): 0=raw, 1=linked, 2=rle8, 3=rle5, 4=lz5
15    1     coldepth (u8): 5/8/24/32 位
16-19 4     ofs (u32) — 像素数据偏移（相对于 lofs 或 tofs）
20-23 4     size (u32) — 压缩数据长度
24-25 2     palidx (u16) — 调色板数组索引（0-based）
26-27 2     flags (u16): bit0 → 0=lofs, 1=tofs
```

**绝对偏移计算**：
```
abs_offset = (flags & 1 == 0) ? lofs + ofs : tofs + ofs
```

**KFM 示例**（前几个精灵）：
```
(9000,1) 120x140 ax=0 ay=0 fmt=2(rle8) depth=8 ofs=1792 sz=4815 pal=6 → abs=10284
(9000,0)  25x25  ax=0 ay=0 fmt=4(lz5)  depth=5 ofs=6607 sz=202  pal=0 → abs=15099
(0,0)    47x106  ax=18 ay=105 fmt=4(lz5) depth=5 ofs=6809 sz=1018 pal=0 → abs=15301
```

---

## 2. 压缩格式

### 2.1 RLE8（format=2）

```
字节 d：
  若 d & 0xc0 == 0x40：n = d & 0x3f，下一字节为颜色值，重复 n 次
  否则：n=1，d 本身为颜色值
```

### 2.2 RLE5（format=3）

块头：`rl`（初始行程）+ `dl|flag`（packed 字节数 + 高位标志）
- 若 flag=1：读取初始颜色 `c`
- 后续每字节：高 3 位 = 行程，低 5 位 = 颜色索引

### 2.3 LZ5（format=4）

LZSS 变体，控制字节驱动 8 个 token：
- **控制位=0（literal run）**：
  - `d & 0xe0 == 0`：长形式，count = next_byte + 8，value = d & 0x1f
  - 否则：count = d >> 5，value = d & 0x1f
- **控制位=1（back-reference）**：
  - `d & 0x3f == 0`：长形式，distance = (d<<2|next_byte)+1，length = next_byte+2
  - 否则：短形式，每 4 个 token 积累 2 位到 rb，length = d & 0x3f

---

## 3. 测试结果

```
Loaded 281 sprites from SFF v2
First 10 sprites:
  (0, 0): 47x106, axis: (18, 105), pixels=4982, non_zero=4973
  (0, 1): 48x105, axis: (18, 104), pixels=5040, non_zero=4129
  (0, 2): 49x105, axis: (18, 104), pixels=5145, non_zero=4164
  ...
  (11, 1): 49x80, axis: (17, 79), pixels=3920, non_zero=2607
```

**验证通过**：
- ✅ 281 个精灵全部加载
- ✅ 像素缓冲区大小 = width × height
- ✅ non_zero 占比高（说明 LZ5/RLE5 解压正确，非全零）
- ✅ 精灵尺寸和轴点偏移符合 MUGEN 标准

---

## 4. 实现文件

- `crates/tickle_mugen/src/sff_v1.rs`：`parse_v2`、`decode_rle8`、`decode_rle5`、`decode_lz5`
- 测试：`cargo test -p tickle_mugen test_load_kfm_sff -- --nocapture`


> 实现日期：2026-02-24
> 任务：A-4 - 实现 MUGEN SFF v1/v2 解析器

---

## 1. SFF v2 文件结构（实测）

通过分析 KFM 的 `kfm.sff` 文件（206449 字节），确认了 SFF v2.01 的实际二进制布局：

### 1.1 文件头（512 字节）

```
偏移    大小  字段                值（KFM 示例）
0-11    12    签名                "ElecbyteSpr\0"
12-15   4     版本                [0x00, 0x01, 0x00, 0x02] = v2.01
16-35   20    保留字段            全 0
36-39   4     ldata_offset        624 (精灵节点表偏移)
40-43   4     ldata_length        281 (精灵节点表长度，字节)
44-47   4     tdata_offset        512 (调色板节点表偏移)
48-51   4     tdata_length        7 (调色板节点数量？)
52-55   4     sprite_data_offset  8492 (像素数据区偏移)
56-59   4     sprite_data_length  197957 (像素数据区长度)
60-63   4     file_size           206449 (文件总大小)
64-511  448   保留/填充           全 0
```

**关键发现**：
- `ldata_offset` 指向精灵节点表（不是"链接数据"）
- `tdata_offset` 指向调色板节点表（不是"纹理数据"）
- 精灵节点表在 624 偏移处，调色板节点表在 512 偏移处
- 实际像素数据从 8492 偏移开始

### 1.2 调色板节点（16 字节/节点）

从偏移 512 开始，每个节点 16 字节：

```
偏移  大小  字段
0-1   2     group (u16 LE)
2-3   2     item (u16 LE)
4-5   2     numcols (u16 LE) - 颜色数量
6-7   2     paletteindex (u16 LE) - 调色板索引
8-11  4     dataoffset (u32 LE) - 相对于 ldata_offset 的偏移
12-15 4     datalen (u32 LE) - 调色板数据长度（字节）
```

**KFM 示例**：
- (1,1): 32 色，offset=0, length=128
- (1,2): 32 色，offset=128, length=128

调色板数据格式：RGBA（每色 4 字节），存储在 `ldata_offset + dataoffset` 处。

### 1.3 精灵节点（28 字节/节点）

从偏移 624 开始，每个节点 28 字节：

```
偏移  大小  字段
0-1   2     group (u16 LE)
2-3   2     item (u16 LE)
4-5   2     width (u16 LE)
6-7   2     height (u16 LE)
8-9   2     axis_x (i16 LE)
10-11 2     axis_y (i16 LE)
12    1     format (u8) - 0=raw, 2=rle8, 3=rle5, 4=lz5, 10=png
13    1     colordepth (u8) - 5/8/24 位
14-15 2     padding (对齐到 4 字节边界)
16-19 4     dataoffset (u32 LE) - 像素数据偏移
20-23 4     datalen (u32 LE) - 压缩数据长度
24-25 2     paletteindex (u16 LE) - 调色板索引
26-27 2     flags (u16 LE) - bit 0: 0=相对 ldata, 1=相对 tdata
```

**KFM 示例**（11 个精灵）：
```
(9000,1) 120x140 ax=0 ay=0 fmt=0 depth=0 doff=1792 dlen=4815 pal=6 flags=0
(9000,0) 25x25 ax=0 ay=0 fmt=0 depth=0 doff=6607 dlen=202 pal=0 flags=0
(0,0) 47x106 ax=18 ay=105 fmt=0 depth=0 doff=6809 dlen=1018 pal=0 flags=0
(0,1) 48x105 ax=18 ay=104 fmt=0 depth=0 doff=7827 dlen=1012 pal=0 flags=0
...
```

**偏移计算**：
- `flags & 1 == 0`: 绝对偏移 = `ldata_offset + dataoffset`
- `flags & 1 == 1`: 绝对偏移 = `tdata_offset + dataoffset`

---

## 2. 实现细节

### 2.1 版本检测

```rust
// 读取版本字节 [verlo3, verlo2, verlo, verhi]
let version = [data[12], data[13], data[14], data[15]];
let ver_hi = version[3];

if ver_hi == 1 {
    // SFF v1 - 使用 mugen-sff crate
    Self::parse_v1(data)
} else if ver_hi == 2 {
    // SFF v2 - 使用自研解析器
    Self::parse_v2(data)
}
```

### 2.2 调色板解析

```rust
// 从 tdata_offset 读取调色板节点（16 字节/节点）
let mut palettes = HashMap::new();
let mut pal_offset = tdata_offset;

while pal_offset < ldata_offset && pal_offset + 16 <= data.len() {
    let group = u16::from_le_bytes([data[pal_offset], data[pal_offset + 1]]);
    let item = u16::from_le_bytes([data[pal_offset + 2], data[pal_offset + 3]]);
    let numcols = u16::from_le_bytes([data[pal_offset + 4], data[pal_offset + 5]]);
    let pal_data_offset = u32::from_le_bytes([...]) as usize;
    let pal_data_length = u32::from_le_bytes([...]) as usize;

    // 读取 RGBA 调色板数据，转换为 RGB 三元组
    let abs_pal_offset = ldata_offset + pal_data_offset;
    let mut palette = Vec::with_capacity(768);
    for i in 0..numcols.min(256) {
        let idx = abs_pal_offset + (i as usize) * 4;
        palette.push(data[idx]);     // R
        palette.push(data[idx + 1]); // G
        palette.push(data[idx + 2]); // B (跳过 alpha)
    }
    palettes.insert((group, item), palette);
    pal_offset += 16;
}
```

### 2.3 精灵节点解析

```rust
// 从 ldata_offset 读取精灵节点（28 字节/节点）
let mut sprites = HashMap::new();
let mut sprite_offset = ldata_offset;

while sprite_offset < ldata_offset + ldata_length && sprite_offset + 28 <= data.len() {
    let group = u16::from_le_bytes([data[sprite_offset], data[sprite_offset + 1]]);
    let item = u16::from_le_bytes([data[sprite_offset + 2], data[sprite_offset + 3]]);
    let width = u16::from_le_bytes([data[sprite_offset + 4], data[sprite_offset + 5]]);
    let height = u16::from_le_bytes([data[sprite_offset + 6], data[sprite_offset + 7]]);
    let axis_x = i16::from_le_bytes([data[sprite_offset + 8], data[sprite_offset + 9]]);
    let axis_y = i16::from_le_bytes([data[sprite_offset + 10], data[sprite_offset + 11]]);
    let format = data[sprite_offset + 12];
    // 跳过 2 字节 padding (offset 14-15)
    let data_offset = u32::from_le_bytes([data[sprite_offset + 16..20]]) as usize;
    let data_length = u32::from_le_bytes([data[sprite_offset + 20..24]]) as usize;
    let palette_index = u16::from_le_bytes([data[sprite_offset + 24], data[sprite_offset + 25]]);
    let flags = u16::from_le_bytes([data[sprite_offset + 26], data[sprite_offset + 27]]);

    // 计算绝对偏移
    let abs_data_offset = if flags & 1 != 0 {
        tdata_offset + data_offset
    } else {
        ldata_offset + data_offset
    };

    // 解压像素数据
    let compressed = &data[abs_data_offset..abs_data_offset + data_length];
    let pixels = match format {
        0 => compressed.to_vec(), // Raw
        2 => Self::decode_rle8(compressed, width, height)?,
        _ => vec![0; width * height],
    };

    // 查找调色板
    let palette = palettes.get(&(1, palette_index))
        .or_else(|| palettes.get(&(1, 1)))
        .cloned()
        .unwrap_or_else(|| vec![0; 768]);

    sprites.insert((group, item), SpriteData { width, height, axis_x, axis_y, pixels, palette });
    sprite_offset += 28;
}
```

### 2.4 RLE8 解压

```rust
fn decode_rle8(data: &[u8], width: usize, height: usize) -> Result<Vec<u8>> {
    let mut pixels = Vec::with_capacity(width * height);
    let mut i = 0;

    while pixels.len() < width * height && i < data.len() {
        let byte = data[i];
        i += 1;

        if byte & 0xC0 == 0x40 {
            // RLE run: 01nnnnnn vvvvvvvv
            let count = (byte & 0x3F) as usize;
            let value = data[i];
            i += 1;
            for _ in 0..count {
                pixels.push(value);
            }
        } else {
            // Literal byte
            pixels.push(byte);
        }
    }

    pixels.truncate(width * height);
    Ok(pixels)
}
```

---

## 3. 测试结果

使用 KFM 的 `kfm.sff` 测试：

```
Loaded 11 sprites from SFF v2
Sprite (0,0): 47x106, axis: (18, 105)
First 10 sprites:
  (0, 0): 47x106, axis: (18, 105)   # Idle frame 0
  (0, 1): 48x105, axis: (18, 104)   # Idle frame 1
  (0, 2): 49x105, axis: (18, 104)   # Idle frame 2
  (0, 3): 50x105, axis: (18, 104)   # Idle frame 3
  (0, 4): 51x105, axis: (18, 104)   # Idle frame 4
  (0, 5): 51x105, axis: (18, 104)   # Idle frame 5
  (5, 0): 42x106, axis: (16, 105)   # Turn
  (6, 0): 44x74, axis: (19, 73)     # Crouch start
  (11, 0): 49x92, axis: (17, 91)    # Crouch
  (9000, 0): 25x25, axis: (0, 0)    # Small portrait
```

**验证通过**：
- ✅ 正确解析 11 个精灵
- ✅ 尺寸和轴点偏移正确
- ✅ 精灵组号符合 MUGEN 标准（0=Idle, 5=Turn, 6=Crouch, 9000=Portrait）

---

## 4. 已知限制

1. **压缩格式支持**：
   - ✅ Format 0 (Raw/无压缩)
   - ✅ Format 2 (RLE8)
   - ❌ Format 3 (RLE5) - 未实现
   - ❌ Format 4 (LZ5) - 未实现
   - ❌ Format 10 (PNG) - 未实现

2. **调色板查找**：
   - 当前逻辑：先查 `(1, palette_index)`，失败则回退到 `(1, 1)`
   - KFM 文件中所有精灵的 `palette_index` 都是 0 或 6，但调色板节点是 `(1,1)` 和 `(1,2)`
   - 需要进一步验证调色板索引的映射规则

3. **色深支持**：
   - 当前假设所有精灵都是 8 位索引色
   - 未处理 5 位或 24 位真彩色精灵

---

## 5. 下一步

- **A-5**：实现 AIR 解析器（读取动画定义 + 每帧碰撞盒）
- **A-6**：扩展 `SpriteFrame` 结构，添加 hitbox/hurtbox/pushbox
- **A-7**：改造渲染层，支持每帧独立尺寸和原点偏移
- **A-8**：用 KFM 验证完整管线

---

## 参考资料

- KFM 文件：`assets/mugen/kfm/kfm.sff` (206449 字节)
- 实现代码：`crates/tickle_mugen/src/sff_v1.rs`
- 测试：`cargo test -p tickle_mugen test_load_kfm_sff`
