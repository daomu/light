## Context

仓库当前是标准 ESP-IDF Rust 模板：`main.rs` 仅打印 `Hello, world!`，`Cargo.toml` 带 `esp-idf-svc 0.52.1` + `embassy-time 0.5` + embassy feature。技术方案 §6/§14 已定调"原生 FreeRTOS 任务 + 事件队列"，embassy 路线需移除以免后续变更在两套并发模型之间摇摆。技术方案 §12 给出目标工程结构，但其中 `board.rs` / `app.rs` 职责未定义，与 `main.rs` / `config.rs` 存在职责重叠，本变更决定删除二者以简化结构。

## Goals / Non-Goals

**Goals:**
- 建立后续所有变更共享的目录与文件骨架。
- 把 §20 的全部固定参数集中到 `config.rs`，后续变更只读不写。
- 落地 §17 的前缀日志宏，让后续变更日志风格统一。
- 把 `Cargo.toml` 对齐到 §13.1，移除 embassy 路线。
- 写明统一错误处理基线，作为后续驱动/任务变更的参考。
- 在 OpenSpec `config.yaml` 写入项目背景，提升后续变更生成质量。

**Non-Goals:**
- 不实现任何驱动、状态机、任务逻辑——只建空壳与常量。
- 不修改 `sdkconfig.defaults`、`rust-toolchain.toml`、`.cargo/config.toml`、`build.rs`。
- 不引入 NVS / 网络 / 蓝牙 / 配置界面（§24 明确排除）。
- 不做 host-target 单测工程改造（fade_table/controller 测试见各相关变更）。
- 不处理板载 LED——用户已确认仅有不可控电源灯，纯硬件前置。

## Decisions

### D1. 删除 `board.rs` 与 `app.rs`
§12 列出二者但 §12.1 未定义职责。保留会造成"板级抽象放哪、装配放哪"的持续争议。决定：
- 装配逻辑（spawn 任务、连接队列、启动定时器）直接放 `main.rs`。
- 板级引脚/外设句柄的封装归入各 `drivers/*.rs`。
- `config.rs` 只存纯常量。

### D2. 模块骨架（最终结构）
```
src/
├── main.rs                  # 装配入口（boot-sequence 变更 7 填充）
├── config.rs                # 固定参数 const
├── event.rs                 # Event enum（变更 5 填充）
├── state.rs                 # 状态与上下文（变更 6 填充）
├── controller.rs            # 状态机（变更 6 填充）
├── tasks/
│   ├── mod.rs
│   ├── light_sensor_task.rs # 变更 3 填充
│   └── controller_task.rs   # 变更 6 填充
├── drivers/
│   ├── mod.rs
│   ├── led_pwm.rs           # 变更 2 填充
│   ├── motion_input.rs      # 变更 4 填充
│   └── bh1750_service.rs    # 变更 3 填充
└── utils/
    ├── mod.rs
    ├── fade_table.rs        # 变更 2 填充
    └── log_ext.rs           # 本变更填充
```
本变更只创建文件 + `mod.rs` 声明 + 必要的占位 `todo!()` 或空 `pub fn`，让 `cargo check` 通过。

### D3. `config.rs` 内容范围
按 §20 全部锁定为 `pub const`：
- `BH1750_SAMPLE_PERIOD_MS = 1000`
- `LUX_DARK_THRESHOLD = 6.0`
- `LUX_BRIGHT_THRESHOLD = 25.0`
- `DARK_CONFIRM_COUNT = 2`
- `BRIGHT_CONFIRM_COUNT = 1`
- `FADE_TOTAL_DURATION_MIN = 20`
- `FADE_TICK_PERIOD_MS = 5000`
- `FADE_STEPS = 240`
- `PWM_FREQ_HZ = 4000`
- `PWM_RESOLUTION_BITS = 10`
- `LED_RESISTOR_OHM = 220`（仅注释，不参与逻辑）
- `LED_RESISTOR_ALT_OHM = 150`（仅注释）
- GPIO 分配：`PIN_BH1750_SDA = GPIO6`、`PIN_BH1750_SCL = GPIO7`、`PIN_AM312_OUT = GPIO4`、`PIN_LED_PWM = GPIO5`（用 `esp_idf_hal::gpio::pins::*` 类型）
- `I2C_ADDR_BH1750 = 0x23`
- `EVENT_QUEUE_LEN = 16`

### D4. `utils/log_ext.rs` 设计
封装 §17 前缀宏，基于 `log` crate 的 `info!`/`warn!`/`error!`，前缀作为 target 或固定字符串：

```rust
macro_rules! log_boot { ($($t:tt)*) => { info!("[BOOT] {}", format_args!($($t)*)) } }
macro_rules! log_init { ($($t:tt)*) => { info!("[INIT] {}", format_args!($($t)*)) } }
macro_rules! log_lux  { ($($t:tt)*) => { info!("[LUX ] {}", format_args!($($t)*)) } }
macro_rules! log_evt  { ($($t:tt)*) => { info!("[EVT ] {}", format_args!($($t)*)) } }
macro_rules! log_lamp { ($($t:tt)*) => { info!("[LAMP] {}", format_args!($($t)*)) } }
macro_rules! log_fade { ($($t:tt)*) => { info!("[FADE] {}", format_args!($($t)*)) } }
```
后续变更按场景调用对应宏。

### D5. Cargo.toml 调整
- 移除：`esp-idf-svc` 的 `critical-section`/`embassy-time-driver`/`embassy-sync` feature；移除 `embassy-time` 依赖。
- 新增：`esp-idf-sys = { version = "0.34", features = ["binstart"] }`、`esp-idf-hal = "0.44"`、`anyhow = "1"`、`thiserror = "1"`、`once_cell = "1"`。
- 保留：`log = "0.4"`、`esp-idf-svc = "0.52.1"`（无 embassy feature）。
- 版本号在实现时按 crates.io 实际兼容版本微调，但 esp-idf-svc 保持在 0.52.x。

### D6. 统一错误处理基线
所有驱动与任务遵循：
1. **I2C / GPIO / PWM 初始化失败**：日志 `[INIT]` 记录错误，启动流程不继续（boot-sequence 变更 7 决定整体策略）。
2. **运行期单次采样失败 / I2C 读失败**：`[LUX ]` warn 一次，跳过本次采样，不更新 `env_dark`，不影响后续采样。
3. **ISR 异常**：ISR 内不做错误处理，最坏丢弃本次事件。
4. **不 panic**：除启动期致命错误外，运行期任何错误都不允许 panic 主任务。
5. **不影响其他任务**：单个任务循环出错必须 `sleep` 后重试，不能 busy-loop。

### D7. OpenSpec `config.yaml` 更新
填入 `context` 字段：技术栈（Rust + ESP-IDF std, ESP32-C3）、领域（插电式智能夜灯）、关键约定（原生 FreeRTOS、§17 日志前缀、§20 固定参数、错误处理基线）。

## Risks / Trade-offs

- **实现期已验证**：esp-idf-svc 0.52.x 没有 `queue` 模块（FreeRTOS 队列封装位于 `esp_idf_hal::task::queue::Queue<T>`）；`esp_idf_svc::timer::EspTimer` 是 ESP-IDF `esp_timer` 而非 FreeRTOS software timer。移除 embassy feature 后这些 API 仍可用。变更 5/6 已据此调整。
- **风险：版本兼容**——`esp-idf-hal`/`esp-idf-sys` 与 `esp-idf-svc 0.52.1` 的版本对齐需实测，可能要锁定到具体 patch 版本。
- **权衡：删除 `board.rs`/`app.rs`**——损失了一个潜在的可移植层，但 MVP 不考虑多板型，可移植性暂无价值。
- **权衡：不做 host 单测**——损失 fade_table/controller 的可测性，换取工程复杂度下降。未来如需可测，单独开变更改造 dual-target。
