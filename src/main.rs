#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

use std::num::NonZeroU32;
use std::rc::Rc;
use std::time::{Duration, Instant};

use winit::{
    application::ApplicationHandler,
    event::{WindowEvent, ElementState},
    event_loop::{ActiveEventLoop, EventLoop,ControlFlow},
    window::{Window, WindowId},
    dpi::PhysicalSize,
    keyboard::{Key, NamedKey},
};


use softbuffer;


mod icon;
mod draw;

const APP_NAME: &str = "WATCHRS - Analog Clock";
const WIDTH: usize = 1200;
const HEIGHT: usize = 900;
const COLOR_BACKGROUND: u32 = draw::color_rgb(75, 95, 100);
const COLOR_1: u32 = draw::color_rgb(0, 200, 255);
const MAX_FPS: u32 = 20; 
const MIN_FPS: u32 = 1;


struct App {
    window:     Option<Rc<Window>>,
    surface:    Option<softbuffer::Surface<Rc<Window>, Rc<Window>>>, // double Rc for both window and display
    start:      Instant,
    fps:        u32,
    next_frame: Instant
}

impl ApplicationHandler<()> for App {
    // We’ll add window creation here in the next step.
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {

        // Create a window
        let attrs = Window::default_attributes()
            .with_title(APP_NAME)
            .with_window_icon(icon::load_icon_embedded())
            .with_inner_size(PhysicalSize::new(WIDTH as u32, HEIGHT as u32))
            .with_resizable(true)
            ;

        // With an Rc we 'own' the window and hand owned handles to softbuffer
        let window = Rc::new(event_loop.create_window(attrs).unwrap());
        let context = softbuffer::Context::new(window.clone()).unwrap();
        let mut surface = softbuffer::Surface::new(&context, window.clone()).unwrap();


        // resize the surface to the actual inner_size (PHYSICAL)
        let window_size = window.inner_size(); 
        surface
            .resize(
                NonZeroU32::new(window_size.width).unwrap(),
                NonZeroU32::new(window_size.height).unwrap(),
            )
            .unwrap();

        self.window = Some(window);
        self.surface = Some(surface);
        // self.next_frame = Instant::now();
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let fps = self.fps.clamp(MIN_FPS, MAX_FPS);
        let now = Instant::now();

        // Are we at/after the scheduled time?
        if now >= self.next_frame {
            // 1) Request exactly one redraw for this tick
            if let Some(w) = &self.window {
                w.request_redraw();
            }

            let frame_duration = Duration::from_secs_f32(1.0 / fps as f32);
            // self.next_frame += frame_duration;
            // self.next_frame = self.start + self.start.elapsed() + frame_duration;
            loop {
                self.next_frame += frame_duration;
                if self.next_frame > now { break; }
            }
        }
        event_loop.set_control_flow(ControlFlow::WaitUntil(self.next_frame));
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

                let window = self.window.as_ref().unwrap();
                let window_size = window.inner_size();
                
                // Acquire the frame
                let surface = self.surface.as_mut().unwrap();
                let canvas_size = draw::Dimensions { width: window_size.width as usize, height: window_size.height as usize };
                let mut canvas_buffer = surface.buffer_mut().unwrap();

                let mut canvas = draw::Canvas::new(&mut canvas_buffer, canvas_size);

                canvas.clear(COLOR_BACKGROUND);
                let center = canvas.center();


                // Draw a frame around the edge
                let thick = (canvas.min_dim() as f32 * 0.03).max(1.0).round() as usize;
                let frame_padding = (canvas.min_dim() as f32 * 0.04).max(1.0).round() as usize;
                canvas.draw_frame(frame_padding, thick, COLOR_1);


                // Clock hand geometry
                // This saturating_sub() prevents the number usize from becoming negative
                // if it was isize, then it is: ((canvas.min_dim() / 2)-(frame_padding * 2)).max(0);
                let seconds_hand_length = (canvas.min_dim() / 2).saturating_sub(frame_padding * 2);

                // Tip of the hand
                let seconds_since_start = self.start.elapsed().as_secs_f32() % 60.0;
                let ticks = (seconds_since_start* self.fps as f32).round() / self.fps as f32; 
                let angle = -std::f32::consts::FRAC_PI_2 + ticks * (std::f32::consts::TAU / 60.0);

                let seconds_hand_tip = Point::new(
                    center.x + (angle.cos() * seconds_hand_length as f32).round() as isize,
                    center.y + (angle.sin() * seconds_hand_length as f32).round() as isize,
                );

                // Draw the hand
                canvas.draw_line(center, seconds_hand_tip,thick, COLOR_1);

                window.pre_present_notify();
                canvas_buffer.present().unwrap();
            }

            WindowEvent::KeyboardInput { event: key_event, .. } => {
                if key_event.state == ElementState::Pressed {

                    eprintln!(
                        "Key pressed: logical={:?}, text={:?}",
                        key_event.logical_key,
                        key_event.text
                    );

                    match &key_event.logical_key {
                        Key::Character(s) if s == "+" => {
                            // shift+'=' on many keyboards; this catches the "+" character
                            self.fps = (self.fps + 2).min(MAX_FPS);
                            eprintln!("TPS increased to: {}", self.fps);
                        }
                        Key::Character(s) if s == "-" => {
                            self.fps = self.fps.saturating_sub(2).max(MIN_FPS);
                            eprintln!("TPS reduced to:: {}", self.fps);
                        }
                        Key::Named(NamedKey::Escape) => event_loop.exit(),
                        _ => {}
                    }
                }
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
        fps:MIN_FPS,
        next_frame: Instant::now(),
    };
    event_loop.run_app(&mut app).unwrap();
}
