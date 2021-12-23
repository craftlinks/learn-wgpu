use raw_window_handle::HasRawWindowHandle;
use raw_window_handle::RawWindowHandle::Win32;
use windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, GetMessageW, PeekMessageW, TranslateMessage, MSG, PM_REMOVE, WM_QUIT,
};
pub type Result<T> = core::result::Result<T, Win32Error>;
use crate::{error::Win32Error, window::Window};

pub struct App {
    pub window: Window,
}

impl App {
    pub fn new() -> App {
        App {
            window: Window::new(800, 600, "-"),
        }
    }

    pub fn run(&mut self) -> Result<()> {
        self.window.initialize()?;
        let win_handle = self.window.raw_window_handle();
        match win_handle {
            Win32(Win32Handle) => println!("Window handle: {:?} - Instance: {:?}", Win32Handle.hwnd, Win32Handle.hinstance),
            _ => {}
        }

        let mut message = MSG::default();
        loop {
            unsafe {
                // Initially the window is not visible
                if self.window.visible {
                    while PeekMessageW(&mut message, None, 0, 0, PM_REMOVE).into() {
                        if message.message == WM_QUIT {
                            return Ok(());
                        }
                        TranslateMessage(&message);
                        DispatchMessageW(&message);
                    }
                    self.render()?;
                } else {
                    GetMessageW(&mut message, None, 0, 0);

                    if message.message == WM_QUIT {
                        return Ok(());
                    }
                    TranslateMessage(&message);
                    DispatchMessageW(&message);
                }
            }
        }
    }

    fn render(&mut self) -> Result<()> {
        Ok(())
    }
}
