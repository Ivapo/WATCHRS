
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowId,Icon},
};


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


struct App {
    // main_window: Option<WindowId>,
    _window: Option<Window>,
}

impl ApplicationHandler<()> for App {
    // We’ll add window creation here in the next step.
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // I first load the resources/icon.png to use it as the window icon
        let icon = load_icon("resources//icon.png");

        // Create a window with a title and icon
        let attrs = Window::default_attributes()
            .with_title("WATCHRS - Analog Clock")
            .with_window_icon(icon)
            // .with_fullscreen(Some(Fullscreen::Borderless(None)))
            ;

        let window= event_loop.create_window(attrs).unwrap();
        self._window = Some(window);
    }

    fn window_event(
            &mut self,
            event_loop: &ActiveEventLoop,   
            _id: WindowId,
            event: WindowEvent,
        ) {
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
