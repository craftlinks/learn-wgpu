use raw_window_handle::HasRawWindowHandle;
use raw_window_handle::RawWindowHandle::Win32;
use windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, GetMessageW, PeekMessageW, TranslateMessage, MSG, PM_REMOVE, WM_QUIT, PostQuitMessage,
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
                    //self.render()?;
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

    // fn render(&mut self) -> Result<()> {
    //     println!("APP_RENDER");
    //     let gfx = self.window.gfx.as_mut().unwrap();
    //     match gfx.render() {
    //         Ok(_) => {}
    //         // Reconfigure the surface if lost
    //         Err(wgpu::SurfaceError::Lost) => gfx.resize(self.window.width as u32, self.window.height as u32),
    //         // The system is out of memory, we should probably quit
    //         Err(wgpu::SurfaceError::OutOfMemory) => unsafe {PostQuitMessage(0)},
    //         // All other errors (Outdated, Timeout) should be resolved by the next frame
    //         Err(e) => eprintln!("{:?}", e),
    //     }
    //     Ok(())
    // }
}
