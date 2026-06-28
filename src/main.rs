//! 微光夜灯固件入口（技术方案 §18 启动顺序）。
//!
//! 装配顺序：
//!  1. link_patches + 日志初始化
//!  2. 板载 LED 硬件前置提醒（仅日志）
//!  3. LED PWM 初始化（duty=0）
//!  4. I2C 初始化 + BH1750 连续模式
//!  5. 事件队列单例触发 + 渐暗定时器创建
//!  6. AM312 GPIO 中断挂载（依赖队列已就绪，ISR 内直接投递）
//!  7. 排空队列（丢弃 AM312 上电抖动产生的伪事件）
//!  8. 同步阻塞读 2 次 BH1750 建立初始 env_dark
//!  9. 构造 ControllerCtx + spawn controller_task + spawn light_sensor_task
//! 10. 主循环阻塞

use std::thread;
use std::time::Duration;

use anyhow::Result;
use esp_idf_hal::i2c::{I2cDriver, config::Config as I2cConfig};
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::units::Hertz;
use esp_idf_svc::log::EspLogger;
use esp_idf_svc::sys::link_patches;

mod config;
mod controller;
mod drivers;
mod event;
mod state;
mod tasks;
mod utils;

use crate::config::{
    BH1750_FIRST_MEASURE_DELAY_MS, BOOTSTRAP_READ_RETRY, BOOTSTRAP_SAMPLE_INTERVAL_MS,
    I2C_ADDR_BH1750, LUX_DARK_THRESHOLD,
};
use crate::drivers::bh1750_service::Bh1750;
use crate::drivers::led_pwm::LedPwm;
use crate::drivers::motion_input::MotionInput;
use crate::state::ControllerCtx;
use crate::tasks::controller_task;
use crate::tasks::light_sensor_task;

fn main() -> Result<()> {
    // 步骤 1：link_patches + 日志
    link_patches();
    EspLogger::initialize_default();
    log_boot!("app start, version={}", env!("CARGO_PKG_VERSION"));

    // 步骤 2：板载 LED 硬件前置提醒（用户已确认仅有不可控电源灯）
    log_init!("board_led: hardware-only, verify physically obscured");
    log_init!(
        "gpio: SDA=6 SCL=7 AM312=4 LED_PWM=5  I2C_ADDR=0x{:02X}",
        I2C_ADDR_BH1750
    );

    let peripherals = Peripherals::take()?;

    // 步骤 3：LED PWM —— 消费 ledc + pins.gpio5
    let led = LedPwm::new(peripherals.ledc, peripherals.pins.gpio5)?;
    log_init!("LED PWM ok, pin=GPIO5, freq=4000Hz, res=10bit");

    // 步骤 4：I2C + BH1750
    let i2c = I2cDriver::new(
        peripherals.i2c0,
        peripherals.pins.gpio6,
        peripherals.pins.gpio7,
        &I2cConfig::new().baudrate(Hertz(100_000)),
    )?;
    let mut bh = Bh1750::new(i2c);
    bh.init_continuous_mode()?;
    log_init!("BH1750 ok, mode=CONT_H_RES, addr=0x{:02X}", I2C_ADDR_BH1750);

    // 步骤 5：事件队列单例触发 + 渐暗定时器
    // ⚠️ 必须在 AM312 ISR 武装之前完成：ISR 回调直接调用 `event::queue()`,
    //    若 Lazy 尚未初始化，ISR 上下文触发 `once_cell::sync::Lazy` 初始化
    //    会调 `xQueueCreate`（非 ISR 安全）+ `std::sync::Once`（互斥锁），
    //    导致 ISR 失败/卡死，MotionDetected 永不入队。
    let _ = event::queue(); // 触发 Lazy 初始化
    let fade_timer = controller_task::create_fade_timer()?;
    log_init!("event queue + fade timer ok");

    // 步骤 6：AM312 GPIO 中断
    let _motion = MotionInput::new(peripherals.pins.gpio4)?;
    // _motion 必须保活到程序结束，否则 PinDriver drop 会 unsubscribe

    // 步骤 7：排空队列（丢弃 AM312 上电抖动伪事件）
    while event::queue().recv_front(0).is_some() {}
    log_init!("queue drained");

    // 步骤 8：同步阻塞读 2 次 BH1750 建立初始 env_dark
    thread::sleep(Duration::from_millis(BH1750_FIRST_MEASURE_DELAY_MS));
    let (env_dark, armed) = bootstrap_initial_env(&mut bh)?;
    log_init!("bootstrap: env_dark={} armed={}", env_dark, armed);

    // 步骤 9：构造 ctx + spawn 任务
    let ctx = ControllerCtx::new(env_dark, armed);
    let _controller_handle = controller_task::spawn(led, fade_timer, ctx)?;
    let _sensor_handle = light_sensor_task::spawn(bh, env_dark)?;

    // 步骤 10：主循环阻塞（ESP-IDF main 任务不能退出）
    log_boot!("running");
    loop {
        thread::sleep(Duration::from_secs(u64::MAX / 2));
    }
}

/// 同步阻塞读 2 次 BH1750 建立初始 env_dark（§7.4）。
///
/// 两次采样均 `<= LUX_DARK_THRESHOLD` 才视为暗；启动已暗 → armed=false，
/// 启动明亮 → armed=true。
fn bootstrap_initial_env(bh: &mut Bh1750) -> Result<(bool, bool)> {
    let mut last_err: Option<anyhow::Error> = None;
    for _ in 0..BOOTSTRAP_READ_RETRY {
        let lux1 = match bh.read_lux() {
            Ok(v) => v,
            Err(e) => {
                last_err = Some(anyhow::anyhow!("lux1 read fail: {}", e));
                continue;
            }
        };
        crate::log_lux!("bootstrap lux1={}", lux1);
        thread::sleep(Duration::from_millis(BOOTSTRAP_SAMPLE_INTERVAL_MS));
        let lux2 = match bh.read_lux() {
            Ok(v) => v,
            Err(e) => {
                last_err = Some(anyhow::anyhow!("lux2 read fail: {}", e));
                continue;
            }
        };
        crate::log_lux!("bootstrap lux2={}", lux2);
        let env_dark = lux1 <= LUX_DARK_THRESHOLD && lux2 <= LUX_DARK_THRESHOLD;
        let armed = !env_dark;
        return Ok((env_dark, armed));
    }
    // 全部失败：保守当作明亮
    if let Some(e) = last_err {
        log_init!("bootstrap all retries failed: {}, fallback to bright", e);
    }
    Ok((false, true))
}
