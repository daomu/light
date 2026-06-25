## 1. 模块骨架与 Cargo 调整

- [x] 1.1 在 `src/` 下创建 `tasks/`、`drivers/`、`utils/` 目录及对应 `mod.rs`
- [x] 1.2 创建 `config.rs`、`event.rs`、`state.rs`、`controller.rs` 占位文件（空 `pub fn` 或 `todo!()`）
- [x] 1.3 创建 `tasks/light_sensor_task.rs`、`tasks/controller_task.rs` 占位
- [x] 1.4 创建 `drivers/led_pwm.rs`、`drivers/motion_input.rs`、`drivers/bh1750_service.rs` 占位
- [x] 1.5 创建 `utils/fade_table.rs` 占位
- [x] 1.6 在 `main.rs` 顶部 `mod` 声明全部新模块，保持 `cargo check` 通过

## 2. 配置常量

- [x] 2.1 在 `config.rs` 写入 §20 全部 `pub const`（GPIO 分配、lux 阈值、采样/渐暗/PWM 参数、I2C 地址、队列长度）
- [x] 2.2 为 LED 限流电阻推荐值（220Ω/150Ω）添加注释说明（仅文档，不参与逻辑）

## 3. 日志宏

- [x] 3.1 在 `utils/log_ext.rs` 实现六个前缀宏：`log_boot!` / `log_init!` / `log_lux!` / `log_evt!` / `log_lamp!` / `log_fade!`
- [x] 3.2 在 `main.rs` 用 `log_boot!` 替换原 `Hello, world!`，验证宏可用

## 4. Cargo 依赖对齐

- [x] 4.1 移除 `Cargo.toml` 中 `esp-idf-svc` 的 `critical-section`/`embassy-time-driver`/`embassy-sync` feature
- [x] 4.2 移除 `embassy-time` 依赖
- [x] 4.3 新增 `esp-idf-sys`(binstart) / `esp-idf-hal` / `anyhow` / `thiserror` / `once_cell`
- [x] 4.4 运行 `cargo check` 确认依赖树可解（不实际烧录）

## 5. OpenSpec 元数据

- [x] 5.1 在 `openspec/config.yaml` 的 `context` 字段写入项目技术栈、领域、关键约定
- [x] 5.2 运行 `openspec validate add-project-foundation` 确认变更结构合法

## 6. 验证

- [x] 6.1 `cargo check` 通过
- [x] 6.2 `cargo build --release` 通过（不烧录）
- [x] 6.3 日志宏在仿真调用下输出正确前缀（可临时插入一行 `log_lux!("test")` 验证后移除）
