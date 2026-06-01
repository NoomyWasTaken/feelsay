// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    if std::env::args().any(|arg| arg == "--caption-placement-child") {
        let settings = overlay_settings_from_env();
        let placement_file =
            std::env::var_os("FEELSAY_OVERLAY_PLACEMENT_FILE").map(std::path::PathBuf::from);
        run_caption_window_child(settings, placement_file);
        return;
    }

    if std::env::args().any(|arg| arg == "--caption-window-child") {
        run_caption_window_child(overlay_settings_from_env(), None);
        return;
    }

    feelsay_lib::run()
}

fn overlay_settings_from_env() -> feelsay_lib::overlay_settings::OverlaySettings {
    std::env::var("FEELSAY_OVERLAY_SETTINGS")
        .ok()
        .and_then(|settings| {
            serde_json::from_str::<feelsay_lib::overlay_settings::OverlaySettings>(&settings).ok()
        })
        .unwrap_or_default()
        .normalized()
}

#[cfg(windows)]
fn run_caption_window_child(
    settings: feelsay_lib::overlay_settings::OverlaySettings,
    placement_file: Option<std::path::PathBuf>,
) {
    use core::ffi::c_void;
    use feelsay_lib::overlay_settings::{hex_to_rgb, OverlayPlacement, OverlaySettings};
    use std::cell::Cell;
    use windows::{
        core::{w, PCWSTR},
        Win32::{
            Foundation::{COLORREF, HWND, LPARAM, LRESULT, POINT, RECT, SIZE, WPARAM},
            Graphics::Gdi::{
                BeginPaint, CreateCompatibleDC, CreateDIBSection, CreateFontW, CreatePen,
                CreateSolidBrush, DeleteDC, DeleteObject, DrawTextW, Ellipse, EndPaint,
                GetMonitorInfoW, MonitorFromWindow, SelectObject, SetBkMode, SetTextColor,
                AC_SRC_ALPHA, AC_SRC_OVER, ANTIALIASED_QUALITY, BITMAPINFO, BI_RGB,
                CLIP_DEFAULT_PRECIS, DEFAULT_CHARSET, DIB_RGB_COLORS, DT_CENTER, DT_SINGLELINE,
                DT_VCENTER, HDC, MONITORINFO, MONITOR_DEFAULTTONEAREST, OUT_DEFAULT_PRECIS,
                PAINTSTRUCT, PS_SOLID, TRANSPARENT,
            },
            UI::WindowsAndMessaging::{
                CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetClientRect,
                GetCursorPos, GetMessageW, GetWindowLongPtrW, GetWindowRect, PostQuitMessage,
                RegisterClassW, SetWindowLongPtrW, SetWindowPos, TranslateMessage,
                UpdateLayeredWindow, CREATESTRUCTW, CW_USEDEFAULT, GWLP_USERDATA, HTBOTTOM,
                HTBOTTOMLEFT, HTBOTTOMRIGHT, HTCAPTION, HTCLIENT, HTLEFT, HTRIGHT, HTTOP,
                HTTOPLEFT, HTTOPRIGHT, HWND_TOPMOST, MSG, SWP_NOMOVE, SWP_NOSIZE, ULW_ALPHA,
                WINDOW_EX_STYLE, WINDOW_STYLE, WM_CREATE, WM_DESTROY, WM_ERASEBKGND,
                WM_EXITSIZEMOVE, WM_LBUTTONUP, WM_MOVE, WM_MOVING, WM_NCCALCSIZE, WM_NCHITTEST,
                WM_PAINT, WM_SIZE, WNDCLASSW, WS_EX_APPWINDOW, WS_EX_LAYERED, WS_EX_TOPMOST,
                WS_MAXIMIZEBOX, WS_MINIMIZEBOX, WS_POPUP, WS_SYSMENU, WS_THICKFRAME, WS_VISIBLE,
            },
        },
    };

    const BACKGROUND_MARKER: u32 = 0x00010101;
    const RESIZE_BORDER: i32 = 10;
    const CENTER_SNAP_THRESHOLD: i32 = 8;
    const CENTER_SNAP_RELEASE_THRESHOLD: i32 = 22;
    const SET_BUTTON_SIZE: i32 = 30;
    const SET_BUTTON_MARGIN: i32 = 12;

    struct CaptionWindowState {
        settings: OverlaySettings,
        placement_file: Option<std::path::PathBuf>,
        snap_x_cursor: Cell<Option<i32>>,
        snap_y_cursor: Cell<Option<i32>>,
        snap_x_suppressed: Cell<bool>,
        snap_y_suppressed: Cell<bool>,
    }

    unsafe extern "system" fn window_proc(
        hwnd: HWND,
        message: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        match message {
            WM_CREATE => {
                let create_struct = lparam.0 as *const CREATESTRUCTW;
                let state = unsafe { (*create_struct).lpCreateParams as *mut CaptionWindowState };
                unsafe {
                    SetWindowLongPtrW(hwnd, GWLP_USERDATA, state as isize);
                }
                LRESULT(0)
            }
            WM_NCCALCSIZE => LRESULT(0),
            WM_NCHITTEST => unsafe { hit_test_caption_window(hwnd, lparam) },
            WM_ERASEBKGND => LRESULT(1),
            WM_SIZE => {
                unsafe {
                    render_caption(hwnd);
                    write_placement(hwnd, false);
                }
                LRESULT(0)
            }
            WM_MOVE => {
                unsafe {
                    write_placement(hwnd, false);
                }
                LRESULT(0)
            }
            WM_MOVING => {
                unsafe {
                    snap_to_center(hwnd, lparam);
                    write_moving_placement(hwnd, lparam);
                }
                LRESULT(1)
            }
            WM_EXITSIZEMOVE => {
                unsafe {
                    reset_snap_state(hwnd);
                }
                LRESULT(0)
            }
            WM_LBUTTONUP => unsafe { handle_click(hwnd, lparam) },
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
                    let state = SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
                    if state != 0 {
                        drop(Box::from_raw(state as *mut CaptionWindowState));
                    }
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

        if is_placement_mode(hwnd) && screen_point_in_set_button(window_rect, cursor_x, cursor_y) {
            return LRESULT(HTCLIENT as isize);
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

    unsafe fn handle_click(hwnd: HWND, lparam: LPARAM) -> LRESULT {
        if !is_placement_mode(hwnd) {
            return DefWindowProcW(hwnd, WM_LBUTTONUP, WPARAM(0), lparam);
        }

        let mut client_rect = RECT::default();
        if GetClientRect(hwnd, &mut client_rect).is_ok()
            && client_point_in_rect(
                lparam_low_word(lparam),
                lparam_high_word(lparam),
                set_button_rect(client_rect),
            )
        {
            write_placement(hwnd, true);
            let _ = DestroyWindow(hwnd);
            return LRESULT(0);
        }

        LRESULT(0)
    }

    unsafe fn render_caption(hwnd: HWND) {
        let Some(state) = caption_state(hwnd) else {
            return;
        };
        let settings = &state.settings;

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

        let font_family: Vec<u16> = settings.font_family.encode_utf16().chain([0]).collect();
        let font = CreateFontW(
            -(settings.font_size as i32),
            0,
            0,
            0,
            settings.font_weight.win32_weight(),
            0,
            0,
            0,
            DEFAULT_CHARSET,
            OUT_DEFAULT_PRECIS,
            CLIP_DEFAULT_PRECIS,
            ANTIALIASED_QUALITY,
            0,
            PCWSTR(font_family.as_ptr()),
        );
        let previous_font = SelectObject(hdc, font.into());
        SetBkMode(hdc, TRANSPARENT);

        let format = DT_CENTER | DT_VCENTER | DT_SINGLELINE;
        let text = "Live captions will appear here.";

        if settings.outline_width > 0 {
            SetTextColor(hdc, color_ref(&settings.outline_color));
            let outline_width = settings.outline_width as i32;
            for x in -outline_width..=outline_width {
                for y in -outline_width..=outline_width {
                    if x == 0 && y == 0 {
                        continue;
                    }
                    let mut outline_rect = offset_rect(rect, x, y);
                    let mut outline_text: Vec<u16> = text.encode_utf16().collect();
                    DrawTextW(hdc, &mut outline_text, &mut outline_rect, format);
                }
            }
        }

        SetTextColor(hdc, color_ref(&settings.text_color));
        let mut caption_text: Vec<u16> = text.encode_utf16().collect();
        DrawTextW(hdc, &mut caption_text, &mut rect, format);

        if state.placement_file.is_some() {
            draw_set_button(hdc, rect);
        }

        let (red, green, blue) = hex_to_rgb(&settings.background_color);
        let background_alpha =
            ((settings.background_opacity.clamp(0.0, 1.0) * 255.0).round() as u32).max(1);
        let background_pixel =
            (background_alpha << 24) | ((red as u32) << 16) | ((green as u32) << 8) | blue as u32;

        for pixel in pixels {
            if (*pixel & 0x00ffffff) == BACKGROUND_MARKER {
                *pixel = background_pixel;
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

    unsafe fn caption_state(hwnd: HWND) -> Option<&'static CaptionWindowState> {
        let pointer = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const CaptionWindowState;
        pointer.as_ref()
    }

    unsafe fn is_placement_mode(hwnd: HWND) -> bool {
        caption_state(hwnd)
            .and_then(|state| state.placement_file.as_ref())
            .is_some()
    }

    unsafe fn write_placement(hwnd: HWND, is_accepted: bool) {
        let Some(state) = caption_state(hwnd) else {
            return;
        };
        let Some(path) = state.placement_file.as_ref() else {
            return;
        };

        let mut rect = RECT::default();
        if GetWindowRect(hwnd, &mut rect).is_err() {
            return;
        }

        let placement = OverlayPlacement {
            x: rect.left,
            y: rect.top,
            width: (rect.right - rect.left).max(0) as u32,
            height: (rect.bottom - rect.top).max(0) as u32,
            is_accepted,
        };

        write_placement_file(path, placement);
    }

    unsafe fn write_moving_placement(hwnd: HWND, lparam: LPARAM) {
        let Some(state) = caption_state(hwnd) else {
            return;
        };
        let Some(path) = state.placement_file.as_ref() else {
            return;
        };

        let rect = *(lparam.0 as *const RECT);
        let placement = OverlayPlacement {
            x: rect.left,
            y: rect.top,
            width: (rect.right - rect.left).max(0) as u32,
            height: (rect.bottom - rect.top).max(0) as u32,
            is_accepted: false,
        };

        write_placement_file(path, placement);
    }

    fn write_placement_file(path: &std::path::Path, placement: OverlayPlacement) {
        if let Ok(content) = serde_json::to_string(&placement) {
            let _ = std::fs::write(path, content);
        }
    }

    unsafe fn snap_to_center(hwnd: HWND, lparam: LPARAM) {
        let Some(state) = caption_state(hwnd) else {
            return;
        };
        if state.placement_file.is_none() {
            return;
        }

        let mut cursor = POINT::default();
        if GetCursorPos(&mut cursor).is_err() {
            return;
        }

        let rect = &mut *(lparam.0 as *mut RECT);
        let monitor = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST);
        let mut monitor_info = MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };

        if !GetMonitorInfoW(monitor, &mut monitor_info).as_bool() {
            return;
        }

        let work_area = monitor_info.rcWork;
        let width = rect.right - rect.left;
        let height = rect.bottom - rect.top;
        let monitor_center_x = work_area.left + (work_area.right - work_area.left) / 2;
        let monitor_center_y = work_area.top + (work_area.bottom - work_area.top) / 2;
        let window_center_x = rect.left + width / 2;
        let window_center_y = rect.top + height / 2;

        if should_snap_axis(
            &state.snap_x_cursor,
            &state.snap_x_suppressed,
            cursor.x,
            window_center_x - monitor_center_x,
        ) {
            rect.left = monitor_center_x - width / 2;
            rect.right = rect.left + width;
        }

        if should_snap_axis(
            &state.snap_y_cursor,
            &state.snap_y_suppressed,
            cursor.y,
            window_center_y - monitor_center_y,
        ) {
            rect.top = monitor_center_y - height / 2;
            rect.bottom = rect.top + height;
        }
    }

    fn should_snap_axis(
        anchor_cursor: &Cell<Option<i32>>,
        is_suppressed: &Cell<bool>,
        cursor_position: i32,
        center_distance: i32,
    ) -> bool {
        if let Some(anchor_position) = anchor_cursor.get() {
            if (cursor_position - anchor_position).abs() <= CENTER_SNAP_RELEASE_THRESHOLD {
                return true;
            }

            anchor_cursor.set(None);
            is_suppressed.set(true);
            return false;
        }

        if is_suppressed.get() {
            if center_distance.abs() > CENTER_SNAP_THRESHOLD {
                is_suppressed.set(false);
            }
            return false;
        }

        if center_distance.abs() <= CENTER_SNAP_THRESHOLD {
            anchor_cursor.set(Some(cursor_position));
            return true;
        }

        false
    }

    unsafe fn reset_snap_state(hwnd: HWND) {
        let Some(state) = caption_state(hwnd) else {
            return;
        };

        state.snap_x_cursor.set(None);
        state.snap_y_cursor.set(None);
        state.snap_x_suppressed.set(false);
        state.snap_y_suppressed.set(false);
    }

    unsafe fn draw_set_button(hdc: HDC, client_rect: RECT) {
        let button_rect = set_button_rect(client_rect);
        let brush = CreateSolidBrush(color_ref("#00d1b2"));
        let pen = CreatePen(PS_SOLID, 1, color_ref("#ffffff"));
        let previous_brush = SelectObject(hdc, brush.into());
        let previous_pen = SelectObject(hdc, pen.into());

        let _ = Ellipse(
            hdc,
            button_rect.left,
            button_rect.top,
            button_rect.right,
            button_rect.bottom,
        );

        let button_font = CreateFontW(
            -10,
            0,
            0,
            0,
            700,
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
        let previous_font = SelectObject(hdc, button_font.into());
        SetTextColor(hdc, color_ref("#ffffff"));
        SetBkMode(hdc, TRANSPARENT);

        let mut label = "Set".encode_utf16().collect::<Vec<u16>>();
        let mut label_rect = button_rect;
        DrawTextW(
            hdc,
            &mut label,
            &mut label_rect,
            DT_CENTER | DT_VCENTER | DT_SINGLELINE,
        );

        SelectObject(hdc, previous_font);
        SelectObject(hdc, previous_pen);
        SelectObject(hdc, previous_brush);
        let _ = DeleteObject(button_font.into());
        let _ = DeleteObject(pen.into());
        let _ = DeleteObject(brush.into());
    }

    fn set_button_rect(client_rect: RECT) -> RECT {
        RECT {
            left: client_rect.right - SET_BUTTON_MARGIN - SET_BUTTON_SIZE,
            top: client_rect.top + SET_BUTTON_MARGIN,
            right: client_rect.right - SET_BUTTON_MARGIN,
            bottom: client_rect.top + SET_BUTTON_MARGIN + SET_BUTTON_SIZE,
        }
    }

    fn screen_point_in_set_button(window_rect: RECT, x: i32, y: i32) -> bool {
        let client_rect = RECT {
            left: 0,
            top: 0,
            right: window_rect.right - window_rect.left,
            bottom: window_rect.bottom - window_rect.top,
        };
        let button_rect = set_button_rect(client_rect);
        client_point_in_rect(x - window_rect.left, y - window_rect.top, button_rect)
    }

    fn client_point_in_rect(x: i32, y: i32, rect: RECT) -> bool {
        x >= rect.left && x < rect.right && y >= rect.top && y < rect.bottom
    }

    fn color_ref(color: &str) -> COLORREF {
        let (red, green, blue) = hex_to_rgb(color);
        COLORREF((red as u32) | ((green as u32) << 8) | ((blue as u32) << 16))
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

        let x = settings.start_x.unwrap_or(CW_USEDEFAULT);
        let y = settings.start_y.unwrap_or(CW_USEDEFAULT);
        let width = settings.start_width as i32;
        let height = settings.start_height as i32;
        let state = Box::into_raw(Box::new(CaptionWindowState {
            settings,
            placement_file,
            snap_x_cursor: Cell::new(None),
            snap_y_cursor: Cell::new(None),
            snap_x_suppressed: Cell::new(false),
            snap_y_suppressed: Cell::new(false),
        }));

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
            x,
            y,
            width,
            height,
            None,
            None,
            None,
            Some(state.cast::<c_void>()),
        )
        .map(|hwnd| {
            render_caption(hwnd);
            write_placement(hwnd, false);
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
            drop(Box::from_raw(state));
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
fn run_caption_window_child(
    _settings: feelsay_lib::overlay_settings::OverlaySettings,
    _placement_file: Option<std::path::PathBuf>,
) {
    eprintln!("caption window child process is Windows-only for now");
}
