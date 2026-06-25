## Context

变更 1 已建立骨架与 `config.rs`。BH1750 协议极简（一条命令字 + 2 字节读），自写 service 比引依赖更可控。采样任务必须维护 `env_dark` 真值（按用户确认：采样任务是唯一真值源，controller 维护镜像）。本变更的关键是双阈值滞回 + 确认计数，避免遮挡/抖动误触发。

## Goals / Non-Goals

**Goals:**
- 提供 `Bh1750::init()` + `Bh1750::read_lux()` 两个最小接口。
- 提供一个常驻 FreeRTOS 任务，持续采样并投递明/暗事件。
- 把 §15.1 的滞回伪代码精确落地，边界条件（6/25 lux）严格对齐 §5。
- 启动时可被 boot-sequence 复用同一 `Bh1750` 实例做"读 2 次建立初值"。

**Non-Goals:**
- 不做 BH1750 的其他模式（One-Time、Low-Res）——只支持连续高分辨率。
- 不做 lux 历史缓存或平滑滤波——§15.1 的确认计数已是抗抖动机制。
- 不做控制器内 lux 镜像维护——那是变更 6 的事。
- 不向 controller 推送每次 lux 值——只在状态切换时带 lux 字段。

## Decisions

### D1. BH1750 service 接口
```rust
pub struct Bh1750 { i2c: I2cDriver<'static, ...> }

impl Bh1750 {
    pub fn new(i2c: I2cDriver<'static>) -> Result<Self, BhError>;
    pub fn init_continuous_mode(&mut self) -> Result<(), BhError>;  // 发 0x10
    pub fn read_lux(&mut self) -> Result<f32, BhError>;             // 读 2 字节 -> lux
}
```
命令字 `0x10` = 连续高分辨率模式。lux 换算：`(raw << 8 | raw2) / 1.2`（标准 BH1750 datasheet 公式）。

### D2. I2C 初始化归属
I2C bus driver 由 `main.rs` 创建一次，`Bh1750::new(i2c)` 借用。本变更提供 `Bh1750`，I2C bus 的实际构造放变更 7 的 boot-sequence。本变更的 `tasks/light_sensor_task.rs` 接收一个已构造好的 `Bh1750` 实例（按值或 `&mut` 通过 channel/`Mutex` 传递——见 D4）。

### D3. 采样任务循环
```rust
pub fn spawn(bh: Bh1750, queue: EventQueue) -> JoinHandle {
    thread::Builder::new().stack_size(4096).spawn(move || {
        let mut bh = bh;
        bh.init_continuous_mode();
        let mut env_dark = false;       // 初值由 boot-sequence 通过外部设置，见 D5
        let mut dark_candidate = 0u32;
        loop {
            match bh.read_lux() {
                Ok(lux) => {
                    log_lux!("value={}", lux);
                    if !env_dark {
                        if lux <= LUX_DARK_THRESHOLD {
                            dark_candidate += 1;
                            if dark_candidate >= DARK_CONFIRM_COUNT {
                                env_dark = true;
                                queue.send(EnvDarkEntered{lux});
                            }
                        } else {
                            dark_candidate = 0;
                        }
                    } else {
                        if lux >= LUX_BRIGHT_THRESHOLD {
                            env_dark = false;
                            dark_candidate = 0;
                            queue.send(EnvBrightEntered{lux});
                        }
                    }
                }
                Err(e) => log_lux!("read fail: {}", e),  // 跳过本次，不更新 env_dark
            }
            sleep(SAMPLE_PERIOD);
        }
    })
}
```

### D4. `env_dark` 初值与启动同步
用户确认：boot-sequence 同步阻塞读 2 次建立初值。本变更的 `light_sensor_task` 在 spawn 时接收一个 `initial_env_dark: bool` 参数作为 `env_dark` 初值，避免采样任务刚 spawn 后误判一次。`dark_candidate_count` 初值恒为 0。
- `initial_env_dark == true` → 进入循环后直接进入"已暗"分支，等下一次 lux >= 25 才切回。
- `initial_env_dark == false` → 进入"未暗"分支，需要 2 次连续暗确认才发 EnvDarkEntered。

### D5. 阈值边界严格对齐
- `lux <= 6.0` → 暗候选计数 +1
- `lux >= 25.0` → 立即判亮
- `6.0 < lux < 25.0` → 不改变状态，但若处于"未暗+候选中"，候选计数 **重置为 0**（§15.1 伪代码 `lux > 6 => reset`）。

### D6. 队列投递失败处理
`queue.send` 在队列满（容量 16）时返回错误。本变更的策略：日志 warn 后丢弃本次事件。理由：环境事件延迟 1 秒（下个采样周期会重新评估）远好过阻塞采样任务。

### D7. 错误处理对齐基线
- I2C 初始化失败：`new`/`init_continuous_mode` 返回 `Err`，由 boot-sequence 决定是否中止启动（变更 7）。
- 单次 `read_lux` 失败：warn 日志 + 跳过 + 不更新 `env_dark` + 不重置 `dark_candidate`（保留前一次的候选计数，让用户遮挡短暂失败不影响判定）。

## Risks / Trade-offs

- **风险：I2C 上拉电阻**——多数 BH1750 模块自带 4.7kΩ 上拉，但若选了无上拉模块，需硬件补。本变更不处理，留给硬件验收。
- **风险：BH1750 测量周期**——连续高分辨率模式一次测量约 120ms，1s 周期足够。但若初始化后立即第一次 read 可能读到 0；本变更在 `init_continuous_mode` 后 sleep 200ms 再进循环。
- **权衡：`env_dark` 不跨任务共享原子变量**——采样任务内部 `env_dark` 是栈局部变量，状态切换通过事件传给 controller。无锁、无竞争，代价是 controller 的镜像是"事件后"的，理论上滞后一次事件传播。可接受。
- **权衡：候选计数不因 I2C 失败重置**——如 BH1750 长期故障，候选计数可能滞留，但下一次成功采样会重置。可接受。
