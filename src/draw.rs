
/// Pack 8-bit R, G, B into a single u32 pixel in softbuffer's format: 0x00RRGGBB.
///
/// Bit layout of the returned value (most-significant bit on the left):
///  31            24 23            16 15             8 7              0
/// [    unused = 0 ][      RED      ][     GREEN      ][      BLUE      ]
#[inline]
pub fn color_rgb(r: u8, g: u8, b: u8) -> u32 {
    // 1) Widen to u32 so shifts don't overflow 8-bit values.
    let r = r as u32;
    let g = g as u32;
    let b = b as u32;

    // 2) Move each channel into its byte slot: R->bits 23..16, G->15..8, B->7..0.
    let red_bits   = r << 16;
    let green_bits = g << 8;
    let blue_bits  = b;

    // 3) Combine with bitwise OR. Top byte (31..24) stays 0 (alpha unused). and return.
    red_bits | green_bits | blue_bits
}

// clear entire buffer with a single color
pub fn clear(buf: &mut [u32], color: u32) {
    buf.fill(color);
}