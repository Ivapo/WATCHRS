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
        self.next_frame = Instant::now();
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let tps = self.fps.clamp(1, 60);
        let now = Instant::now();

        // Are we at/after the scheduled time?
        if now >= self.next_frame {
            // 1) Request exactly one redraw for this tick
            if let Some(w) = &self.window {
                w.request_redraw();
            }

            // 2) Advance next_frame
            if tps == 1 {
                // align to the next whole second since `self.start`
                let secs = now.duration_since(self.start).as_secs();
                self.next_frame = self.start + Duration::from_secs(secs + 1);
            } else {
                let frame = Duration::from_secs_f64(1.0 / tps as f64);
                // advance by whole frames until next_frame is in the future
                // (handles long pauses without spamming redraws)
                loop {
                    self.next_frame += frame;
                    if self.next_frame > now { break; }
                }
            }
        }
        event_loop.set_control_flow(ControlFlow::WaitUntil(self.next_frame));

        // let frame = if tps == 1 {
        //     // align to whole seconds
        //     let secs = now.duration_since(self.start).as_secs();
        //     let next_tick = self.start + Duration::from_secs(secs + 1);
        //     // schedule wake at the next whole second
        //     self.next_frame = next_tick;
        //     Duration::from_secs(1)
        // } else {
        //     // steady 1/tps
        //     Duration::from_secs_f64(1.0 / tps as f64)
        // };

        //     // If we fell behind, catch up so we only draw once per tick
        // let now = Instant::now();
        // if self.next_frame <= now {
        //     // schedule the *next* frame time
        //     self.next_frame = now + frame;
        //     // request one redraw now
        //     if let Some(w) = &self.window {
        //         w.request_redraw();
        //     }
        // }
            
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

                // // Whole seconds since launch (0..59), always starts at 0 on first frame
                // let secs = (self.start.elapsed().as_secs() % 60) as f32;

                // // Angle: 0..2π over 60s, with 0 pointing UP (12 o'clock)
                // let angle = -std::f32::consts::FRAC_PI_2 + secs * (std::f32::consts::TAU / 60.0);

                // New (fractional; always smooth, but you only *see* smoothness when TPS > 1)
                let secs = self.start.elapsed().as_secs_f32() % 60.0;

                // Angle: 0..2π over 60s, 0 points up
                let angle = -std::f32::consts::FRAC_PI_2 + secs * (std::f32::consts::TAU / 60.0);

                // Tip of the hand
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
                        // Key::Named(NamedKey::Add) => { 
                        //     self.fps = (self.fps + 1).min(60); 
                        // }
                        // Key::Named(NamedKey::Subtract) => { 
                        //     self.fps = self.fps.saturating_sub(1).max(1); 
                        // }
                        Key::Character(s) if s == "+" => {
                            // shift+'=' on many keyboards; this catches the "+" character
                            self.fps = (self.fps + 5).min(60);
                            eprintln!("TPS increased to: {}", self.fps);
                        }
                        Key::Character(s) if s == "-" => {
                            self.fps = self.fps.saturating_sub(5).max(1);
                            eprintln!("TPS reduced to:: {}", self.fps);
                        }
                        Key::Named(NamedKey::Escape) => event_loop.exit(),
                        _ => {}
                    }
                }
            }
            // WindowEvent::KeyboardInput { event, .. } => {
            //     use winit::keyboard::{KeyCode, PhysicalKey, KeyState};
            //     if let PhysicalKey::Code(keycode) = event.physical_key {
            //         match (keycode, event.state) {
            //             (KeyCode::Escape, KeyState::Pressed) => {
            //                 event_loop.exit();
            //             }
            //             // Add more key handling here
            //             _ => {}
            //         }
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
        start: Instant::now(),
        fps:1,
        next_frame: Instant::now(),
    };
    event_loop.run_app(&mut app).unwrap();
}
