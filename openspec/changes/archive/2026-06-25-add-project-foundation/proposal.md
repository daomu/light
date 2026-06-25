## Why

当前仓库是裸的 ESP-IDF Rust 模板（仅 `main.rs` hello world），且 `Cargo.toml` 引入了 `embassy-time`/`embassy-sync` feature，与技术方案 §6/§14 明确的"原生 FreeRTOS 任务 + 事件队列"路线冲突。在动工任何驱动或状态机之前，必须先落定工程骨架、依赖基线、配置常量与日志格式，否则后续变更会各自发挥、风格分裂、难以验收。

## What Changes

- **新增模块骨架**：按技术方案 §12 创建 `config.rs` / `event.rs` / `state.rs` / `controller.rs` / `tasks/` / `drivers/` / `utils/` 目录与占位文件。**删除** §12 列出但职责未定义的 `board.rs` 与 `app.rs`，装配逻辑直接放 `main.rs`。
- **新增 `config.rs`**：集中固定参数（GPIO 分配、lux 阈值、采样周期、渐暗总时长、PWM 频率、渐暗步数等，按 §20 全部锁定为 const 常量）。
- **新增 `utils/log_ext.rs`**：封装 §17 定义的前缀日志宏（`[BOOT]` / `[INIT]` / `[LUX]` / `[EVT]` / `[LAMP]` / `[FADE]`）。
- **BREAKING** 调整 `Cargo.toml`：移除 `esp-idf-svc` 的 `embassy-time-driver`/`embassy-sync`/`critical-section` feature 与 `embassy-time` 依赖；新增 `esp-idf-sys`(binstart) / `esp-idf-hal` / `anyhow` / `thiserror` / `once_cell`（按 §13.1）。
- **确立错误处理基线**：在 `design.md` 写明统一姿态——日志记录 + 跳过当次操作 + 不 panic + 不影响其他任务。后续所有驱动/任务变更沿用此基线。
- **更新 OpenSpec `config.yaml` 的 `context`**：写入项目技术栈、约定与领域背景，供后续变更 AI 上下文使用。

## Capabilities

### New Capabilities
- `project-foundation`: 工程模块结构、固定参数集中地、前缀日志宏、Cargo 依赖基线、统一错误处理姿态。

### Modified Capabilities
<!-- 无既有 capability，本变更为首次引入 -->

## Impact

- **代码**：`src/main.rs` 改为装配入口占位；新增多个空/半空模块文件；`Cargo.toml` 与 `Cargo.lock` 同步更新。
- **依赖**：移除 embassy 相关；新增 `esp-idf-sys`/`esp-idf-hal`/`anyhow`/`thiserror`/`once_cell`。
- **构建**：`cargo build` 仍以 `riscv32imc-esp-espidf` 为目标，本变更后应仍能编译通过（hello world 行为不变）。
- **后续变更**：变更 2-7 全部依赖本变更建立的骨架、常量与日志宏。
