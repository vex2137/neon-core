//! Zero-allocation fixed-capacity UTF-8 stack buffer and floating combat text.

use macroquad::color::Color;
use macroquad::math::Vec2;
use std::fmt;

/// Stack-allocated UTF-8 text buffer (`Copy`, zero heap allocations).
/// Used in hot combat popups and per-frame HUD label formatting.
#[derive(Clone, Copy)]
pub struct TextBuf<const N: usize = 32> {
    buf: [u8; N],
    len: u8,
}

impl<const N: usize> TextBuf<N> {
    #[inline]
    pub const fn new() -> Self {
        Self {
            buf: [0; N],
            len: 0,
        }
    }

    #[inline]
    pub fn from_str_trunc(s: &str) -> Self {
        let mut out = Self::new();
        out.push_str(s);
        out
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        // Safety invariant: `push_str` only copies valid UTF-8 slices at char boundaries.
        std::str::from_utf8(&self.buf[..self.len as usize]).unwrap_or("")
    }

    #[inline]
    pub fn push_str(&mut self, s: &str) {
        let max_cap = N.min(u8::MAX as usize);
        let mut cur = self.len as usize;
        for ch in s.chars() {
            let mut enc = [0u8; 4];
            let encoded = ch.encode_utf8(&mut enc).as_bytes();
            if cur + encoded.len() > max_cap {
                break;
            }
            self.buf[cur..cur + encoded.len()].copy_from_slice(encoded);
            cur += encoded.len();
        }
        self.len = cur as u8;
    }
}

impl<const N: usize> Default for TextBuf<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> fmt::Write for TextBuf<N> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.push_str(s);
        Ok(())
    }
}

#[derive(Clone, Copy)]
pub struct DmgText {
    pub pos: Vec2,
    pub val: TextBuf<32>,
    pub life: f32,
    pub color: Color,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::Write;

    #[test]
    fn text_buf_formats_and_truncates_utf8_cleanly() {
        let mut tb = TextBuf::<16>::new();
        let _ = write!(&mut tb, "HULL {}/{}", 100, 120);
        assert_eq!(tb.as_str(), "HULL 100/120");

        let long = TextBuf::<8>::from_str_trunc("OVERDRIVE! +123 SHARDS");
        assert_eq!(long.as_str(), "OVERDRIV");
    }
}
