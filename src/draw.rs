
/// Pack 8-bit R, G, B into a single u32 pixel in softbuffer's format: 0x00RRGGBB.
///
/// Bit layout of the returned value (most-significant bit on the left):
///  31            24 23            16 15             8 7              0
/// [    unused = 0 ][      RED      ][     GREEN      ][      BLUE      ]
#[inline]
pub const fn color_rgb(r: u8, g: u8, b: u8) -> u32 {
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
        debug_assert_eq!(buf.len(), (size.width) * (size.height));
        Self { buf, size }
    }

    pub fn width(&self) -> usize  { 
        self.size.width 
    }
    pub fn height(&self) -> usize { 
        self.size.height
    }
    pub fn max_x(&self) -> usize { 
        self.width() - 1 
    }
    pub fn max_y(&self) -> usize { 
        self.height() - 1 
    }

    pub fn center(&self) -> Point {
        Point::new((self.width() as isize) / 2, (self.height() as isize) / 2)
    }

    pub fn min_dim(&self) -> usize {
        self.width().min(self.height())
    }
    // pub fn _aspect_ratio(&self) -> f32 {
    //     self.width() as f32 / self.height() as f32
    // }

    /// Convert normalized coords (0..1) into pixel coordinates.
    // pub fn _from_norm(&self, x: f32, y: f32) -> Point {
    //     let px = (x.clamp(0.0, 1.0) * (self.width().saturating_sub(1) as f32)).round() as isize;
    //     let py = (y.clamp(0.0, 1.0) * (self.height().saturating_sub(1) as f32)).round() as isize;
    //     Point::new(px, py)
    // }

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
        if x  >= self.width() || y >= self.height() {
            return;
        }
        self.buf[y * self.width() + x] = color;
    }

    pub fn draw_filled_circle(&mut self, center: Point, radius: isize, color: u32) {
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                if dx*dx + dy*dy <= (radius * radius) {
                    self.put_pixel(center.x + dx, center.y + dy, color);
                }
            }
        }
    }

    pub fn draw_line(&mut self, a: Point, b: Point, thickness: isize, color: u32) {
        let mut x0 = a.x;
        let mut y0 = a.y;
        let x1 = b.x;
        let y1 = b.y;

        let dx = (x1 - x0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let dy = -(y1 - y0).abs();
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;

        // let radius = (thickness as f32 * 0.5).ceil() as isize;
        let radius = thickness/2;


        loop {
            self.draw_filled_circle(Point::new(x0, y0), radius, color);

            if x0 == x1 && y0 == y1 { break; }

            let e2 = 2 * err;
            if e2 >= dy { err += dy; x0 += sx; }
            if e2 <= dx { err += dx; y0 += sy; }
        }
    }

    pub fn draw_frame(&mut self, padding: isize, thickness: isize, color: u32) {
        let w = self.max_x() as isize;
        let h = self.max_y() as isize;
        let p = padding;

        let top_left    = Point::new(p,     p); 
        let top_right   = Point::new(w - p, p);
        let bottom_left = Point::new(p,     h - p);
        let bottom_right= Point::new(w-p,   h - p);

        self.draw_line(top_left,top_right, thickness, color); 
        self.draw_line(top_left,bottom_left, thickness, color); 
        self.draw_line(bottom_left,bottom_right, thickness, color); 
        self.draw_line(bottom_right,top_right, thickness, color); 
    }
}
