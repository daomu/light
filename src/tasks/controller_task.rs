//! 控制任务（技术方案 §6.2.B）。
//!
//! 作为事件队列的唯一消费者，无限超时 `recv_front` 阻塞等待事件，
//! 调用 `controller::handle_event` 推进状态机。

use std::thread::{self, JoinHandle};
use std::time::Duration;

use anyhow::Result;
use esp_idf_hal::delay::BLOCK;
use esp_idf_svc::timer::EspTimer;
use log::warn;

use crate::config::TASK_STACK_SIZE;
use crate::controller::handle_event;
use crate::drivers::led_pwm::LedPwm;
use crate::event::{self, Event};
use crate::log_init;
use crate::state::ControllerCtx;

/// 启动控制任务。
///
/// - `led`：已初始化的 LED PWM 驱动（按值转移所有权）
/// - `fade_timer`：已创建但未启动的 EspTimer（回调投递 `FadeTick`）
/// - `initial_ctx`：启动期建立的控制器初值
pub fn spawn(
    mut led: LedPwm,
    fade_timer: EspTimer<'static>,
    initial_ctx: ControllerCtx,
) -> Result<JoinHandle<()>> {
    let handle = thread::Builder::new()
        .stack_size(TASK_STACK_SIZE)
        .spawn(move || {
            log_init!("controller_task start");
            let mut ctx = initial_ctx;

            loop {
                match event::queue().recv_front(BLOCK) {
                    Some((evt, _hp_woken)) => {
                        handle_event(&mut ctx, &mut led, &fade_timer, evt);
                    }
                    None => {
                        // BLOCK 模式下不应返回 None，保险起见让出 CPU
                        warn!("[EVT ] recv returned None unexpectedly");
                        thread::sleep(Duration::from_millis(1));
                    }
                }
            }
        })?;
    Ok(handle)
}

/// 创建渐暗定时器（EspTimer），回调在 esp_timer 任务上下文投递 `FadeTick`。
///
/// 返回的定时器未启动，由 controller 在进入 Lighting 时 `every(5s)` 启动。
pub fn create_fade_timer() -> Result<EspTimer<'static>> {
    use esp_idf_svc::timer::EspTimerService;
    let svc = EspTimerService::new()?;
    let timer = svc.timer(move || {
        // 队列满：丢弃。Lighting 状态渐暗推进丢失一次 tick 只是延迟 5s，可接受。
        let _ = event::queue().send_back(Event::FadeTick, 0);
    })?;
    Ok(timer)
}

// Event 类型在此处仅用作类型推导提示，无需直接引用。
#[allow(unused_imports)]
use crate::event::Event as _EventHint;
