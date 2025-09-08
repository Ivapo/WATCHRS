use winit::window::Icon;

// Embed the PNG bytes
const ICON_PNG: &[u8] = include_bytes!("..//resources//icon.png");

pub fn load_icon_embedded() -> Option<Icon> {
    match image::load_from_memory(ICON_PNG) {
        Ok(img) => {
            let img = img.into_rgba8();
            let (width, height) = img.dimensions();
            Icon::from_rgba(img.into_raw(), width, height).ok()
        }
        Err(_) => {
            eprintln!("⚠️  Could not load icon to embed.");
            None
        }
    }
}

#[allow(dead_code)]
pub fn load_icon(path: &str) -> Option<Icon> {
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