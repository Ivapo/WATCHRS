#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

use std::num::NonZeroU32;
use std::rc::Rc;
use std::time::{Duration, Instant};

use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop,ControlFlow},
    window::{Window, WindowId},
    dpi::PhysicalSize,
};

use softbuffer;


mod icon;
mod draw;

const WIDTH: usize = 1200;
const HEIGHT: usize = 900;
const COLOR_BACKGROUND: u32 = draw::color_rgb(75, 95, 100);
const COLOR_1: u32 = draw::color_rgb(0, 200, 255);

struct App {
    window: Option<Rc<Window>>,
    surface: Option<softbuffer::Surface<Rc<Window>, Rc<Window>>>, // double Rc for both window and display
    start: Instant,
}

impl ApplicationHandler<()> for App {
    // We’ll add window creation here in the next step.
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {

        // Create a window with a title and icon
        let attrs = Window::default_attributes()
            .with_title("WATCHRS - Analog Clock")
            .with_window_icon(icon::load_icon_embedded())
            .with_inner_size(PhysicalSize::new(WIDTH as u32, HEIGHT as u32))
            .with_resizable(true)
            ;

        // Own the window via Rc so we can hand owned handles to softbuffer
        let window = Rc::new(event_loop.create_window(attrs).unwrap());
        let context = softbuffer::Context::new(window.clone()).unwrap();
        let mut surface = softbuffer::Surface::new(&context, window.clone()).unwrap();


        // always resize the surface to the actual inner_size (PHYSICAL)
        let sz = window.inner_size(); 
        surface
            .resize(
                NonZeroU32::new(sz.width).unwrap(),
                NonZeroU32::new(sz.height).unwrap(),
            )
            .unwrap();

        self.window = Some(window);
        self.surface = Some(surface);
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.start);
        let secs = elapsed.as_secs();

        // Schedule the wake-up for the *next* whole second
        let next_tick = self.start + Duration::from_secs(secs + 1);
        event_loop.set_control_flow(ControlFlow::WaitUntil(next_tick));

        // Always redraw on wake-up
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

            WindowEvent::Resized(new_size) => {
                let surface = self.surface.as_mut().unwrap();
                surface.resize(
                    NonZeroU32::new(new_size.width).unwrap(),
                    NonZeroU32::new(new_size.height).unwrap()). 
                    unwrap();
            }

            WindowEvent::RedrawRequested => {
                use draw::Point;

                let surface = self.surface.as_mut().unwrap();
                let window = self.window.as_ref().unwrap();
                let window_size = window.inner_size();

                // Acquire the frame
                let canvas_size = draw::Dimensions { width: window_size.width as usize, height: window_size.height as usize };
                let mut canvas_buffer = surface.buffer_mut().unwrap();
                let mut canvas = draw::Canvas::new(&mut canvas_buffer, canvas_size);

                // use our clear() drawing function
                canvas.clear(COLOR_BACKGROUND);


                // canvas.draw_line(
                //     Point::new(700, 50),
                //     Point::new(50, 700),
                //     30, // thickness in pixels
                //     draw::color_rgb(0, 200, 0),
                // );

                // // Blue filled circle
                // canvas.draw_filled_circle(
                //     Point::new(300, 300),
                //     50,
                //     draw::color_rgb(0, 200, 255)
                // );

                // // responsive circle: radius is 45% of the shorter side
                // let center = canvas.center();
                // let radius = (canvas.min_dim() as f32 * 0.45).round() as isize;
                // canvas.draw_filled_circle(center,radius,COLOR_1);

                // // responsive line: from left edge to right edge through center, thickness scales too
                // let left  = Point::new(0, center.y);
                // let right = Point::new(canvas.width() as isize - 1, center.y);
                // let thick = (canvas.min_dim() as f32 * 0.03).max(1.0).round() as isize;
                // canvas.draw_line(left, right, thick, draw::color_rgb(255, 200, 80));

                let thick = (canvas.min_dim() as f32 * 0.03).max(1.0).round() as isize;
                let frame_padding = (canvas.min_dim() as f32 * 0.04).max(1.0).round() as isize;
                canvas.draw_frame(frame_padding, thick, COLOR_1);

                // Clock hand geometry
                let center = canvas.center();
                let radius = canvas.min_dim() / 2usize - (frame_padding*2) as usize - 2;

                // Whole seconds since launch (0..59), always starts at 0 on first frame
                let secs = (self.start.elapsed().as_secs() % 60) as f32;

                // Angle: 0..2π over 60s, with 0 pointing UP (12 o'clock)
                let angle = -std::f32::consts::FRAC_PI_2 + secs * (std::f32::consts::TAU / 60.0);

                // Tip of the hand
                let tip = Point::new(
                    center.x + (angle.cos() * radius as f32).round() as isize,
                    center.y + (angle.sin() * radius as f32).round() as isize,
                );

                // Draw the hand
                canvas.draw_line(center, tip,thick, COLOR_1);

                window.pre_present_notify();
                canvas_buffer.present().unwrap();
            }

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
        start: Instant::now(),
    };
    event_loop.run_app(&mut app).unwrap();
}
