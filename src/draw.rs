
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

#[derive(Copy, Clone, Debug)]
pub struct Point {
    pub x: isize,
    pub y: isize,
}

impl Point {
    pub fn new(x: isize, y: isize) -> Self {
        Self { x, y }
    }
}

pub struct Dimensions {
    pub width: usize,
    pub height: usize,
}

pub struct Canvas<'a> {
    pub buf: &'a mut [u32],
    pub size: Dimensions,
}

impl<'a> Canvas<'a> {
    pub fn new(buf: &'a mut [u32], size: Dimensions) -> Self {
        // (Optional) sanity check in debug builds:
        debug_assert_eq!(buf.len(), (size.width as usize) * (size.height as usize));
        Self { buf, size }
    }

    #[inline] 
    pub fn width(&self) -> usize  { self.size.width }
    #[inline] 
    pub fn height(&self) -> usize { self.size.height}

    /// Clear the entire canvas with a color. can also be used to set a background.
    pub fn clear(&mut self, color: u32) {
        self.buf.fill(color);
    }

    /// Plot one pixel at (x,y), ignoring if out of bounds.
    pub fn put_pixel(&mut self, x: isize, y: isize, color: u32) {
        
        if x < 0 || y < 0 {
            return;
        }
        
        let (x, y) = (x as usize, y as usize);

        if x >= self.width() || y >= self.height() {
            return;
        }
        self.buf[y * self.width() as usize + x] = color;
    }

    pub fn draw_filled_circle(&mut self, center: Point, radius: usize, color: u32) {
        let r = radius as isize;
        let r2 = (radius * radius) as isize;

        for dy in -r..=r {
            for dx in -r..=r {
                if dx*dx + dy*dy <= r2 {
                    self.put_pixel(center.x + dx, center.y + dy, color);
                }
            }
        }
    }

    pub fn draw_line(&mut self, a: Point, b: Point, thickness: usize, color: u32) {
        let mut x0 = a.x;
        let mut y0 = a.y;
        let x1 = b.x;
        let y1 = b.y;

        let dx = (x1 - x0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let dy = -(y1 - y0).abs();
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;

        let radius = (thickness as f32 * 0.5).ceil() as usize;

        loop {
            self.draw_filled_circle(Point::new(x0, y0), radius, color);

            if x0 == x1 && y0 == y1 { break; }

            let e2 = 2 * err;
            if e2 >= dy { err += dy; x0 += sx; }
            if e2 <= dx { err += dx; y0 += sy; }
        }
    }
}
