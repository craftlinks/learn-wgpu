#[macro_use]
mod error;
use error::Win32Error;
mod win32_common;
mod window;
mod keyboard;
mod mouse;
mod app;
use app::App;
pub type Result<T> = core::result::Result<T, Win32Error>;

fn main() -> Result<()> {
    let mut app = App::new();
    app.run() 
}
