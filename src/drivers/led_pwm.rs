//! LEDC PWM 驱动（技术方案 §10.1）。
//!
//! 使用 ESP32-C3 LEDC 低速通道：timer0 + channel0，GPIO5，4kHz，10-bit。
//! 对外提供 `new` / `set_duty` / `set_off` / `set_full`。

use anyhow::Result;
use esp_idf_hal::gpio::OutputPin;
use esp_idf_hal::ledc::{LedcDriver, LedcTimerDriver, Resolution, config::TimerConfig};
use esp_idf_hal::ledc::LEDC;
use esp_idf_hal::units::Hertz;
use thiserror::Error;

use crate::config::{PWM_FREQ_HZ, PWM_MAX_DUTY, PWM_RESOLUTION_BITS};

#[derive(Debug, Error)]
pub enum LedError {
    #[error("ledc error: {0}")]
    Esp(#[from] esp_idf_sys::EspError),
}

/// LED PWM 驱动句柄，持有 channel0 与 timer0 的所有权。
pub struct LedPwm {
    driver: LedcDriver<'static>,
}

impl LedPwm {
    /// 初始化 LEDC：timer0 (4kHz, 10-bit) + channel0 + GPIO5，初始 duty=0。
    ///
    /// 调用方需传入首次 `Peripherals::take()` 得到的 `ledc` 与 `gpio5`。
    pub fn new(ledc: LEDC, pin: impl OutputPin + 'static) -> Result<Self, LedError> {
        let timer_driver = LedcTimerDriver::new(
            ledc.timer0,
            &TimerConfig::new()
                .frequency(Hertz(PWM_FREQ_HZ))
                .resolution(match PWM_RESOLUTION_BITS {
                    10 => Resolution::Bits10,
                    _ => unimplemented!("PWM_RESOLUTION_BITS only supports 10"),
                }),
        )?;

        let driver = LedcDriver::new(ledc.channel0, timer_driver, pin)?;

        let mut led = LedPwm { driver };
        led.set_off()?;
        Ok(led)
    }

    /// 设置 duty，入参自动截断到 `0..=PWM_MAX_DUTY`。
    #[inline]
    pub fn set_duty(&mut self, duty: u16) -> Result<(), LedError> {
        let clamped = if duty > PWM_MAX_DUTY {
            PWM_MAX_DUTY
        } else {
            duty
        };
        self.driver.set_duty(clamped as u32)?;
        Ok(())
    }

    /// 等价于 `set_duty(0)`。
    #[inline]
    pub fn set_off(&mut self) -> Result<(), LedError> {
        self.driver.set_duty(0)?;
        Ok(())
    }

    /// 等价于 `set_duty(PWM_MAX_DUTY)`。
    #[inline]
    pub fn set_full(&mut self) -> Result<(), LedError> {
        self.driver.set_duty(PWM_MAX_DUTY as u32)?;
        Ok(())
    }
}
