//! 状态机逻辑（技术方案 §7.3 / §15.2 / §15.3）。
//!
//! `handle_event` 是核心入口，接收事件后按当前状态与标志决定动作。
//! 动作包括：开关 LED、启停渐暗定时器、更新状态与标志。

use esp_idf_svc::timer::EspTimer;

use crate::config::FADE_STEPS;
use crate::drivers::led_pwm::LedPwm;
use crate::event::Event;
use crate::log_fade;
use crate::log_lamp;
use crate::state::{ControllerCtx, State};
use crate::utils::fade_table;

/// 处理一个事件，按状态机规则执行动作并更新上下文。
///
/// - `ctx`：控制器上下文
/// - `led`：LED PWM 驱动
/// - `fade_timer`：渐暗定时器（EspTimer，已创建但未启动）
pub fn handle_event(
    ctx: &mut ControllerCtx,
    led: &mut LedPwm,
    fade_timer: &EspTimer<'static>,
    evt: Event,
) {
    match ctx.state {
        State::Idle => handle_idle(ctx, led, fade_timer, evt),
        State::Lighting => handle_lighting(ctx, led, fade_timer, evt),
    }
}

fn handle_idle(
    ctx: &mut ControllerCtx,
    led: &mut LedPwm,
    fade_timer: &EspTimer<'static>,
    evt: Event,
) {
    match evt {
        Event::EnvDarkEntered { lux } => {
            // 镜像同步：无论是否开灯，都更新 env_dark
            ctx.env_dark = true;

            if ctx.auto_dark_trigger_armed {
                // 由亮变暗自动开灯
                start_lighting(ctx, led, fade_timer, "dark_edge", lux);
                ctx.auto_dark_trigger_armed = false;
            }
            // disarmed：忽略，不重复开灯
        }
        Event::EnvBrightEntered { lux } => {
            ctx.env_dark = false;
            ctx.auto_dark_trigger_armed = true;
            crate::log_evt!("env_bright_entered lux={}", lux);
        }
        Event::MotionDetected => {
            if ctx.env_dark {
                // 黑暗中人体触发开灯，不改变 armed（§15.3）
                start_lighting(ctx, led, fade_timer, "motion", 0.0);
            }
            // 明亮中忽略
        }
        Event::FadeTick => {
            // Idle 状态不应收到 FadeTick，但若定时器有残留 tick 则忽略
        }
    }
}

fn handle_lighting(
    ctx: &mut ControllerCtx,
    led: &mut LedPwm,
    fade_timer: &EspTimer<'static>,
    evt: Event,
) {
    match evt {
        Event::FadeTick => {
            let step = (ctx.fade_step as usize) + 1;
            if step >= FADE_STEPS {
                // 渐暗结束：熄灯
                let _ = led.set_off();
                let _ = fade_timer.cancel();
                ctx.fade_step = 0;
                ctx.state = State::Idle;
                log_lamp!("off reason=fade_done");
            } else {
                ctx.fade_step = step as u16;
                let duty = fade_table::at(step);
                let _ = led.set_duty(duty);
                log_fade!("step={} duty={}", step, duty);
            }
        }
        Event::EnvBrightEntered { lux } => {
            // 环境变亮立即关灯
            let _ = led.set_off();
            let _ = fade_timer.cancel();
            ctx.env_dark = false;
            ctx.auto_dark_trigger_armed = true;
            ctx.fade_step = 0;
            ctx.state = State::Idle;
            log_lamp!("off reason=env_bright lux={}", lux);
        }
        Event::MotionDetected => {
            // §7.3.B 事件 3：完全忽略
        }
        Event::EnvDarkEntered { lux } => {
            // 已在 Lighting，镜像同步 env_dark，其余忽略
            ctx.env_dark = true;
            let _ = lux;
        }
    }
}

/// 进入 Lighting 状态：满亮 + 启动渐暗定时器 + 切状态。
fn start_lighting(
    ctx: &mut ControllerCtx,
    led: &mut LedPwm,
    fade_timer: &EspTimer<'static>,
    reason: &str,
    lux: f32,
) {
    // 先 cancel 残留定时器（若从 Lighting 异常回到 Lighting 不应发生，保险）
    let _ = fade_timer.cancel();
    // 满亮：fade_table::at(0) == 1023
    let duty = fade_table::at(0);
    let _ = led.set_duty(duty);
    // 启动周期 5s 的 FadeTick
    let _ = fade_timer.every(std::time::Duration::from_millis(
        crate::config::FADE_TICK_PERIOD_MS,
    ));
    ctx.fade_step = 0;
    ctx.state = State::Lighting;
    if reason == "motion" {
        log_lamp!("on reason=motion");
    } else {
        log_lamp!("on reason={} lux={}", reason, lux);
    }
    log_fade!("step=0 duty={}", duty);
}
