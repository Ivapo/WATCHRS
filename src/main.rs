#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

use std::num::NonZeroU32;
use std::rc::Rc;

use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowId},
    dpi::PhysicalSize,
};

use softbuffer;

mod icon;
mod draw;

const WIDTH: usize = 1200;
const HEIGHT: usize = 900;

struct App {
    window: Option<Rc<Window>>,
    surface: Option<softbuffer::Surface<Rc<Window>, Rc<Window>>>, // double Rc for both window and display
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

            WindowEvent::Resized(new_size) => {
                let surface = self.surface.as_mut().unwrap();
                surface.resize(
                    NonZeroU32::new(new_size.width).unwrap(),
                    NonZeroU32::new(new_size.height).unwrap()). 
                    unwrap();
            }

            WindowEvent::RedrawRequested => {
                let window = self.window.as_ref().unwrap();
                let window_size = window.inner_size();
                let canvas_size = draw::Dimensions { width: window_size.width as usize, height: window_size.height as usize };
                
                let surface = self.surface.as_mut().unwrap();

                // Acquire the frame
                let mut canvas_buffer = surface.buffer_mut().unwrap();

                let mut canvas = draw::Canvas::new(&mut canvas_buffer, canvas_size);

                // use our clear() drawing function
                canvas.clear(draw::color_rgb(255, 0, 0));

                use draw::Point;

                canvas.draw_line(
                    Point::new(700, 50),
                    Point::new(50, 700),
                    30, // thickness in pixels
                    draw::color_rgb(0, 200, 0),
                );

                canvas.draw_line(
                    Point::new(100, 100),
                    Point::new(700, 700),
                    15,
                    draw::color_rgb(0, 200, 255),
                );

                // Blue filled circle
                canvas.draw_filled_circle(
                    Point::new(300, 300),
                    50,
                    draw::color_rgb(0, 200, 255)
                );

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
    };
    event_loop.run_app(&mut app).unwrap();
}
