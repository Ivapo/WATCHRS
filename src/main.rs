
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowId},
};
struct App {
    // main_window: Option<WindowId>,
    _window: Option<Window>,
}

impl ApplicationHandler<()> for App {
    // We’ll add window creation here in the next step.
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // This is called when the application is resumed
        // Create a window with a title
        let attrs = Window::default_attributes().with_title("WATCHRS - Analog Clock");
        let window= event_loop.create_window(attrs).unwrap();
        // self.main_window = Some(window.id());
        self._window = Some(window);
    }

    fn window_event(
            &mut self,
            event_loop: &ActiveEventLoop,   
            _id: WindowId,
            event: WindowEvent,
        ) {
        // he is the code given a window event
        // Handle the close button
        if matches!(event, WindowEvent::CloseRequested) {
            event_loop.exit();
        }
    }   
}
fn main() {
    // 1) Create the event loop on the main thread
    let event_loop = EventLoop::new().unwrap();

    // 2) Run your (empty) app inside that loop
    let mut app = App { _window: None };
    event_loop.run_app(&mut app).unwrap();
}
