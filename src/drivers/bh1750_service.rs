//! BH1750 极薄 I2C service（技术方案 §8.1 / §13.2）。
//!
//! 仅封装：初始化连续高分辨率模式（命令 0x10）与读取 2 字节 lux。
//! 不引外部 crate，不缓存值，不做滤波。

use anyhow::Result;
use esp_idf_hal::delay::TickType;
use esp_idf_hal::i2c::I2cDriver;
use thiserror::Error;

use crate::config::{BH1750_CMD_CONT_H_RES, I2C_ADDR_BH1750};

#[derive(Debug, Error)]
pub enum BhError {
    #[error("i2c error: {0}")]
    Esp(#[from] esp_idf_sys::EspError),
    #[error("invalid sample bytes: got {0} (expect 2)")]
    InvalidSampleLen(usize),
}

/// BH1750 service 句柄，持有 I2C 驱动的独占所有权。
pub struct Bh1750 {
    i2c: I2cDriver<'static>,
}

impl Bh1750 {
    /// 以已构造好的 I2C master 驱动创建 BH1750 service。不发送任何命令。
    pub fn new(i2c: I2cDriver<'static>) -> Self {
        Self { i2c }
    }

    /// 发送连续高分辨率模式命令（0x10）。调用方需在之后等待约 120ms 让首次测量完成。
    pub fn init_continuous_mode(&mut self) -> Result<(), BhError> {
        self.i2c
            .write(I2C_ADDR_BH1750, &[BH1750_CMD_CONT_H_RES], Self::i2c_timeout())?;
        Ok(())
    }

    /// 读取一次 lux 值。读 2 字节，换算 `lux = (h<<8 | l) / 1.2`。
    pub fn read_lux(&mut self) -> Result<f32, BhError> {
        let mut buf = [0u8; 2];
        self.i2c.read(I2C_ADDR_BH1750, &mut buf, Self::i2c_timeout())?;
        let raw = ((buf[0] as u16) << 8) | (buf[1] as u16);
        Ok(raw as f32 / 1.2)
    }

    #[inline]
    fn i2c_timeout() -> esp_idf_hal::delay::TickType_t {
        TickType::new_millis(100).ticks()
    }
}
