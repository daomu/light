//! 光照采样任务（技术方案 §6.2.A / §15.1）。
//!
//! 每 `config::BH1750_SAMPLE_PERIOD_MS` 读一次 BH1750，维护 `env_dark` 真值，
//! 按双阈值滞回 + 确认计数规则，在状态切换时向事件队列投递
//! `EnvDarkEntered{lux}` / `EnvBrightEntered{lux}`。

use std::thread::{self, JoinHandle};
use std::time::Duration;

use anyhow::Result;
use log::warn;

use crate::config::{
    BH1750_FIRST_MEASURE_DELAY_MS, BH1750_SAMPLE_PERIOD_MS, BRIGHT_CONFIRM_COUNT,
    DARK_CONFIRM_COUNT, LUX_BRIGHT_THRESHOLD, LUX_DARK_THRESHOLD, TASK_STACK_SIZE,
};
use crate::drivers::bh1750_service::Bh1750;
use crate::event::{self, Event};
use crate::log_init;
use crate::log_lux;

/// 启动光照采样任务。
///
/// - `bh`：已调用过 `init_continuous_mode` 的 BH1750 service（按值转移所有权）
/// - `initial_env_dark`：启动期同步建立的初值，作为采样任务的 `env_dark` 初态
///
/// 返回任务句柄。任务永不退出（除非 panic）。
pub fn spawn(mut bh: Bh1750, initial_env_dark: bool) -> Result<JoinHandle<()>> {
    let handle = thread::Builder::new()
        .stack_size(TASK_STACK_SIZE)
        .spawn(move || {
            log_init!("light_sensor_task start, initial_env_dark={}", initial_env_dark);

            // BH1750 连续模式首次测量约 120ms，启动时已 sleep 过，这里再保险等一次。
            thread::sleep(Duration::from_millis(BH1750_FIRST_MEASURE_DELAY_MS));

            let mut env_dark = initial_env_dark;
            let mut dark_candidate: u32 = 0;
            let mut bright_candidate: u32 = 0;

            loop {
                match bh.read_lux() {
                    Ok(lux) => {
                        log_lux!("value={}", lux);
                        if !env_dark {
                            // 当前明：判断是否进入暗
                            if lux <= LUX_DARK_THRESHOLD {
                                dark_candidate += 1;
                                if dark_candidate >= DARK_CONFIRM_COUNT {
                                    env_dark = true;
                                    dark_candidate = 0;
                                    bright_candidate = 0;
                                    send_event(Event::EnvDarkEntered { lux });
                                }
                            } else {
                                dark_candidate = 0;
                            }
                        } else {
                            // 当前暗：判断是否退出（变亮）
                            if lux >= LUX_BRIGHT_THRESHOLD {
                                bright_candidate += 1;
                                if bright_candidate >= BRIGHT_CONFIRM_COUNT {
                                    env_dark = false;
                                    bright_candidate = 0;
                                    dark_candidate = 0;
                                    send_event(Event::EnvBrightEntered { lux });
                                }
                            } else {
                                bright_candidate = 0;
                            }
                        }
                    }
                    Err(e) => {
                        // 遵循错误基线：warn + 跳过本次 + 不更新 env_dark + 不重置候选计数
                        warn!("[LUX ] read fail: {}", e);
                    }
                }
                thread::sleep(Duration::from_millis(BH1750_SAMPLE_PERIOD_MS));
            }
        })?;
    Ok(handle)
}

#[inline]
fn send_event(evt: Event) {
    // 0 超时（非阻塞）：队列满即丢弃。环境事件下一周期会重新评估，丢一次可接受。
    if let Err(e) = event::queue().send_back(evt, 0) {
        warn!("[EVT ] queue full, drop {:?}: {}", evt, e);
    }
}
