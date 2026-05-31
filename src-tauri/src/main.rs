// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    if std::env::args().any(|arg| arg == "--caption-window-child") {
        run_caption_window_child();
        return;
    }

    feelsay_lib::run()
}

#[cfg(windows)]
fn run_caption_window_child() {
    use windows::{
        core::w,
        Win32::{
            Foundation::{COLORREF, HWND, LPARAM, LRESULT, POINT, RECT, SIZE, WPARAM},
            Graphics::Gdi::{
                BeginPaint, CreateCompatibleDC, CreateDIBSection, CreateFontW, DeleteDC,
                DeleteObject, DrawTextW, EndPaint, SelectObject, SetBkMode, SetTextColor,
                AC_SRC_ALPHA, AC_SRC_OVER, ANTIALIASED_QUALITY, BITMAPINFO, BI_RGB,
                CLIP_DEFAULT_PRECIS, DEFAULT_CHARSET, DIB_RGB_COLORS, DT_CENTER, DT_SINGLELINE,
                DT_VCENTER, FW_BOLD, OUT_DEFAULT_PRECIS, PAINTSTRUCT, TRANSPARENT,
            },
            UI::WindowsAndMessaging::{
                CreateWindowExW, DefWindowProcW, DispatchMessageW, GetClientRect, GetMessageW,
                GetWindowRect, PostQuitMessage, RegisterClassW, SetWindowPos, TranslateMessage,
                UpdateLayeredWindow, CW_USEDEFAULT, HTBOTTOM, HTBOTTOMLEFT, HTBOTTOMRIGHT,
                HTCAPTION, HTLEFT, HTRIGHT, HTTOP, HTTOPLEFT, HTTOPRIGHT, HWND_TOPMOST, MSG,
                SWP_NOMOVE, SWP_NOSIZE, ULW_ALPHA, WINDOW_EX_STYLE, WINDOW_STYLE, WM_DESTROY,
                WM_ERASEBKGND, WM_NCCALCSIZE, WM_NCHITTEST, WM_PAINT, WM_SIZE, WNDCLASSW,
                WS_EX_APPWINDOW, WS_EX_LAYERED, WS_EX_TOPMOST, WS_MAXIMIZEBOX, WS_MINIMIZEBOX,
                WS_POPUP, WS_SYSMENU, WS_THICKFRAME, WS_VISIBLE,
            },
        },
    };

    const BACKGROUND_ALPHA: u8 = 120;
    const BACKGROUND_MARKER: u32 = 0x00010101;
    const RESIZE_BORDER: i32 = 10;

    unsafe extern "system" fn window_proc(
        hwnd: HWND,
        message: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        match message {
            WM_NCCALCSIZE => LRESULT(0),
            WM_NCHITTEST => unsafe { hit_test_caption_window(hwnd, lparam) },
            WM_ERASEBKGND => LRESULT(1),
            WM_SIZE => {
                unsafe {
                    render_caption(hwnd);
                }
                LRESULT(0)
            }
            WM_PAINT => {
                unsafe {
                    let mut paint = PAINTSTRUCT::default();
                    let _ = BeginPaint(hwnd, &mut paint);
                    let _ = EndPaint(hwnd, &paint);
                    render_caption(hwnd);
                }
                LRESULT(0)
            }
            WM_DESTROY => {
                unsafe {
                    PostQuitMessage(0);
                }
                LRESULT(0)
            }
            _ => unsafe { DefWindowProcW(hwnd, message, wparam, lparam) },
        }
    }

    unsafe fn hit_test_caption_window(hwnd: HWND, lparam: LPARAM) -> LRESULT {
        let cursor_x = lparam_low_word(lparam);
        let cursor_y = lparam_high_word(lparam);
        let mut window_rect = RECT::default();

        if GetWindowRect(hwnd, &mut window_rect).is_err() {
            return LRESULT(HTCAPTION as isize);
        }

        let on_left = cursor_x < window_rect.left + RESIZE_BORDER;
        let on_right = cursor_x >= window_rect.right - RESIZE_BORDER;
        let on_top = cursor_y < window_rect.top + RESIZE_BORDER;
        let on_bottom = cursor_y >= window_rect.bottom - RESIZE_BORDER;

        let hit_test = match (on_left, on_right, on_top, on_bottom) {
            (true, _, true, _) => HTTOPLEFT,
            (_, true, true, _) => HTTOPRIGHT,
            (true, _, _, true) => HTBOTTOMLEFT,
            (_, true, _, true) => HTBOTTOMRIGHT,
            (true, _, _, _) => HTLEFT,
            (_, true, _, _) => HTRIGHT,
            (_, _, true, _) => HTTOP,
            (_, _, _, true) => HTBOTTOM,
            _ => HTCAPTION,
        };

        LRESULT(hit_test as isize)
    }

    fn lparam_low_word(lparam: LPARAM) -> i32 {
        (lparam.0 as u32 & 0xffff) as i16 as i32
    }

    fn lparam_high_word(lparam: LPARAM) -> i32 {
        ((lparam.0 as u32 >> 16) & 0xffff) as i16 as i32
    }

    unsafe fn render_caption(hwnd: HWND) {
        let mut rect = RECT::default();
        if GetClientRect(hwnd, &mut rect).is_err() {
            return;
        }

        let width = rect.right - rect.left;
        let height = rect.bottom - rect.top;
        if width <= 0 || height <= 0 {
            return;
        }

        let hdc = CreateCompatibleDC(None);
        let mut bitmap_info = BITMAPINFO::default();
        bitmap_info.bmiHeader.biSize = std::mem::size_of_val(&bitmap_info.bmiHeader) as u32;
        bitmap_info.bmiHeader.biWidth = width;
        bitmap_info.bmiHeader.biHeight = -height;
        bitmap_info.bmiHeader.biPlanes = 1;
        bitmap_info.bmiHeader.biBitCount = 32;
        bitmap_info.bmiHeader.biCompression = BI_RGB.0;

        let mut bits = std::ptr::null_mut();
        let Ok(bitmap) =
            CreateDIBSection(Some(hdc), &bitmap_info, DIB_RGB_COLORS, &mut bits, None, 0)
        else {
            let _ = DeleteDC(hdc);
            return;
        };

        let previous_bitmap = SelectObject(hdc, bitmap.into());
        let pixel_count = (width * height) as usize;
        let pixels = std::slice::from_raw_parts_mut(bits as *mut u32, pixel_count);
        pixels.fill(BACKGROUND_MARKER);

        let font = CreateFontW(
            -42,
            0,
            0,
            0,
            FW_BOLD.0 as i32,
            0,
            0,
            0,
            DEFAULT_CHARSET,
            OUT_DEFAULT_PRECIS,
            CLIP_DEFAULT_PRECIS,
            ANTIALIASED_QUALITY,
            0,
            w!("Segoe UI"),
        );
        let previous_font = SelectObject(hdc, font.into());
        SetBkMode(hdc, TRANSPARENT);

        let format = DT_CENTER | DT_VCENTER | DT_SINGLELINE;
        let text = "Live captions will appear here.";

        SetTextColor(hdc, COLORREF(0x00000000));
        for x in -2..=2 {
            for y in -2..=2 {
                if x == 0 && y == 0 {
                    continue;
                }
                let mut outline_rect = offset_rect(rect, x, y);
                let mut outline_text: Vec<u16> = text.encode_utf16().collect();
                DrawTextW(hdc, &mut outline_text, &mut outline_rect, format);
            }
        }

        SetTextColor(hdc, COLORREF(0x00ffffff));
        let mut caption_text: Vec<u16> = text.encode_utf16().collect();
        DrawTextW(hdc, &mut caption_text, &mut rect, format);

        for pixel in pixels {
            if (*pixel & 0x00ffffff) == BACKGROUND_MARKER {
                *pixel = (BACKGROUND_ALPHA as u32) << 24;
            } else {
                *pixel |= 0xff000000;
            }
        }

        let size = SIZE {
            cx: width,
            cy: height,
        };
        let source = POINT { x: 0, y: 0 };
        let blend = windows::Win32::Graphics::Gdi::BLENDFUNCTION {
            BlendOp: AC_SRC_OVER as u8,
            BlendFlags: 0,
            SourceConstantAlpha: 255,
            AlphaFormat: AC_SRC_ALPHA as u8,
        };

        let _ = UpdateLayeredWindow(
            hwnd,
            None,
            None,
            Some(&size),
            Some(hdc),
            Some(&source),
            COLORREF(0),
            Some(&blend),
            ULW_ALPHA,
        );

        SelectObject(hdc, previous_font);
        SelectObject(hdc, previous_bitmap);
        let _ = DeleteObject(font.into());
        let _ = DeleteObject(bitmap.into());
        let _ = DeleteDC(hdc);
    }

    fn offset_rect(rect: RECT, x: i32, y: i32) -> RECT {
        RECT {
            left: rect.left + x,
            top: rect.top + y,
            right: rect.right + x,
            bottom: rect.bottom + y,
        }
    }

    unsafe {
        let class_name = w!("FeelSayCaptionWindowChild");
        let window_class = WNDCLASSW {
            lpfnWndProc: Some(window_proc),
            lpszClassName: class_name,
            ..Default::default()
        };

        RegisterClassW(&window_class);

        if CreateWindowExW(
            WINDOW_EX_STYLE(WS_EX_LAYERED.0 | WS_EX_TOPMOST.0 | WS_EX_APPWINDOW.0),
            class_name,
            w!("FeelSay Caption Window"),
            WINDOW_STYLE(
                WS_POPUP.0
                    | WS_VISIBLE.0
                    | WS_THICKFRAME.0
                    | WS_SYSMENU.0
                    | WS_MINIMIZEBOX.0
                    | WS_MAXIMIZEBOX.0,
            ),
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            900,
            180,
            None,
            None,
            None,
            None,
        )
        .map(|hwnd| {
            render_caption(hwnd);
            let _ = SetWindowPos(
                hwnd,
                Some(HWND_TOPMOST),
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE,
            );
        })
        .is_err()
        {
            return;
        }

        let mut message = MSG::default();
        while GetMessageW(&mut message, None, 0, 0).into() {
            let _ = TranslateMessage(&message);
            DispatchMessageW(&message);
        }
    }
}

#[cfg(not(windows))]
fn run_caption_window_child() {
    eprintln!("caption window child process is Windows-only for now");
}
