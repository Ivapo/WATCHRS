use std::num::NonZeroU32;
use std::rc::Rc;

use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowId,Icon},
    dpi::PhysicalSize,
    raw_window_handle::{HasDisplayHandle, HasWindowHandle},
};



use softbuffer_rgb::{RgbBuffer, softbuffer};

const WIDTH: usize = 1200;
const HEIGHT: usize = 900;

fn load_icon(path: &str) -> Option<Icon> {
    match image::open(path) {
        Ok(img) => {
            let img = img.into_rgba8();
            let (width, height) = img.dimensions();
            Icon::from_rgba(img.into_raw(), width, height).ok()
        }
        Err(_) => {
            eprintln!("⚠️  Could not load icon file at: '{path}'");
            None
        }
    }
}


#[inline]
fn color_rgb(r: u8, g: u8, b: u8) -> [u8; 4] {
    [b, g, r, 0]
}


// Accept the actual handle types (D, W), bounded by the traits RgbBuffer uses.
fn clear<D, W>(buf: &mut RgbBuffer<WIDTH, HEIGHT, D, W>, color: [u8; 4])
where
    D: HasDisplayHandle,
    W: HasWindowHandle,
{
    for row in buf.pixels.iter_mut() {
        for px in row.iter_mut() {
            *px = color;
        }
    }
}

struct App {
    window: Option<Rc<Window>>,
    surface: Option<softbuffer::Surface<Rc<Window>, Rc<Window>>>, // <- note Rc<Window> here
}

impl ApplicationHandler<()> for App {
    // We’ll add window creation here in the next step.
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {

        // Create a window with a title and icon
        let attrs = Window::default_attributes()
            .with_title("WATCHRS - Analog Clock")
            .with_window_icon(load_icon("resources//icon.png"))
            .with_inner_size(PhysicalSize::new(WIDTH as u16, HEIGHT as u16))
            .with_resizable(false)
            ;

        // Own the window via Rc so we can hand owned handles to softbuffer
        let window = Rc::new(event_loop.create_window(attrs).unwrap());

        // request the exact PHYSICAL size again (some WMs tweak initial sizes)
        // window.request_inner_size(PhysicalSize::new(WIDTH as u16, HEIGHT as u16));

        // Create context + surface using Rc clones (cheap pointer clones)
        let context = softbuffer::Context::new(window.clone()).unwrap();
        let mut surface = softbuffer::Surface::new(&context, window.clone()).unwrap();

        // always resize the surface to the actual inner_size (PHYSICAL)
        let sz = window.inner_size(); // PhysicalSize<u32>
        surface
            .resize(
                NonZeroU32::new(sz.width).unwrap(),
                NonZeroU32::new(sz.height).unwrap(),
            )
            .unwrap();

        self.window = Some(window);
        self.surface = Some(surface);
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(w) = &self.window {
            w.request_redraw();
        }
    }

    fn window_event(
            &mut self,
            event_loop: &ActiveEventLoop,   
            _id: WindowId,
            event: WindowEvent,
        ) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),

            WindowEvent::RedrawRequested => {
                if let (Some(window), Some(surface)) = (&self.window, &mut self.surface) {
                    // Acquire the frame
                    let raw = surface.buffer_mut().unwrap();
                    let mut canva = RgbBuffer::<WIDTH, HEIGHT, _, _>::from_softbuffer(raw).unwrap();

                    // --- draw something visible ---
                    clear(&mut canva, color_rgb(255, 0, 0)); // background
                    // fill_rect(&mut rgb, 20, 20, 200, 120, rgb(35, 90, 160)); // rectangle
                    // draw_line(&mut rgb, 20, 180, 280, 60, rgb(240, 210, 40)); // diagonal line   
                    // draw_circle_outline(&mut rgb, 400, 300, 120, rgb(200, 60, 60)); // circle

                    window.pre_present_notify();
                    canva.buffer.present().unwrap();
                }
            }
            
            // WindowEvent::RedrawRequested => {
            //     if let Some(surface) = &mut self.surface {
            //         // Get a raw softbuffer buffer for this frame
            //         let raw = surface.buffer_mut().unwrap();

            //         // Wrap it in a typed, safe RgbBuffer (no unsafe)
            //         let mut rgb = RgbBuffer::<WIDTH, HEIGHT, _, _>::from_softbuffer(raw).unwrap();

            //         // Fill the entire buffer with a solid color using the safe pixels API
            //         for row in rgb.pixels.iter_mut() {
            //             for px in row.iter_mut() {
            //                 *px = [0, 30, 200, 40]; // 0RGB (the first byte must be 0)
            //             }
            //         }

            //         // Present the frame
            //         rgb.buffer.present().unwrap();
            //     }
            // }
            _ => {}
        }   
    }
}

fn main() {
    // 1) Create the event loop on the main thread
    let event_loop = EventLoop::new().unwrap();

    // 2) Run your (empty) app inside that loop
    let mut app = App {
        window: None,
        surface: None,
    };
    event_loop.run_app(&mut app).unwrap();
}
