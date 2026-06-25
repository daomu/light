//! 微光夜灯固定参数集中地（技术方案 §20）。
//!
//! 所有运行参数在此以 `pub const` 形式定义，其他模块只读引用。
//! GPIO 引脚号仅作文档与校验，实际 `GpioN` 实例须从 `Peripherals::take().pins.gpioN` 获取。

#![allow(dead_code)]

// ── GPIO 分配（§4.1）──────────────────────────────────────────────
/// BH1750 I2C 数据线
pub const PIN_BH1750_SDA_NUM: u8 = 6;
/// BH1750 I2C 时钟线
pub const PIN_BH1750_SCL_NUM: u8 = 7;
/// AM312 数字输出，上升沿中断
pub const PIN_AM312_OUT_NUM: u8 = 4;
/// 夜灯 LED PWM 输出
pub const PIN_LED_PWM_NUM: u8 = 5;

// ── BH1750（§8）──────────────────────────────────────────────────
/// BH1750 采样周期（毫秒）
pub const BH1750_SAMPLE_PERIOD_MS: u64 = 1000;
/// 进入暗环境的 lux 阈值
pub const LUX_DARK_THRESHOLD: f32 = 6.0;
/// 退出暗环境（变亮）的 lux 阈值
pub const LUX_BRIGHT_THRESHOLD: f32 = 25.0;
/// 进入暗环境所需的连续确认次数
pub const DARK_CONFIRM_COUNT: u32 = 2;
/// 退出暗环境所需的连续确认次数
pub const BRIGHT_CONFIRM_COUNT: u32 = 1;
/// BH1750 I2C 地址（ADDR -> GND）
pub const I2C_ADDR_BH1750: u8 = 0x23;
/// BH1750 连续高分辨率模式命令字
pub const BH1750_CMD_CONT_H_RES: u8 = 0x10;
/// BH1750 初始化后等待首次测量完成的时间（毫秒）
pub const BH1750_FIRST_MEASURE_DELAY_MS: u64 = 200;

// ── LED PWM（§10.1）──────────────────────────────────────────────
/// PWM 频率（Hz）
pub const PWM_FREQ_HZ: u32 = 4000;
/// PWM 分辨率位数
pub const PWM_RESOLUTION_BITS: u32 = 10;
/// PWM 最大占空比（10-bit）
pub const PWM_MAX_DUTY: u16 = 1023;

// ── 渐暗（§10.2 / §10.3）─────────────────────────────────────────
/// 渐暗总时长（分钟）
pub const FADE_TOTAL_DURATION_MIN: u64 = 20;
/// 渐暗亮度更新周期（毫秒）
pub const FADE_TICK_PERIOD_MS: u64 = 5000;
/// 渐暗总步数
pub const FADE_STEPS: usize = 240;
/// 渐暗 gamma 曲线指数
pub const FADE_GAMMA: f32 = 2.2;

// ── LED 限流电阻（§3.1，仅注释，不参与逻辑）─────────────────────
/// LED 默认限流电阻：220Ω
pub const LED_RESISTOR_OHM: u32 = 220;
/// LED 备选限流电阻：150Ω（亮度不足时使用）
pub const LED_RESISTOR_ALT_OHM: u32 = 150;

// ── 事件队列（§14.1）─────────────────────────────────────────────
/// 全局事件队列容量
pub const EVENT_QUEUE_LEN: usize = 16;

// ── 任务栈（§6.2，sdkconfig.defaults 已配 main=8192）─────────────
/// 子任务默认栈大小（字节）
pub const TASK_STACK_SIZE: usize = 4096;

// ── 启动期（§7.4 / §18）─────────────────────────────────────────
/// 启动期建立初始 env_dark 的采样间隔（毫秒）
pub const BOOTSTRAP_SAMPLE_INTERVAL_MS: u64 = 1000;
/// 启动期 read_lux 失败重试次数
pub const BOOTSTRAP_READ_RETRY: u32 = 3;
