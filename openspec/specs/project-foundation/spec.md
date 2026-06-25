# project-foundation Specification

## Purpose
TBD - created by archiving change add-project-foundation. Update Purpose after archive.
## Requirements
### Requirement: 模块骨架
系统源码 SHALL 按 `main.rs` / `config.rs` / `event.rs` / `state.rs` / `controller.rs` / `tasks/` / `drivers/` / `utils/` 结构组织，且 MUST NOT 保留职责未定义的 `board.rs` 或 `app.rs` 文件。

#### Scenario: cargo check 通过
- **WHEN** 执行 `cargo check`
- **THEN** 编译成功，无错误，所有模块声明被正确解析

#### Scenario: 模块占位可用
- **WHEN** 后续变更引用 `config::*` / `utils::log_ext::*` 等模块
- **THEN** 符号可解析，不需修改 `mod` 声明

### Requirement: 固定参数集中地
所有 §20 列出的运行参数（采样周期、lux 阈值、确认次数、渐暗总时长与周期、渐暗步数、PWM 频率与分辨率、GPIO 分配、I2C 地址、事件队列长度）SHALL 以 `pub const` 形式集中在 `config.rs`，其他模块 MUST 只读引用，不得在本地重新定义字面量。

#### Scenario: 参数唯一来源
- **WHEN** 任意模块需要使用如开灯阈值或采样周期
- **THEN** 必须从 `config` 模块导入常量，不得在本地重新定义字面量

#### Scenario: GPIO 分配固定
- **WHEN** 后续驱动初始化
- **THEN** BH1750 SDA/SCL 必须使用 GPIO6/GPIO7，AM312 使用 GPIO4，LED PWM 使用 GPIO5

### Requirement: 前缀日志宏
`utils/log_ext.rs` SHALL 提供六个日志宏：`log_boot!` / `log_init!` / `log_lux!` / `log_evt!` / `log_lamp!` / `log_fade!`，分别输出前缀 `[BOOT]` / `[INIT]` / `[LUX ]` / `[EVT ]` / `[LAMP]` / `[FADE]`，底层 MUST 基于 `log` crate。

#### Scenario: 前缀格式正确
- **WHEN** 调用 `log_lux!("value={}", 3.8)`
- **THEN** 串口日志输出包含 `[LUX ] value=3.8` 字样

### Requirement: Cargo 依赖基线
`Cargo.toml` MUST NOT 引入 `embassy-time` 或 `esp-idf-svc` 的 `embassy-time-driver`/`embassy-sync`/`critical-section` feature；且 MUST 引入 `esp-idf-sys`(binstart) / `esp-idf-hal` / `anyhow` / `thiserror` / `once_cell`。

#### Scenario: 依赖树对齐技术方案
- **WHEN** 执行 `cargo tree`
- **THEN** 不出现 `embassy-time`/`embassy-sync` 节点；出现 `esp-idf-sys`/`esp-idf-hal`/`anyhow`/`thiserror`/`once_cell`

### Requirement: 统一错误处理基线
所有驱动与任务在运行期遇到可恢复错误（I2C 读失败、单次采样异常）时，SHALL 记录日志后跳过当次操作，且 MUST NOT panic、MUST NOT busy-loop、MUST NOT 影响其他任务。

#### Scenario: 单次采样失败不致命
- **WHEN** BH1750 单次 I2C 读取返回错误
- **THEN** 记录 warn 日志后继续下一次采样周期，`env_dark` 不被更新

#### Scenario: 运行期不 panic
- **WHEN** 任意任务循环内发生可恢复错误
- **THEN** 任务不退出，sleep 后重试

