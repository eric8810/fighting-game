# 高刷新率支持设计补充

---

## 1. 帧率架构设计

### 1.1 双层帧率系统

格斗游戏需要区分**逻辑帧率**和**渲染帧率**：

```
逻辑帧率（固定 60 FPS）
  ↓ 驱动游戏逻辑、状态机、碰撞检测
  ↓ 回滚网络依赖固定帧率
  ↓
渲染帧率（可变 60/120/144/240 FPS）
  ↓ 插值渲染，提升视觉流畅度
  ↓ 不影响游戏逻辑
```

**核心原则：**
- 游戏逻辑永远以 60 FPS 运行（格斗游戏标准）
- 渲染层以显示器刷新率运行（120/144/240 Hz）
- 渲染帧之间使用插值（Interpolation）平滑过渡

---

## 2. 实现方案

### 2.1 固定时间步长（Fixed Timestep）

```rust
pub struct GameLoop {
    logic_hz: f64,           // 60.0
    logic_dt: f64,           // 1.0 / 60.0 = 0.01666...
    accumulator: f64,        // 累积时间
    previous_time: Instant,
}

impl GameLoop {
    pub fn new() -> Self {
        Self {
            logic_hz: 60.0,
            logic_dt: 1.0 / 60.0,
            accumulator: 0.0,
            previous_time: Instant::now(),
        }
    }

    pub fn update(&mut self, world: &mut World) -> f32 {
        let current_time = Instant::now();
        let frame_time = (current_time - self.previous_time).as_secs_f64();
        self.previous_time = current_time;

        // 限制单帧最大时间（防止死亡螺旋）
        let frame_time = frame_time.min(0.25);
        self.accumulator += frame_time;

        // 固定时间步长更新逻辑
        while self.accumulator >= self.logic_dt {
            self.update_logic(world);
            self.accumulator -= self.logic_dt;
        }

        // 返回插值因子（0.0 - 1.0）
        (self.accumulator / self.logic_dt) as f32
    }

    fn update_logic(&mut self, world: &mut World) {
        // 游戏逻辑更新（状态机、碰撞、物理）
        input_system(world);
        state_machine_system(world);
        physics_system(world);
        collision_system(world);
        combat_system(world);
        animation_system(world);
    }
}
```

### 2.2 渲染插值

```rust
/// 渲染系统使用插值位置
pub fn render_system(world: &World, alpha: f32) {
    let mut query = world.query::<(&Position, &PreviousPosition, &SpriteAnimation)>();

    for (_, (pos, prev_pos, sprite)) in query.iter() {
        // 插值当前位置和上一帧位置
        let interpolated_pos = LogicVec2 {
            x: prev_pos.pos.x + ((pos.pos.x - prev_pos.pos.x) as f32 * alpha) as i32,
            y: prev_pos.pos.y + ((pos.pos.y - prev_pos.pos.y) as f32 * alpha) as i32,
        };

        renderer.draw_sprite(interpolated_pos, sprite);
    }
}

/// 每次逻辑更新后保存位置
pub fn save_previous_position_system(world: &mut World) {
    let mut query = world.query::<(&Position, &mut PreviousPosition)>();
    for (_, (pos, prev_pos)) in query.iter() {
        prev_pos.pos = pos.pos;
    }
}
```

### 2.3 新增组件

```rust
/// 上一帧位置（用于插值）
#[derive(Component, Clone, Copy)]
pub struct PreviousPosition {
    pub pos: LogicVec2,
}
```

---

## 3. VSync 与帧率控制

### 3.1 wgpu 配置

```rust
pub fn create_surface_config(
    size: winit::dpi::PhysicalSize<u32>,
    format: wgpu::TextureFormat,
    vsync: bool,
) -> wgpu::SurfaceConfiguration {
    wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format,
        width: size.width,
        height: size.height,
        // VSync 模式选择
        present_mode: if vsync {
            wgpu::PresentMode::AutoVsync  // 自动适配显示器刷新率
        } else {
            wgpu::PresentMode::AutoNoVsync // 无限帧率
        },
        alpha_mode: wgpu::CompositeAlphaMode::Auto,
        view_formats: vec![],
        desired_maximum_frame_latency: 2,
    }
}
```

### 3.2 帧率显示

```rust
pub struct FrameCounter {
    frames: u32,
    last_second: Instant,
    current_fps: u32,
}

impl FrameCounter {
    pub fn tick(&mut self) -> Option<u32> {
        self.frames += 1;
        let now = Instant::now();
        if now.duration_since(self.last_second).as_secs() >= 1 {
            self.current_fps = self.frames;
            self.frames = 0;
            self.last_second = now;
            return Some(self.current_fps);
        }
        None
    }
}
```

---

## 4. 高刷新率优化

### 4.1 输入采样

高刷新率下，输入采样更频繁，需要避免重复处理：

```rust
pub struct InputSampler {
    last_logic_frame_input: InputState,
}

impl InputSampler {
    pub fn sample(&mut self, raw_input: RawInput) -> Option<InputState> {
        let current_input = self.process_raw_input(raw_input);

        // 只有输入变化时才记录
        if current_input != self.last_logic_frame_input {
            self.last_logic_frame_input = current_input;
            return Some(current_input);
        }
        None
    }
}
```

### 4.2 渲染优化

高刷新率下，渲染调用更频繁，需要优化：

```rust
// 使用 Sprite Batch 减少 Draw Call
// 使用纹理 Atlas 减少纹理切换
// 使用实例化渲染（wgpu::RenderPass::draw_indexed_indirect）
```

---

## 5. 配置选项

### 5.1 用户可配置

```ron
// config/graphics.ron
GraphicsConfig(
    vsync: true,              // 是否启用 VSync
    target_fps: Auto,         // Auto / Fixed(60) / Fixed(120) / Fixed(144)
    resolution: (1920, 1080),
    fullscreen: false,
    interpolation: true,      // 是否启用渲染插值
)
```

### 5.2 运行时切换

```rust
pub fn toggle_vsync(&mut self, enabled: bool) {
    self.surface_config.present_mode = if enabled {
        wgpu::PresentMode::AutoVsync
    } else {
        wgpu::PresentMode::AutoNoVsync
    };
    self.surface.configure(&self.device, &self.surface_config);
}
```

---

## 6. 测试验证

### 6.1 帧率稳定性测试

```rust
#[test]
fn test_fixed_timestep() {
    let mut game_loop = GameLoop::new();
    let mut world = World::new();

    // 模拟 144 FPS 渲染帧率
    let render_dt = 1.0 / 144.0;
    let mut logic_frame_count = 0;

    for _ in 0..1440 {  // 模拟 10 秒
        std::thread::sleep(Duration::from_secs_f64(render_dt));
        let alpha = game_loop.update(&mut world);

        if alpha < 0.1 {  // 刚执行完逻辑帧
            logic_frame_count += 1;
        }
    }

    // 验证逻辑帧数接近 600（60 FPS × 10 秒）
    assert!((logic_frame_count - 600).abs() < 5);
}
```

### 6.2 插值平滑度测试

```rust
#[test]
fn test_interpolation() {
    let pos1 = LogicVec2 { x: 0, y: 0 };
    let pos2 = LogicVec2 { x: 1000, y: 0 };

    // alpha = 0.5 时应该在中点
    let interpolated = interpolate(pos1, pos2, 0.5);
    assert_eq!(interpolated.x, 500);

    // alpha = 0.0 时应该在起点
    let interpolated = interpolate(pos1, pos2, 0.0);
    assert_eq!(interpolated.x, 0);

    // alpha = 1.0 时应该在终点
    let interpolated = interpolate(pos1, pos2, 1.0);
    assert_eq!(interpolated.x, 1000);
}
```

---

## 7. 性能指标

| 刷新率 | 逻辑帧 | 渲染帧 | CPU 占用 | 延迟 |
|--------|--------|--------|----------|------|
| 60 Hz  | 60 FPS | 60 FPS | 基准 | 16.7ms |
| 120 Hz | 60 FPS | 120 FPS | +15% | 8.3ms |
| 144 Hz | 60 FPS | 144 FPS | +20% | 6.9ms |
| 240 Hz | 60 FPS | 240 FPS | +30% | 4.2ms |

**注意：** 逻辑帧永远是 60 FPS，CPU 增加主要来自渲染调用频率提升。

---

## 8. 回滚网络兼容性

高刷新率不影响回滚网络，因为：
- GGRS 只关心逻辑帧（60 FPS）
- 渲染插值是本地行为，不参与网络同步
- Save State / Load State 只保存逻辑状态，不保存插值数据

```rust
// 回滚时，插值因子重置为 0.0
pub fn rollback_to_frame(&mut self, frame: Frame, snapshot: &GameSnapshot) {
    self.world.restore(snapshot);
    self.interpolation_alpha = 0.0;  // 重置插值
}
```

---

## 9. 实施优先级

| 阶段 | 任务 | 优先级 |
|------|------|--------|
| 阶段 1 | 实现固定时间步长（60 FPS 逻辑） | P0（必须） |
| 阶段 2 | 实现 VSync 支持（自动适配刷新率） | P0（必须） |
| 阶段 3 | 实现渲染插值（PreviousPosition 组件） | P1（重要） |
| 阶段 4 | 添加帧率显示（FPS Counter） | P2（可选） |
| 阶段 5 | 用户配置选项（VSync 开关） | P2（可选） |

---

## 10. 参考资料

- [Fix Your Timestep!](https://gafferongames.com/post/fix_your_timestep/) - Glenn Fiedler 经典文章
- [Game Programming Patterns - Game Loop](https://gameprogrammingpatterns.com/game-loop.html)
- [Gaffer on Games - Networked Physics](https://gafferongames.com/post/networked_physics_2004/)
