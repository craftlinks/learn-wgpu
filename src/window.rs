use crate::win32_common::ToWide;
use std::ffi::c_void;
use std::os::raw;
use windows::Win32::Foundation::{BOOL, HWND, LPARAM, LRESULT, PWSTR, RECT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Input::KeyboardAndMouse::{ReleaseCapture, SetCapture, VK_MENU};
use windows::Win32::UI::WindowsAndMessaging::{
    AdjustWindowRect, CreateWindowExW, DefWindowProcW, DestroyWindow,
    GetWindowLongPtrW, LoadCursorW, MessageBoxW, PostQuitMessage,
    RegisterClassW, SetWindowLongPtrW, CREATESTRUCTW, CS_HREDRAW, CS_VREDRAW,
    CW_USEDEFAULT, GWLP_USERDATA, IDC_CROSS, MB_OK,
    WM_ACTIVATE, WM_CHAR, WM_DESTROY, WM_KEYDOWN, WM_KEYUP, WM_KILLFOCUS, WM_LBUTTONDOWN,
    WM_LBUTTONUP, WM_MOUSEHWHEEL, WM_MOUSEMOVE, WM_NCCREATE, WM_RBUTTONDOWN, WM_RBUTTONUP,
    WM_SYSKEYDOWN, WM_SYSKEYUP, WNDCLASSW, WS_CAPTION, WS_MINIMIZEBOX, WS_OVERLAPPEDWINDOW,
    WS_SYSMENU, WS_VISIBLE, WM_SIZE, GetClientRect, WM_PAINT,
};

use crate::keyboard::Keyboard;
use crate::mouse::Mouse;
use crate::gfx::GFX;

// Dealing with errors
//======================
// .map_err(|e| os_error!(e));
// return Err(os_error!(::windows::core::Error::from_win32()))
// For example: AdjustWindowRect(&mut wr, WS_CAPTION | WS_MINIMIZEBOX | WS_SYSMENU, BOOL(0)).ok().map_err(|e| win_error!(e))?;
use crate::error::Win32Error;
pub type Result<T> = core::result::Result<T, Win32Error>;

pub struct Window {
    pub width: i32,
    pub height: i32,
    window_name: String,
    window_handle: HWND,
    pub visible: bool,
    kbd: Keyboard,
    mouse: Mouse,
    gfx: Option<GFX>,
}

impl Window {
    pub fn new(width: i32, height: i32, window_user_name: &str) -> Window {
        
        Window {
            width,
            height,
            window_name: window_user_name.into(),
            window_handle: 0,
            visible: false, // will need to be set on actual window creation
            kbd: Keyboard::new(),
            mouse: Mouse::new(),
            gfx: None,
        }
    }

    pub fn initialize(&mut self) -> Result<()> {
        unsafe {
            let instance = GetModuleHandleW(None);
            let window_class_name = "window".to_wide().as_ptr() as *mut u16;

            let wc = {
                WNDCLASSW {
                    hCursor: LoadCursorW(None, IDC_CROSS),
                    hInstance: instance,
                    lpszClassName: PWSTR(window_class_name),

                    style: CS_HREDRAW | CS_VREDRAW,
                    lpfnWndProc: Some(Self::wndproc),
                    ..Default::default()
                }
            };

            let atom = RegisterClassW(&wc);
            debug_assert!(atom != 0);

            let window_handle = {
                // calculate window size based on desired client region size
                let mut wr = RECT::default();
                wr.left = 100;
                wr.right = self.width + wr.left;
                wr.top = 100;
                wr.bottom = self.height + wr.top;
                // Adjust window size to accomodate the desired client dimensions specified by `width` and `height`.
                AdjustWindowRect(&mut wr, WS_CAPTION | WS_MINIMIZEBOX | WS_SYSMENU, BOOL(0))
                    .ok()
                    .map_err(|e| win_error!(e))?;
                let window_name: &str = &self.window_name;
                CreateWindowExW(
                    Default::default(),
                    PWSTR(window_class_name),
                    PWSTR(window_name.to_wide().as_ptr() as *mut u16),
                    WS_OVERLAPPEDWINDOW | WS_VISIBLE,
                    CW_USEDEFAULT,
                    CW_USEDEFAULT,
                    wr.right - wr.left,
                    wr.bottom - wr.top,
                    None,
                    None,
                    instance,
                    self as *mut Window as *const c_void,
                )
            };


            // Initialize Graphics
            let mut gfx = pollster::block_on(GFX::new(&self)); 
            self.gfx = Some(gfx);
            
            // Check for error
            debug_assert!(window_handle != 0);
            debug_assert!(window_handle == self.window_handle);

            Ok(())
        }
    }

    fn render(&mut self) -> Result<()> {
        // TEST KBD CODE
        if self.kbd.key_is_pressed(VK_MENU) {
            unsafe {
                MessageBoxW(
                    0,
                    PWSTR("Message Received!".to_wide().as_ptr() as *mut u16),
                    PWSTR("ALT Key Pressed!".to_wide().as_ptr() as *mut u16),
                    MB_OK,
                );
            }
        }

        // TEST MOUSE CODE
        while !self.mouse.is_empty() {
            if let Some(event) = self.mouse.read() {
                if event.get_type() == crate::mouse::EventType::Move {
                    println!(
                        "Mouse Position: {}, {}",
                        event.get_pos_x(),
                        event.get_pos_y()
                    );
                }
            }
        }

        Ok(())
    }

    fn user_message_handler(&mut self, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        unsafe {
            match message {
                WM_ACTIVATE => {
                    self.visible = true;
                    0
                }

                WM_KEYDOWN | WM_SYSKEYDOWN => {
                    // filter for autorepeat key messages to decide whether to process a key press or not.
                    if lparam & 0x40000000 == 0 || self.kbd.auto_repeat_is_enabled() {
                        self.kbd.on_key_pressed(
                            wparam
                                .try_into()
                                .expect("failed to convert keycode to u8"),
                        );
                    }
                    0
                }

                WM_KEYUP | WM_SYSKEYUP => {
                    self.kbd
                        .on_key_released(wparam.try_into().expect("failed to convert keycode"));
                    0
                }

                WM_CHAR => {
                    self.kbd
                        .on_char(wparam.try_into().expect("failed to convert char"));
                    0
                }

                WM_KILLFOCUS => {
                    self.kbd.clear_state();
                    0
                }

                WM_MOUSEMOVE => {
                    // First 16-bits of lparam contain mouse x-position
                    let x = lparam & 0xFFFF;
                    // Next 16-bits of lparam contain mouse y-position
                    let y = (lparam >> 16) & 0xFFFF;

                    // Mouse inside client area
                    if x >= 0 && y >= 0 && x < self.width as isize && y < self.height as isize {
                        self.mouse.on_mouse_move(x, y);
                        if !self.mouse.is_in_window() {
                            // Still receive mouse move events when we leave the window client area
                            SetCapture(self.window_handle);
                            self.mouse.on_mouse_enter();
                        }
                    }
                    // Mouse outside client area
                    else {
                        // track mouse when left or right button is pressed (dragging)
                        if self.mouse.left_is_pressed() || self.mouse.right_is_pressed() {
                            self.mouse.on_mouse_move(x, y);
                        }
                        // Don't track mouse when leaving the client area
                        else {
                            ReleaseCapture();
                            self.mouse.on_mouse_leave();
                        }
                    }
                    0
                }

                WM_LBUTTONDOWN => {
                    self.mouse.on_left_pressed();
                    0
                }

                WM_RBUTTONDOWN => {
                    self.mouse.on_right_pressed();
                    0
                }

                WM_LBUTTONUP => {
                    self.mouse.on_left_released();
                    0
                }

                WM_RBUTTONUP => {
                    self.mouse.on_right_released();
                    0
                }

                WM_MOUSEHWHEEL => {
                    // First 16-bits of lparam contain mouse x-position
                    let x = lparam & 0xFFFF;
                    // Next 16-bits of lparam contain mouse y-position
                    let y = (lparam >> 16) & 0xFFFF;
                    self.mouse.on_wheel_delta(x, y, wparam);
                    0
                }

                WM_SIZE => {
                    println!("WM_SIZE");
                    let mut rc: RECT = RECT::default();
                    GetClientRect(self.window_handle, &mut rc);
                    let new_width = (rc.right - rc.left) as u32;
                    let new_height = (rc.bottom - rc.top) as u32;
                    // Update the GPU 
                    if let Some(gfx) = self.gfx.as_mut() {
                        gfx.resize(new_width, new_height);
                    }
                    0
                }

                WM_PAINT => {
                    println!("WM_PAINT");
                    let gfx =  self.gfx.as_mut().unwrap();
                    match gfx.render() {
                        Ok(_) => {}
                        // Reconfigure the surface if lost
                        Err(wgpu::SurfaceError::Lost) => gfx.resize(self.width as u32, self.height as u32),
                        // The system is out of memory, we should probably quit
                        Err(wgpu::SurfaceError::OutOfMemory) => PostQuitMessage(0),
                        // All other errors (Outdated, Timeout) should be resolved by the next frame
                        Err(e) => eprintln!("{:?}", e),
                    }
                    0
                }
    
                WM_DESTROY => {
                    PostQuitMessage(0);
                    0
                }
                _ => DefWindowProcW(self.window_handle, message, wparam, lparam),
            }
        }
    }

    extern "system" fn wndproc(
        window_handle: HWND,
        message: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        unsafe {
            if message == WM_NCCREATE {
                let cs = lparam as *const CREATESTRUCTW;
                let this = (*cs).lpCreateParams as *mut Self;
                (*this).window_handle = window_handle;
                SetWindowLongPtrW(window_handle, GWLP_USERDATA, this as isize);
            } else {
                let this = GetWindowLongPtrW(window_handle, GWLP_USERDATA) as *mut Self;
                if !this.is_null() {
                    return (*this).user_message_handler(message, wparam, lparam);
                }
            }

            DefWindowProcW(window_handle, message, wparam, lparam)
        }
    }
}

unsafe impl raw_window_handle::HasRawWindowHandle for Window {
    fn raw_window_handle(&self) -> raw_window_handle::RawWindowHandle {
        let mut handle = raw_window_handle::Win32Handle::empty();
        handle.hwnd = self.window_handle as *mut raw::c_void;
        handle.hinstance =
            unsafe { GetModuleHandleW(None) } as *mut raw::c_void;
        raw_window_handle::RawWindowHandle::Win32(handle)
    }
}


impl Drop for Window {
    fn drop(&mut self) {
        unsafe {
            if self.window_handle != 0 {
                println!("Destroying window.");
                let _ = DestroyWindow(self.window_handle)
                    .ok()
                    .map_err(|e| println!("{}", win_error!(e))); // TODO: error triggers on exit!?
            }
        }
    }
}
