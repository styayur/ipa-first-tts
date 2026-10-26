// SPDX-License-Identifier: GPL-3.0-or-later
//! Win32 boundary. Handles and terminated UTF-16 buffers never leave this module.
//! All UI state is owned by one message thread. The window procedure posts state
//! changes to that thread instead of borrowing state reentrantly from callbacks.
use ipa_palette::{
    config::{self, Hotkey, Settings},
    studio_fragment, utf16_packets,
};
use std::{
    mem::{size_of, zeroed},
    path::PathBuf,
    process::{Child, Command},
    ptr::{null, null_mut},
    sync::OnceLock,
};
use symbol_catalog::{Catalog, Symbol};
#[path = "mapping_window.rs"]
mod mapping_window;
use mapping_window::{MappingAction, MappingWindow};
use windows_sys::Win32::{
    Foundation::*,
    Graphics::Gdi::*,
    Security::*,
    System::{DataExchange::*, LibraryLoader::*, Memory::*, Threading::*},
    UI::{Controls::*, HiDpi::*, Input::KeyboardAndMouse::*, Shell::*, WindowsAndMessaging::*},
};
const EVENT: u32 = WM_APP + 1;
const TRAY: u32 = WM_APP + 2;
const TRAY_EVENT: u32 = WM_APP + 3;
const EXPLORER_EVENT: u32 = WM_APP + 4;
// windows-sys exposes NIN_SELECT and NINF_KEY separately.
const TRAY_KEYSELECT: u32 = NIN_SELECT | NINF_KEY;
const SEARCH: usize = 101;
const CATEGORY: usize = 102;
const LANGUAGE: usize = 103;
const UI_LANGUAGE: usize = 104;
const GRID: usize = 105;
const DETAILS: usize = 106;
const COPY: usize = 107;
const INSERT: usize = 108;
const FAVORITE: usize = 109;
const COMPOSE: usize = 110;
const BUFFER: usize = 111;
const CLEAR: usize = 112;
const STUDIO: usize = 113;
const SETTINGS: usize = 114;
const STATUS: usize = 115;
const ACTION: usize = 116;
const HOTKEY_EDIT: usize = 117;
const SAVE_SETTINGS: usize = 118;
const SEARCH_LABEL: usize = 119;
const TARGET_LABEL: usize = 120;
const COMPOSE_LABEL: usize = 121;
const RESET: usize = 122;
const MENU: usize = 123;
const MAPPING: usize = 124;
const SHOW: usize = 201;
const HIDE: usize = 202;
const QUIT: usize = 203;
const HOTKEY_ID: i32 = 1;
const CATEGORIES: &[&str] = &[
    "",
    "recent",
    "favorites",
    "ipa",
    "orthography",
    "diacritics",
    "combinations",
    "vowels",
    "consonants",
    "prosody",
    "punctuation",
];
fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain([0]).collect()
}
fn tray_tip(hotkey: &str) -> [u16; 128] {
    let mut tip = [0; 128];
    for (slot, unit) in tip.iter_mut().take(127).zip(
        format!(
            "IPA Symbol Palette v{} · {hotkey}",
            env!("CARGO_PKG_VERSION")
        )
        .encode_utf16(),
    ) {
        *slot = unit;
    }
    tip
}
unsafe fn window_executable(window: HWND) -> Option<PathBuf> {
    use std::os::windows::ffi::OsStringExt;
    let mut process = 0;
    GetWindowThreadProcessId(window, &mut process);
    let handle = Handle(OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, process));
    if handle.0.is_null() {
        return None;
    }
    let mut buffer = vec![0u16; 32768];
    let mut length = buffer.len() as u32;
    if QueryFullProcessImageNameW(handle.0, 0, buffer.as_mut_ptr(), &mut length) == 0 {
        return None;
    }
    Some(PathBuf::from(std::ffi::OsString::from_wide(
        &buffer[..length as usize],
    )))
}
pub fn error(message: &str) {
    // SAFETY: valid terminated strings and no borrowed window state.
    unsafe {
        MessageBoxW(
            null_mut(),
            wide(message).as_ptr(),
            wide("IPA Symbol Palette").as_ptr(),
            MB_OK | MB_ICONERROR,
        );
    }
}
unsafe fn text(hwnd: HWND) -> String {
    let n = GetWindowTextLengthW(hwnd).clamp(0, 8192) as usize;
    let mut buffer = vec![0; n + 1];
    let read = GetWindowTextW(hwnd, buffer.as_mut_ptr(), buffer.len() as i32).max(0) as usize;
    String::from_utf16_lossy(&buffer[..read])
}
unsafe fn set_text(hwnd: HWND, value: &str) {
    SetWindowTextW(hwnd, wide(value).as_ptr());
}

struct Handle(HANDLE);
struct AppIcon(HICON);
impl Drop for AppIcon {
    fn drop(&mut self) {
        unsafe {
            DestroyIcon(self.0);
        }
    }
}
unsafe fn palette_icon() -> Result<AppIcon, String> {
    let screen = GetDC(null_mut());
    if screen.is_null() {
        return Err("Cannot create palette icon".into());
    }
    let dc = CreateCompatibleDC(screen);
    let bitmap = CreateCompatibleBitmap(screen, 32, 32);
    ReleaseDC(null_mut(), screen);
    let mask = CreateBitmap(32, 32, 1, 1, [0u8; 128].as_ptr().cast());
    if dc.is_null() || bitmap.is_null() || mask.is_null() {
        if !dc.is_null() {
            DeleteDC(dc);
        }
        if !bitmap.is_null() {
            DeleteObject(bitmap);
        }
        if !mask.is_null() {
            DeleteObject(mask);
        }
        return Err("Cannot allocate palette icon".into());
    }
    let old = SelectObject(dc, bitmap);
    let brush = CreateSolidBrush(0x00563b1c);
    let mut area = RECT {
        left: 0,
        top: 0,
        right: 32,
        bottom: 32,
    };
    FillRect(dc, &area, brush);
    DeleteObject(brush);
    let font = CreateFontW(
        -28,
        0,
        0,
        0,
        600,
        0,
        0,
        0,
        DEFAULT_CHARSET as u32,
        0,
        0,
        ANTIALIASED_QUALITY as u32,
        0,
        wide("Segoe UI").as_ptr(),
    );
    let old_font = SelectObject(dc, font);
    SetTextColor(dc, 0xffffff);
    SetBkMode(dc, TRANSPARENT as i32);
    DrawTextW(
        dc,
        wide("ɐ").as_ptr(),
        1,
        &mut area,
        DT_SINGLELINE | DT_CENTER | DT_VCENTER | DT_NOPREFIX,
    );
    SelectObject(dc, old_font);
    DeleteObject(font);
    SelectObject(dc, old);
    let info = ICONINFO {
        fIcon: 1,
        xHotspot: 0,
        yHotspot: 0,
        hbmMask: mask,
        hbmColor: bitmap,
    };
    let icon = CreateIconIndirect(&info);
    DeleteObject(mask);
    DeleteObject(bitmap);
    DeleteDC(dc);
    if icon.is_null() {
        Err("Cannot finalize palette icon".into())
    } else {
        Ok(AppIcon(icon))
    }
}
impl Drop for Handle {
    fn drop(&mut self) {
        unsafe {
            if !self.0.is_null() {
                CloseHandle(self.0);
            }
        }
    }
}
struct Registration {
    window: HWND,
    id: i32,
}
impl Registration {
    unsafe fn new(window: HWND, id: i32, key: Hotkey) -> Result<Self, String> {
        if RegisterHotKey(window, id, key.modifiers | MOD_NOREPEAT, key.key) == 0 {
            Err(format!(
                "Hotkey registration failed (conflict or Windows restriction): {}",
                std::io::Error::last_os_error()
            ))
        } else {
            Ok(Self { window, id })
        }
    }
}
impl Drop for Registration {
    fn drop(&mut self) {
        unsafe {
            UnregisterHotKey(self.window, self.id);
        }
    }
}
#[derive(Debug)]
struct Target {
    window: HWND,
    process: u32,
    thread: u32,
    stamp: usize,
}
fn target_property() -> Vec<u16> {
    wide(&format!("IPAFirstPalette.Target.{}", std::process::id()))
}
impl Drop for Target {
    fn drop(&mut self) {
        unsafe {
            if self.stamp != 0
                && GetPropW(self.window, target_property().as_ptr()) as usize == self.stamp
            {
                RemovePropW(self.window, target_property().as_ptr());
            }
        }
    }
}
impl Target {
    unsafe fn capture(own: HWND) -> Option<Self> {
        let window = GetForegroundWindow();
        if window.is_null() || window == own {
            return None;
        }
        let mut process = 0;
        let thread = GetWindowThreadProcessId(window, &mut process);
        if thread == 0 || process == GetCurrentProcessId() {
            return None;
        }
        // A window property dies with the original HWND. PID/thread checks alone
        // would accept a recycled HWND belonging to a different window on that thread.
        static STAMP: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(1);
        let stamp = STAMP.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let stamped = SetPropW(window, target_property().as_ptr(), stamp as HANDLE) != 0;
        Some(Self {
            window,
            process,
            thread,
            stamp: if stamped { stamp } else { 0 },
        })
    }
    unsafe fn validate(&self) -> Result<(), String> {
        let mut process = 0;
        if IsWindow(self.window) == 0
            || GetWindowThreadProcessId(self.window, &mut process) != self.thread
            || process != self.process
        {
            return Err("The original target window no longer exists. Use Copy.".into());
        }
        if self.stamp == 0
            || GetPropW(self.window, target_property().as_ptr()) as usize != self.stamp
        {
            return Err("Cannot verify original target identity (window replaced or permission mismatch). Use Copy.".into());
        }
        Ok(())
    }
}
unsafe fn integrity(process: HANDLE) -> Result<u32, String> {
    let mut token = null_mut();
    if OpenProcessToken(process, TOKEN_QUERY, &mut token) == 0 {
        return Err("Cannot inspect target permissions. Use Copy.".into());
    }
    let token = Handle(token);
    let mut bytes = 0;
    GetTokenInformation(token.0, TokenIntegrityLevel, null_mut(), 0, &mut bytes);
    let mut data = vec![0usize; (bytes as usize).div_ceil(size_of::<usize>())];
    if GetTokenInformation(
        token.0,
        TokenIntegrityLevel,
        data.as_mut_ptr().cast(),
        bytes,
        &mut bytes,
    ) == 0
    {
        return Err("Cannot inspect integrity level. Use Copy.".into());
    }
    let label = &*(data.as_ptr().cast::<TOKEN_MANDATORY_LABEL>());
    let count = *GetSidSubAuthorityCount(label.Label.Sid);
    if count == 0 {
        return Err("Invalid target security identifier".into());
    }
    Ok(*GetSidSubAuthority(label.Label.Sid, u32::from(count - 1)))
}
fn packets(value: &str) -> Vec<INPUT> {
    utf16_packets(value)
        .into_iter()
        .map(|(unit, up)| INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: 0,
                    wScan: unit,
                    dwFlags: KEYEVENTF_UNICODE | if up { KEYEVENTF_KEYUP } else { 0 },
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        })
        .collect()
}
unsafe fn insert(target: &Target, value: &str, own: HWND) -> Result<(), String> {
    target.validate()?;
    let process = Handle(OpenProcess(
        PROCESS_QUERY_LIMITED_INFORMATION,
        0,
        target.process,
    ));
    if process.0.is_null() {
        return Err("Target process is inaccessible. Use Copy.".into());
    }
    if integrity(process.0)? > integrity(GetCurrentProcess())? {
        return Err("Target has higher privileges (UIPI). Use Copy in the target; do not elevate the palette.".into());
    }
    if [VK_CONTROL, VK_MENU, VK_SHIFT, VK_LWIN, VK_RWIN]
        .iter()
        .any(|k| GetAsyncKeyState(i32::from(*k)) < 0)
    {
        return Err("Release Ctrl / Alt / Shift / Windows keys, then choose Insert again.".into());
    }
    ShowWindow(own, SW_HIDE);
    if SetForegroundWindow(target.window) == 0 || GetForegroundWindow() != target.window {
        return Err("Windows refused target focus. Nothing was sent. Use Copy.".into());
    }
    target.validate()?;
    let events = packets(value);
    // Do not yield between foreground/identity validation and one ordered batch.
    if GetForegroundWindow() != target.window {
        return Err("Foreground changed; nothing sent. Use Copy.".into());
    }
    let sent = SendInput(
        events.len() as u32,
        events.as_ptr(),
        size_of::<INPUT>() as i32,
    );
    if sent as usize != events.len() {
        return Err(format!("Windows dispatched {sent}/{} input events; text may be partial. Check the target before retrying. Use Copy. UIPI / remote or app restrictions may apply.", events.len()));
    }
    Ok(())
}
unsafe fn copy(value: &str, owner: HWND) -> Result<(), String> {
    let units = wide(value);
    let memory = GlobalAlloc(GMEM_MOVEABLE, units.len() * 2);
    if memory.is_null() {
        return Err("Cannot allocate clipboard memory".into());
    }
    let pointer = GlobalLock(memory);
    if pointer.is_null() {
        GlobalFree(memory);
        return Err("Cannot lock clipboard memory".into());
    }
    std::ptr::copy_nonoverlapping(units.as_ptr(), pointer.cast::<u16>(), units.len());
    GlobalUnlock(memory);
    if OpenClipboard(owner) == 0 {
        GlobalFree(memory);
        return Err("Clipboard is busy; try Copy again".into());
    }
    let result = if EmptyClipboard() == 0 || SetClipboardData(13, memory).is_null() {
        GlobalFree(memory);
        Err("Clipboard write failed".into())
    } else {
        Ok(())
    }; // Ownership passes to Windows only on success.
    CloseClipboard();
    result
}
fn taskbar_created_message() -> u32 {
    static MESSAGE: OnceLock<u32> = OnceLock::new();
    *MESSAGE.get_or_init(|| unsafe { RegisterWindowMessageW(wide("TaskbarCreated").as_ptr()) })
}
unsafe extern "system" fn procedure(hwnd: HWND, message: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    match message {
        TRAY => {
            // Shell callbacks may be sent, not queued. Defer to the owning
            // loop rather than borrowing App reentrantly from this procedure.
            PostMessageW(hwnd, TRAY_EVENT, w, l);
            0
        }
        value if value != 0 && value == taskbar_created_message() => {
            PostMessageW(hwnd, EXPLORER_EVENT, 0, 0);
            0
        }
        WM_COMMAND => {
            PostMessageW(hwnd, EVENT, w, l);
            0
        }
        WM_TIMER => {
            PostMessageW(hwnd, EVENT, w, 0);
            0
        }
        WM_CLOSE => {
            PostMessageW(hwnd, EVENT, HIDE, 0);
            0
        }
        WM_DESTROY => {
            PostQuitMessage(0);
            0
        }
        WM_SIZE => {
            PostMessageW(hwnd, EVENT, 300, 0);
            0
        }
        WM_DPICHANGED => {
            let area = &*(l as *const RECT);
            SetWindowPos(
                hwnd,
                null_mut(),
                area.left,
                area.top,
                area.right - area.left,
                area.bottom - area.top,
                SWP_NOZORDER | SWP_NOACTIVATE,
            );
            PostMessageW(hwnd, EVENT, 301, 0);
            0
        }
        WM_GETMINMAXINFO => {
            let info = &mut *(l as *mut MINMAXINFO);
            let dpi = GetDpiForWindow(hwnd).max(96) as i32;
            info.ptMinTrackSize = POINT {
                x: 420 * dpi / 96,
                y: 600 * dpi / 96,
            };
            0
        }
        WM_DRAWITEM => {
            draw_item(&*(l as *const DRAWITEMSTRUCT));
            1
        }
        WM_MEASUREITEM => {
            (*(l as *mut MEASUREITEMSTRUCT)).itemHeight = 52;
            1
        }
        _ => DefWindowProcW(hwnd, message, w, l),
    }
}
unsafe fn draw_item(item: &DRAWITEMSTRUCT) {
    if item.itemID == u32::MAX {
        return;
    }
    let selected = item.itemState & ODS_SELECTED != 0;
    let background = GetSysColorBrush(if selected {
        COLOR_HIGHLIGHT
    } else {
        COLOR_WINDOW
    });
    FillRect(item.hDC, &item.rcItem, background);
    SetTextColor(
        item.hDC,
        GetSysColor(if selected {
            COLOR_HIGHLIGHTTEXT
        } else {
            COLOR_WINDOWTEXT
        }),
    );
    SetBkMode(item.hDC, TRANSPARENT as i32);
    let length = SendMessageW(item.hwndItem, LB_GETTEXTLEN, item.itemID as usize, 0);
    if !(0..=4096).contains(&length) {
        return;
    }
    let mut buffer = vec![0u16; length as usize + 1];
    SendMessageW(
        item.hwndItem,
        LB_GETTEXT,
        item.itemID as usize,
        buffer.as_mut_ptr() as isize,
    );
    let value = String::from_utf16_lossy(&buffer[..length as usize]);
    let (symbol, name) = value.split_once('\n').unwrap_or((&value, ""));
    let font = GetWindowLongPtrW(item.hwndItem, GWLP_USERDATA) as HFONT;
    let previous = SelectObject(item.hDC, font);
    let dpi = GetDpiForWindow(item.hwndItem).max(96) as i32;
    let mut rect = item.rcItem;
    rect.left += 3 * dpi / 96;
    rect.top += dpi / 96;
    rect.bottom = rect.top + 33 * dpi / 96;
    let units = wide(symbol);
    DrawTextW(
        item.hDC,
        units.as_ptr(),
        (units.len() - 1) as i32,
        &mut rect,
        DT_SINGLELINE | DT_CENTER | DT_VCENTER | DT_NOPREFIX | DT_END_ELLIPSIS,
    );
    let ui_font = GetPropW(item.hwndItem, wide("IPA.Palette.UiFont").as_ptr());
    SelectObject(
        item.hDC,
        if ui_font.is_null() {
            GetStockObject(DEFAULT_GUI_FONT)
        } else {
            ui_font
        },
    );
    rect.top = rect.bottom;
    rect.bottom = item.rcItem.bottom;
    let units = wide(name);
    DrawTextW(
        item.hDC,
        units.as_ptr(),
        (units.len() - 1) as i32,
        &mut rect,
        DT_SINGLELINE | DT_CENTER | DT_NOPREFIX | DT_END_ELLIPSIS,
    );
    SelectObject(item.hDC, previous);
    if item.itemState & ODS_FOCUS != 0 {
        DrawFocusRect(item.hDC, &item.rcItem);
    }
}

struct App {
    window: HWND,
    controls: Vec<(usize, HWND)>,
    catalog: Catalog,
    settings: Settings,
    config: PathBuf,
    target: Option<Target>,
    results: Vec<String>,
    registration: Option<Registration>,
    tray: NOTIFYICONDATAW,
    font: HFONT,
    symbol_font: HFONT,
    studio: Option<Child>,
    studio_port: Option<u16>,
    settings_window: HWND,
    settings_controls: Vec<(usize, HWND)>,
    mapping: Option<MappingWindow>,
}
impl Drop for App {
    fn drop(&mut self) {
        self.mapping.take();
        self.registration.take();
        if let Some(mut child) = self.studio.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
        unsafe {
            Shell_NotifyIconW(NIM_DELETE, &self.tray);
            if !self.settings_window.is_null() {
                DestroyWindow(self.settings_window);
            }
            if !self.window.is_null() {
                DestroyWindow(self.window);
            }
            DeleteObject(self.font);
            DeleteObject(self.symbol_font);
        }
    }
}
impl App {
    fn zh(&self) -> bool {
        self.settings.ui_language == "zh-CN"
    }
    fn label<'a>(&self, en: &'a str, zh: &'a str) -> &'a str {
        if self.zh() {
            zh
        } else {
            en
        }
    }
    fn get(&self, id: usize) -> HWND {
        self.controls
            .iter()
            .find(|c| c.0 == id)
            .map(|c| c.1)
            .unwrap_or(null_mut())
    }
    unsafe fn child(
        &mut self,
        id: usize,
        class: &str,
        title: &str,
        style: u32,
    ) -> Result<(), String> {
        let hwnd = create_control(self.window, id, class, title, style)?;
        self.controls.push((id, hwnd));
        Ok(())
    }
    unsafe fn status(&self, value: &str) {
        set_text(self.get(STATUS), value);
    }
    unsafe fn persist(&self) {
        if let Err(e) = config::save(&self.config, &self.settings) {
            self.status(&e);
        }
    }
    unsafe fn fonts(&mut self) {
        let dpi = GetDpiForWindow(self.window).max(96) as i32;
        let previous = self.font;
        let old_symbol = self.symbol_font;
        self.font = CreateFontW(
            -14 * dpi / 96,
            0,
            0,
            0,
            400,
            0,
            0,
            0,
            DEFAULT_CHARSET as u32,
            0,
            0,
            CLEARTYPE_QUALITY as u32,
            0,
            wide("Segoe UI").as_ptr(),
        );
        self.symbol_font = CreateFontW(
            -27 * dpi / 96,
            0,
            0,
            0,
            400,
            0,
            0,
            0,
            DEFAULT_CHARSET as u32,
            0,
            0,
            CLEARTYPE_QUALITY as u32,
            0,
            wide("Segoe UI").as_ptr(),
        );
        for (_, hwnd) in &self.controls {
            SendMessageW(*hwnd, WM_SETFONT, self.font as usize, 1);
        }
        SetWindowLongPtrW(self.get(GRID), GWLP_USERDATA, self.symbol_font as isize);
        SetPropW(
            self.get(GRID),
            wide("IPA.Palette.UiFont").as_ptr(),
            self.font,
        );
        SendMessageW(
            self.get(GRID),
            LB_SETITEMHEIGHT,
            0,
            (52 * dpi / 96) as isize,
        );
        if !previous.is_null() {
            DeleteObject(previous);
        }
        if !old_symbol.is_null() {
            DeleteObject(old_symbol);
        }
    }
    unsafe fn layout(&self) {
        let mut r = zeroed();
        GetClientRect(self.window, &mut r);
        let dpi = GetDpiForWindow(self.window).max(96) as i32;
        let scale = |v: i32| v * dpi / 96;
        let width = r.right * 96 / dpi;
        let height = r.bottom * 96 / dpi;
        let place = |id, x, y, w, h| {
            MoveWindow(self.get(id), scale(x), scale(y), scale(w), scale(h), 1);
        };
        let (column, grid_height, row) = ipa_palette::grid_dimensions(width, height);
        place(SEARCH_LABEL, 12, 8, width - 24, 20);
        place(SEARCH, 12, 32, width - 96, 30);
        place(RESET, width - 76, 32, 64, 30);
        place(CATEGORY, 12, 70, (width - 32) / 2, 320);
        place(LANGUAGE, 20 + (width - 32) / 2, 70, (width - 32) / 2, 320);
        place(TARGET_LABEL, 12, 103, width - 24, 22);
        place(GRID, 12, 128, width - 24, grid_height);
        SendMessageW(self.get(GRID), LB_SETITEMHEIGHT, 0, scale(row) as isize);
        SendMessageW(self.get(GRID), LB_SETCOLUMNWIDTH, scale(column) as usize, 0);
        place(DETAILS, 12, height - 258, width - 24, 72);
        place(COMPOSE_LABEL, 12, height - 182, width - 24, 18);
        place(BUFFER, 12, height - 162, width - 98, 28);
        place(CLEAR, width - 78, height - 162, 66, 28);
        let bw = (width - 48) / 4;
        for (i, id) in [COPY, INSERT, FAVORITE, COMPOSE].iter().enumerate() {
            place(*id, 12 + i as i32 * (bw + 8), height - 126, bw, 32);
        }
        let command_width = (width - 156) / 4;
        for (index, id) in [MAPPING, STUDIO, SETTINGS, MENU].iter().enumerate() {
            place(
                *id,
                12 + index as i32 * (command_width + 4),
                height - 86,
                command_width,
                29,
            );
        }
        place(UI_LANGUAGE, width - 124, height - 86, 112, 250);
        place(ACTION, 12, height - 49, 126, 160);
        place(STATUS, 146, height - 49, width - 226, 45);
        place(QUIT, width - 72, height - 49, 60, 29);
    }
    unsafe fn localize(&self) {
        for (id, en, zh) in [
            (COPY, "&Copy", "复制(&C)"),
            (INSERT, "&Insert", "插入(&I)"),
            (FAVORITE, "&Favorite", "收藏(&F)"),
            (COMPOSE, "Compose +", "组合 +"),
            (CLEAR, "Clear", "清空"),
            (STUDIO, "Studio", "Studio"),
            (MAPPING, "Mapping", "语言映射"),
            (SETTINGS, "Settings", "设置"),
            (RESET, "Reset", "重置"),
            (MENU, "Menu ▾", "菜单 ▾"),
            (QUIT, "Exit", "退出"),
        ] {
            set_text(self.get(id), self.label(en, zh));
        }
        set_text(
            self.window,
            &format!(
                "{} · v{}",
                self.label("IPA Symbol Palette", "IPA 字符面板"),
                env!("CARGO_PKG_VERSION")
            ),
        );
        SendMessageW(
            self.get(SEARCH),
            0x1501,
            1,
            wide(self.label(
                "Search symbols, names, language, U+0252…",
                "搜索字符、名称、语言、U+0252…",
            ))
            .as_ptr() as isize,
        );
        let labels_en = [
            "All symbols",
            "Recent",
            "Favorites",
            "IPA",
            "Language characters",
            "Diacritics",
            "Combinations",
            "Vowels",
            "Consonants",
            "Stress / tone / length",
            "Boundaries / punctuation",
        ];
        set_text(
            self.get(SEARCH_LABEL),
            self.label(
                "Search · symbols, names, languages or U+0252",
                "搜索 · 字符、名称、语言或 U+0252",
            ),
        );
        set_text(
            self.get(COMPOSE_LABEL),
            self.label(
                "Composition · optional; overrides selection when filled",
                "组合输出 · 可选，非空时优先于所选字符",
            ),
        );
        SendMessageW(
            self.get(BUFFER),
            0x1501,
            1,
            wide(self.label(
                "Select a symbol and choose Compose +",
                "选择字符后点“组合 +”，也可直接输入",
            ))
            .as_ptr() as isize,
        );
        let language_selection = SendMessageW(self.get(LANGUAGE), CB_GETCURSEL, 0, 0).max(0);
        let mut language_labels = vec![self.label("All languages", "全部语言").to_string()];
        language_labels.extend(
            self.catalog
                .languages
                .iter()
                .map(|l| format!("{} · {}", l.id, l.native_name)),
        );
        combo(
            self.get(LANGUAGE),
            &language_labels
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>(),
        );
        SendMessageW(
            self.get(LANGUAGE),
            CB_SETCURSEL,
            language_selection as usize,
            0,
        );
        let labels_zh = [
            "全部字符",
            "最近使用",
            "收藏",
            "IPA 音标",
            "语言字符",
            "附加符号",
            "组合序列",
            "元音",
            "辅音",
            "重音 / 声调 / 长音",
            "边界 / 标点",
        ];
        let category = SendMessageW(self.get(CATEGORY), CB_GETCURSEL, 0, 0).max(0);
        combo(
            self.get(CATEGORY),
            if self.zh() { &labels_zh } else { &labels_en },
        );
        SendMessageW(self.get(CATEGORY), CB_SETCURSEL, category as usize, 0);
        combo(
            self.get(ACTION),
            &[
                self.label("Enter: Insert", "Enter：插入"),
                self.label("Enter: Copy", "Enter：复制"),
            ],
        );
        SendMessageW(
            self.get(ACTION),
            CB_SETCURSEL,
            usize::from(self.settings.action == "copy"),
            0,
        );
        self.details();
        self.target_hint();
    }
    fn enter_action(&self) -> usize {
        if self.settings.action == "copy"
            || !self
                .target
                .as_ref()
                .is_some_and(|t| unsafe { t.validate().is_ok() })
        {
            COPY
        } else {
            INSERT
        }
    }
    unsafe fn target_hint(&self) {
        let available = self.target.as_ref().is_some_and(|t| t.validate().is_ok());
        EnableWindow(self.get(INSERT), i32::from(available));
        let message = if let Some(target) = self.target.as_ref().filter(|_| available) {
            let name = text(target.window);
            format!(
                "{} {}",
                self.label("Insert →", "插入到 →"),
                name.chars().take(60).collect::<String>()
            )
        } else {
            self.label(
                "Copy mode · focus an editor, then use the shortcut to insert",
                "复制模式 · 先聚焦编辑器，再用快捷键呼出即可插入",
            )
            .into()
        };
        set_text(self.get(TARGET_LABEL), &message);
        set_text(
            self.get(COPY),
            if self.enter_action() == COPY {
                self.label("Copy ↵", "复制 ↵")
            } else {
                self.label("&Copy", "复制(&C)")
            },
        );
        set_text(
            self.get(INSERT),
            if self.enter_action() == INSERT {
                self.label("Insert ↵", "插入 ↵")
            } else {
                self.label("&Insert", "插入(&I)")
            },
        );
    }
    unsafe fn refresh(&mut self) {
        let previous_id = self.selected().map(|s| s.id.clone());
        let query = text(self.get(SEARCH));
        let cat = SendMessageW(self.get(CATEGORY), CB_GETCURSEL, 0, 0).max(0) as usize;
        let category = CATEGORIES.get(cat).copied().unwrap_or("");
        let lang = SendMessageW(self.get(LANGUAGE), CB_GETCURSEL, 0, 0);
        let language = if lang > 0 {
            self.catalog
                .languages
                .get(lang as usize - 1)
                .map(|l| l.id.as_str())
        } else {
            None
        };
        let filter = if matches!(category, "" | "recent" | "favorites" | "ipa") {
            None
        } else {
            Some(category)
        };
        let mut found = self.catalog.search(&query, language, filter);
        if category == "ipa" {
            found.retain(|s| s.tags.iter().any(|t| t == "ipa"));
        }
        if matches!(category, "recent" | "favorites") {
            let personal = if category == "recent" {
                &self.settings.recent
            } else {
                &self.settings.favorites
            };
            found.retain(|s| personal.contains(&s.id));
            found.sort_by_key(|s| personal.iter().position(|id| id == &s.id));
        }
        self.results = found.iter().map(|s| s.id.clone()).collect();
        SendMessageW(self.get(GRID), WM_SETREDRAW, 0, 0);
        SendMessageW(self.get(GRID), LB_RESETCONTENT, 0, 0);
        for s in found {
            let display = if s.text == " " {
                "␠".into()
            } else if s.kind == symbol_catalog::SymbolKind::Diacritic
                && s.text
                    .chars()
                    .next()
                    .is_some_and(|c| (0x300..=0x36f).contains(&(c as u32)))
            {
                format!("◌{}", s.text)
            } else {
                s.text.clone()
            };
            SendMessageW(
                self.get(GRID),
                LB_ADDSTRING,
                0,
                wide(&format!(
                    "{display}\n{}",
                    if s.codepoints.len() == 1 {
                        s.codepoints[0].clone()
                    } else {
                        format!("{} {}", s.codepoints.len(), self.label("scalars", "码位"))
                    }
                ))
                .as_ptr() as isize,
            );
        }
        let selection = previous_id
            .and_then(|id| self.results.iter().position(|r| r == &id))
            .unwrap_or(0);
        SendMessageW(self.get(GRID), LB_SETCURSEL, selection, 0);
        SendMessageW(self.get(GRID), WM_SETREDRAW, 1, 0);
        InvalidateRect(self.get(GRID), null(), 1);
        self.details();
        self.target_hint();
        let has_selection = !self.results.is_empty();
        for id in [COPY, FAVORITE, COMPOSE] {
            EnableWindow(self.get(id), i32::from(has_selection));
        }
        EnableWindow(
            self.get(INSERT),
            i32::from(has_selection && self.target.as_ref().is_some_and(|t| t.validate().is_ok())),
        );
        self.status(&if self.results.is_empty() {
            self.label(
                "No results. Try Reset or another name.",
                "没有结果。请点重置或尝试其他名称。",
            )
            .into()
        } else {
            format!(
                "{} {} · {}",
                self.results.len(),
                self.label("results", "条结果"),
                self.settings.hotkey
            )
        });
    }
    unsafe fn selected(&self) -> Option<&Symbol> {
        let index = SendMessageW(self.get(GRID), LB_GETCURSEL, 0, 0);
        if index < 0 {
            None
        } else {
            self.results
                .get(index as usize)
                .and_then(|id| self.catalog.get(id))
        }
    }
    unsafe fn details(&self) {
        let Some(s) = self.selected() else {
            set_text(
                self.get(DETAILS),
                self.label("No symbol selected", "未选择字符"),
            );
            return;
        };
        set_text(
            self.get(DETAILS),
            &format!(
                "{}\r\n{} · {} · {}\r\n{}{}",
                s.name,
                s.codepoints.join(" "),
                s.languages.join(", "),
                s.license,
                if s.description == s.name {
                    ""
                } else {
                    &s.description
                },
                if s.examples.is_empty() {
                    String::new()
                } else {
                    format!("  e.g. {}", s.examples.join(" / "))
                }
            ),
        );
        let units: Vec<u16> = s.text.encode_utf16().collect();
        let dc = GetDC(self.window);
        if !dc.is_null() {
            let old = SelectObject(dc, self.symbol_font);
            let mut glyphs = vec![0u16; units.len()];
            let count = GetGlyphIndicesW(
                dc,
                units.as_ptr(),
                units.len() as i32,
                glyphs.as_mut_ptr(),
                GGI_MARK_NONEXISTING_GLYPHS,
            );
            SelectObject(dc, old);
            ReleaseDC(self.window, dc);
            if count == GDI_ERROR as u32
                || glyphs.contains(&0xffff)
                || s.text.chars().any(|c| c as u32 > 0xffff)
            {
                let mut value = text(self.get(DETAILS));
                value.push_str(self.label("\r\nFont coverage needs checking; use the codepoints to verify the glyph in the target.","\r\n需检查字体覆盖；请依据码位核对目标应用中的字形。"));
                set_text(self.get(DETAILS), &value);
            }
        }
        set_text(
            self.get(FAVORITE),
            if self.settings.favorites.contains(&s.id) {
                self.label("Unfavorite", "取消收藏")
            } else {
                self.label("&Favorite", "收藏(&F)")
            },
        );
    }
    unsafe fn owns_window(&self, window: HWND) -> bool {
        !window.is_null()
            && (window == self.window
                || window == self.settings_window
                || self.mapping.as_ref().is_some_and(|m| m.active(window)))
    }
    unsafe fn reveal(&self) {
        self.hide_auxiliary();
        ShowWindow(
            self.window,
            if IsIconic(self.window) != 0 {
                SW_RESTORE
            } else {
                SW_SHOW
            },
        );
        SetForegroundWindow(self.window);
        SetFocus(self.get(SEARCH));
        SendMessageW(self.get(SEARCH), 0x00b1, 0, -1);
    }
    unsafe fn show(&mut self) {
        let minimized = IsIconic(self.window) != 0;
        if !minimized
            && IsWindowVisible(self.window) != 0
            && self.owns_window(GetForegroundWindow())
        {
            self.hide();
            return;
        }
        self.target = Target::capture(self.window);
        // GetWindowRect on an iconic window describes the minimized icon, not
        // the palette. Capture the editor first, then restore before sizing it.
        if minimized {
            ShowWindow(self.window, SW_SHOWNOACTIVATE);
        }
        let mut point = zeroed();
        if GetCursorPos(&mut point) == 0 {
            ShowWindow(self.window, SW_SHOWNORMAL);
            self.status(self.label(
                "Interactive desktop unavailable. Unlock your session to use global input.",
                "当前交互桌面不可用，请解锁会话后使用全局输入。",
            ));
            return;
        }
        let monitor = MonitorFromPoint(point, MONITOR_DEFAULTTONEAREST);
        let mut info: MONITORINFO = zeroed();
        info.cbSize = size_of::<MONITORINFO>() as u32;
        if GetMonitorInfoW(monitor, &mut info) == 0 {
            ShowWindow(self.window, SW_SHOWNORMAL);
            self.status("Cannot query monitor work area; previous window position retained.");
            return;
        }
        let mut rect = zeroed();
        GetWindowRect(self.window, &mut rect);
        let width = (rect.right - rect.left).min(info.rcWork.right - info.rcWork.left);
        let height = (rect.bottom - rect.top).min(info.rcWork.bottom - info.rcWork.top);
        let x = (point.x + 12).clamp(info.rcWork.left, info.rcWork.right - width);
        let y = (point.y + 12).clamp(info.rcWork.top, info.rcWork.bottom - height);
        SetWindowPos(
            self.window,
            HWND_TOPMOST,
            x,
            y,
            width,
            height,
            SWP_SHOWWINDOW,
        );
        SetForegroundWindow(self.window);
        SetFocus(self.get(SEARCH));
        SendMessageW(self.get(SEARCH), 0x00b1, 0, -1); // EM_SETSEL
        self.refresh();
        if self.target.is_none() {
            self.status(self.label(
                "No external target captured. Use Copy or focus a target and recall.",
                "未记录外部目标。可复制，或聚焦目标后重新呼出。",
            ));
        }
    }
    unsafe fn hide_auxiliary(&self) {
        if let Some(mapping) = &self.mapping {
            ShowWindow(mapping.window, SW_HIDE);
        }
        if !self.settings_window.is_null() {
            ShowWindow(self.settings_window, SW_HIDE);
        }
    }
    unsafe fn hide(&self) {
        self.hide_auxiliary();
        ShowWindow(self.window, SW_HIDE);
        if let Some(target) = self.target.as_ref() {
            if target.validate().is_ok() {
                SetForegroundWindow(target.window);
            }
        }
    }
    unsafe fn act(&mut self, id: usize) {
        if id == RESET {
            SendMessageW(self.get(CATEGORY), CB_SETCURSEL, 0, 0);
            SendMessageW(self.get(LANGUAGE), CB_SETCURSEL, 0, 0);
            set_text(self.get(SEARCH), "");
            self.refresh();
            SetFocus(self.get(SEARCH));
            return;
        }
        if id == CLEAR {
            set_text(self.get(BUFFER), "");
            return;
        }
        let Some(s) = self.selected() else {
            self.status(self.label("Select a symbol first", "请先选择字符"));
            return;
        };
        let (symbol_id, selected) = (s.id.clone(), s.text.clone());
        if id == FAVORITE {
            self.settings.favorite(&symbol_id);
            self.persist();
            self.refresh();
            return;
        }
        if id == COMPOSE {
            let mut buffer = text(self.get(BUFFER));
            buffer.push_str(&selected);
            if buffer.len() > 4096 {
                self.status("Composition exceeds 4096 UTF-8 bytes");
                return;
            }
            set_text(self.get(BUFFER), &buffer);
            return;
        }
        let buffer = text(self.get(BUFFER));
        let value = if buffer.is_empty() { selected } else { buffer };
        let result = match id {
            COPY => copy(&value, self.window),
            INSERT => self
                .target
                .as_ref()
                .ok_or_else(|| {
                    self.label(
                        "No target captured. Use Copy, or focus a target and recall.",
                        "未记录目标。请复制，或聚焦目标后重新呼出。",
                    )
                    .to_string()
                })
                .and_then(|target| insert(target, &value, self.window)),
            STUDIO => self.open_studio(&value),
            _ => return,
        };
        match result {
            Ok(()) => {
                self.settings.used(&symbol_id);
                if let Err(e) = config::save(&self.config, &self.settings) {
                    self.status(&format!("Action succeeded; settings write failed: {e}"));
                    return;
                }
                self.status(match id {
                    COPY => self.label("Copied. Clipboard replaced.", "已复制，剪贴板已替换。"),
                    INSERT => self.label(
                        "Input sent. Check the target.",
                        "输入事件已发送，请检查目标。",
                    ),
                    _ => self.label("Sent to IPA Studio.", "已发送至 IPA Studio。"),
                });
            }
            Err(e) => {
                ShowWindow(self.window, SW_SHOW);
                SetForegroundWindow(self.window);
                self.status(&e);
            }
        }
    }
    unsafe fn open_studio(&mut self, value: &str) -> Result<(), String> {
        let executable = std::env::current_exe()
            .map_err(|e| e.to_string())?
            .with_file_name("ipa-tts.exe");
        if !executable.is_file() {
            return Err(self.label("IPA Studio is optional. Install the full package beside the palette; Copy is available.","IPA Studio 为可选组件。请使用完整包，当前仍可复制。").into());
        }
        if let Some(child) = &mut self.studio {
            if child.try_wait().map_err(|e| e.to_string())?.is_some() {
                self.studio = None;
                self.studio_port = None;
            }
        }
        if self.studio.is_none() {
            let listener =
                std::net::TcpListener::bind(("127.0.0.1", 0)).map_err(|e| e.to_string())?;
            let port = listener.local_addr().map_err(|e| e.to_string())?.port();
            drop(listener);
            use std::os::windows::process::CommandExt;
            let child = Command::new(&executable)
                .args(["serve", &port.to_string()])
                .current_dir(executable.parent().ok_or("No app directory")?)
                .creation_flags(0x08000000)
                .spawn()
                .map_err(|e| e.to_string())?;
            self.studio = Some(child);
            self.studio_port = Some(port);
        }
        let port = self.studio_port.ok_or("Studio has no port")?;
        let address = std::net::SocketAddr::from(([127, 0, 0, 1], port));
        let mut ready = false;
        for _ in 0..60 {
            if self
                .studio
                .as_mut()
                .ok_or("Studio process missing")?
                .try_wait()
                .map_err(|e| e.to_string())?
                .is_some()
            {
                self.studio = None;
                self.studio_port = None;
                return Err(
                    "IPA Studio exited during startup. Start it manually to inspect the error."
                        .into(),
                );
            }
            if std::net::TcpStream::connect_timeout(&address, std::time::Duration::from_millis(25))
                .is_ok()
            {
                ready = true;
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(25));
        }
        if !ready {
            return Err("IPA Studio did not become ready. Use Copy and start it manually.".into());
        }
        let url = format!("http://127.0.0.1:{}/#ipa={}", port, studio_fragment(value));
        if ShellExecuteW(
            self.window,
            wide("open").as_ptr(),
            wide(&url).as_ptr(),
            null(),
            null(),
            SW_SHOWNORMAL,
        ) as isize
            <= 32
        {
            return Err("Cannot open default browser. Use Copy and start Studio manually.".into());
        }
        Ok(())
    }
    unsafe fn settings(&mut self) -> Result<(), String> {
        if !self.settings_window.is_null() {
            ShowWindow(self.settings_window, SW_SHOW);
            SetForegroundWindow(self.settings_window);
            return Ok(());
        }
        self.settings_window = CreateWindowExW(
            WS_EX_TOOLWINDOW,
            wide("IPAFirstPalette").as_ptr(),
            wide(self.label("Palette settings", "面板设置")).as_ptr(),
            WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            460 * GetDpiForWindow(self.window).max(96) as i32 / 96,
            285 * GetDpiForWindow(self.window).max(96) as i32 / 96,
            self.window,
            null_mut(),
            GetModuleHandleW(null()),
            null(),
        );
        if self.settings_window.is_null() {
            return Err("Cannot create settings window".into());
        }
        for (id,class,title,style,x,y,w,h) in [
            (401,"STATIC",self.label("Global shortcut (Ctrl/Alt/Shift + key)","全局快捷键（Ctrl/Alt/Shift + 键）"),0,12,14,420,24),
            (HOTKEY_EDIT,"EDIT",self.settings.hotkey.as_str(),WS_BORDER|ES_AUTOHSCROLL as u32,12,42,420,30),
            (SAVE_SETTINGS,"BUTTON",self.label("Save shortcut","保存快捷键"),BS_PUSHBUTTON as u32,12,82,160,32),
            (402,"STATIC",self.label("Copy replaces the clipboard. Insert never changes it.\r\nNo automatic clipboard paste or autostart.\r\nSettings are stored in your local user directory.","复制将替换剪贴板；插入不改动剪贴板。\r\n无自动粘贴回退，无开机自启。\r\n设置保存在本地用户配置目录。"),0,12,126,420,90),
        ] {
            let control = create_control(self.settings_window,id,class,title,style)?;
            let scale=|n|n*GetDpiForWindow(self.settings_window).max(96) as i32/96;
            MoveWindow(control,scale(x),scale(y),scale(w),scale(h),1); SendMessageW(control,WM_SETFONT,self.font as usize,1);
            self.settings_controls.push((id,control));
        }
        ShowWindow(self.settings_window, SW_SHOW);
        SetFocus(
            self.settings_controls
                .iter()
                .find(|c| c.0 == HOTKEY_EDIT)
                .unwrap()
                .1,
        );
        Ok(())
    }
    unsafe fn mapping(&mut self) -> Result<(), String> {
        if let Some(mapping) = &mut self.mapping {
            mapping.localize(self.settings.ui_language == "zh-CN");
            mapping.show();
        } else {
            self.mapping = Some(MappingWindow::new(self.window, self.zh(), true)?);
        }
        Ok(())
    }
    unsafe fn save_hotkey(&mut self) -> Result<(), String> {
        let hotkey_text = text(
            self.settings_controls
                .iter()
                .find(|c| c.0 == HOTKEY_EDIT)
                .ok_or("No hotkey control")?
                .1,
        );
        let key = Hotkey::parse(&hotkey_text)?;
        let previous = Hotkey::parse(&self.settings.hotkey)?;
        if key != previous {
            // Reserve the new combination before releasing the old registration.
            let id = if self.registration.as_ref().is_some_and(|r| r.id == 2) {
                1
            } else {
                2
            };
            let reservation = Registration::new(self.window, id, key)?;
            let mut updated = self.settings.clone();
            updated.hotkey = hotkey_text;
            config::save(&self.config, &updated)?;
            self.registration.take();
            self.registration = Some(reservation);
            self.settings = updated;
        } else if self.registration.is_none() {
            self.registration = Some(Registration::new(self.window, HOTKEY_ID, key)?);
        }
        ShowWindow(self.settings_window, SW_HIDE);
        self.tray.szTip = tray_tip(&self.settings.hotkey);
        Shell_NotifyIconW(NIM_MODIFY, &self.tray);
        self.status(self.label("Shortcut saved", "快捷键已保存"));
        Ok(())
    }
    unsafe fn tray_menu(&mut self) {
        let menu = CreatePopupMenu();
        for (id, en, zh) in [
            (SHOW, "Show palette", "显示面板"),
            (HIDE, "Hide palette", "隐藏面板"),
            (STUDIO, "IPA Studio", "IPA Studio"),
            (SETTINGS, "Settings", "设置"),
            (MAPPING, "Language mapping", "语言映射"),
            (QUIT, "Exit", "退出"),
        ] {
            AppendMenuW(menu, MF_STRING, id, wide(self.label(en, zh)).as_ptr());
        }
        let mut point = zeroed();
        if GetCursorPos(&mut point) == 0 {
            // A keyboard-opened menu remains reachable without a cursor query.
            let mut rect = zeroed();
            if GetWindowRect(self.get(MENU), &mut rect) != 0 {
                point.x = rect.left;
                point.y = rect.bottom;
            }
        }
        SetForegroundWindow(self.window);
        let choice = TrackPopupMenu(
            menu,
            TPM_RETURNCMD | TPM_NONOTIFY | TPM_RIGHTBUTTON,
            point.x,
            point.y,
            0,
            self.window,
            null(),
        );
        DestroyMenu(menu);
        match choice as usize {
            SHOW => {
                self.reveal();
                self.target = None;
                self.target_hint();
            }
            HIDE => {
                self.hide();
            }
            STUDIO => {
                if let Err(e) = self.open_studio("") {
                    self.status(&e);
                    ShowWindow(self.window, SW_SHOW);
                }
            }
            SETTINGS => {
                if let Err(e) = self.settings() {
                    self.status(&e);
                }
            }
            MAPPING => {
                if let Err(e) = self.mapping() {
                    self.status(&e);
                }
            }
            QUIT => {
                PostQuitMessage(0);
            }
            _ => {}
        }
        PostMessageW(self.window, WM_NULL, 0, 0);
    }
}
unsafe fn create_control(
    parent: HWND,
    id: usize,
    class: &str,
    title: &str,
    style: u32,
) -> Result<HWND, String> {
    let hwnd = CreateWindowExW(
        0,
        wide(class).as_ptr(),
        wide(title).as_ptr(),
        WS_CHILD | WS_VISIBLE | if class == "STATIC" { 0 } else { WS_TABSTOP } | style,
        0,
        0,
        100,
        26,
        parent,
        id as HMENU,
        GetModuleHandleW(null()),
        null(),
    );
    if hwnd.is_null() {
        Err(format!(
            "Cannot create native {class}: {}",
            std::io::Error::last_os_error()
        ))
    } else {
        Ok(hwnd)
    }
}
unsafe fn combo(hwnd: HWND, values: &[&str]) {
    SendMessageW(hwnd, CB_RESETCONTENT, 0, 0);
    for value in values {
        SendMessageW(hwnd, CB_ADDSTRING, 0, wide(value).as_ptr() as isize);
    }
    SendMessageW(hwnd, CB_SETCURSEL, 0, 0);
}
pub fn run(catalog: Catalog, args: &[String]) -> Result<(), String> {
    // SAFETY: run_native confines Win32 calls and all UI lifetimes to this thread.
    unsafe { run_native(catalog, args) }
}
unsafe fn run_native(catalog: Catalog, args: &[String]) -> Result<(), String> {
    SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
    let common_controls = INITCOMMONCONTROLSEX {
        dwSize: size_of::<INITCOMMONCONTROLSEX>() as u32,
        dwICC: ICC_STANDARD_CLASSES,
    };
    if InitCommonControlsEx(&common_controls) == 0 {
        return Err("Cannot initialize Windows controls".into());
    }
    if args.first().is_some_and(|a| a == "--quit") {
        let existing = FindWindowW(wide("IPAFirstPalette").as_ptr(), null());
        if !existing.is_null() {
            PostMessageW(existing, EVENT, QUIT, 0);
        }
        return Ok(());
    }
    if args
        .first()
        .is_some_and(|a| !matches!(a.as_str(), "--config" | "--background"))
    {
        return Err("Usage: ipa-symbol-palette [--background | --config file | --check output.json | --quit]".into());
    }
    let config_path = if args.first().is_some_and(|a| a == "--config") {
        PathBuf::from(args.get(1).ok_or("--config requires a file path")?)
    } else {
        config::default_path()?
    };
    // Local namespace is session-scoped. Include a hash of the per-user config directory.
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    config::default_path()?.hash(&mut hasher);
    let instance = Handle(CreateMutexW(
        null(),
        0,
        wide(&format!("Local\\IPAFirstPalette-{:x}", hasher.finish())).as_ptr(),
    ));
    if instance.0.is_null() {
        return Err("Cannot create palette instance lock".into());
    }
    if GetLastError() == ERROR_ALREADY_EXISTS {
        let existing = FindWindowW(wide("IPAFirstPalette").as_ptr(), null());
        if !existing.is_null() {
            let mut process = 0;
            GetWindowThreadProcessId(existing, &mut process);
            // A user-launched second process may grant foreground permission to
            // the existing owner; it never grants permission to every process.
            // https://learn.microsoft.com/windows/win32/api/winuser/nf-winuser-allowsetforegroundwindow
            AllowSetForegroundWindow(process);
            PostMessageW(existing, EVENT, SHOW, 0);
            if let (Some(running), Ok(requested)) =
                (window_executable(existing), std::env::current_exe())
            {
                if running.canonicalize().ok() != requested.canonicalize().ok() {
                    MessageBoxW(existing, wide(&format!(
                        "A palette from another location is already running:\n{}\n\nExit that copy with Menu → Exit, then launch this copy.\n\n已有其他位置的面板正在运行。请在旧面板中选择菜单 → 退出，再启动当前副本。",
                        running.display()
                    )).as_ptr(), wide("IPA Symbol Palette · Existing copy / 已有实例").as_ptr(), MB_OK | MB_ICONINFORMATION);
                }
            }
            return Ok(());
        }
        return Err("Another palette instance is starting; try again shortly".into());
    }
    let (settings, recovery) = config::load(&config_path)?;
    let instance_module = GetModuleHandleW(null());
    let icon = palette_icon()?;
    let class = WNDCLASSW {
        lpfnWndProc: Some(procedure),
        hInstance: instance_module,
        hCursor: LoadCursorW(null_mut(), IDC_ARROW),
        hIcon: icon.0,
        hbrBackground: GetSysColorBrush(COLOR_BTNFACE),
        ..zeroed()
    };
    // Keep class name alive for RegisterClass; the class copies its registration data.
    let class_name = wide("IPAFirstPalette");
    let class = WNDCLASSW {
        lpszClassName: class_name.as_ptr(),
        ..class
    };
    if RegisterClassW(&class) == 0 {
        return Err(format!(
            "Cannot register palette window: {}",
            std::io::Error::last_os_error()
        ));
    }
    let hwnd = CreateWindowExW(
        WS_EX_APPWINDOW | WS_EX_TOPMOST,
        class_name.as_ptr(),
        wide("IPA Symbol Palette").as_ptr(),
        WS_OVERLAPPEDWINDOW & !WS_MAXIMIZEBOX,
        CW_USEDEFAULT,
        CW_USEDEFAULT,
        600 * GetDpiForSystem().max(96) as i32 / 96,
        740 * GetDpiForSystem().max(96) as i32 / 96,
        null_mut(),
        null_mut(),
        instance_module,
        null(),
    );
    if hwnd.is_null() {
        return Err("Cannot create palette window".into());
    }
    let mut app = App {
        window: hwnd,
        controls: vec![],
        catalog,
        settings,
        config: config_path,
        target: None,
        results: vec![],
        registration: None,
        tray: zeroed(),
        font: null_mut(),
        symbol_font: null_mut(),
        studio: None,
        studio_port: None,
        settings_window: null_mut(),
        settings_controls: vec![],
        mapping: None,
    };
    for (id, class, title, style) in [
        (SEARCH_LABEL, "STATIC", "Search", 0),
        (SEARCH, "EDIT", "", WS_BORDER | ES_AUTOHSCROLL as u32),
        (
            CATEGORY,
            "COMBOBOX",
            "Category",
            CBS_DROPDOWNLIST as u32 | WS_VSCROLL,
        ),
        (
            LANGUAGE,
            "COMBOBOX",
            "Language",
            CBS_DROPDOWNLIST as u32 | WS_VSCROLL,
        ),
        (
            GRID,
            "LISTBOX",
            "Symbols",
            WS_BORDER
                | WS_HSCROLL
                | LBS_NOTIFY as u32
                | LBS_MULTICOLUMN as u32
                | LBS_OWNERDRAWFIXED as u32
                | LBS_HASSTRINGS as u32
                | LBS_NOINTEGRALHEIGHT as u32,
        ),
        (
            DETAILS,
            "EDIT",
            "",
            WS_BORDER | ES_READONLY as u32 | ES_MULTILINE as u32 | WS_VSCROLL,
        ),
        (COMPOSE_LABEL, "STATIC", "Composition", 0),
        (BUFFER, "EDIT", "", WS_BORDER | ES_AUTOHSCROLL as u32),
        (COPY, "BUTTON", "Copy", 0),
        (INSERT, "BUTTON", "Insert", 0),
        (FAVORITE, "BUTTON", "Favorite", 0),
        (COMPOSE, "BUTTON", "Compose +", 0),
        (CLEAR, "BUTTON", "Clear", 0),
        (STUDIO, "BUTTON", "IPA Studio", 0),
        (MAPPING, "BUTTON", "Mapping", 0),
        (SETTINGS, "BUTTON", "Settings", 0),
        (RESET, "BUTTON", "Reset", 0),
        (MENU, "BUTTON", "Menu", 0),
        (QUIT, "BUTTON", "Exit", 0),
        (TARGET_LABEL, "STATIC", "", 0),
        (
            UI_LANGUAGE,
            "COMBOBOX",
            "Interface language",
            CBS_DROPDOWNLIST as u32,
        ),
        (ACTION, "COMBOBOX", "Enter action", CBS_DROPDOWNLIST as u32),
        (STATUS, "STATIC", "", 0),
    ] {
        app.child(id, class, title, style)?;
    }
    SendMessageW(app.get(SEARCH), 0x00c5, 256, 0);
    SendMessageW(app.get(BUFFER), 0x00c5, 1024, 0); // EM_SETLIMITTEXT
    let mut languages = vec!["All languages".to_string()];
    languages.extend(
        app.catalog
            .languages
            .iter()
            .map(|l| format!("{} · {}", l.id, l.native_name)),
    );
    combo(
        app.get(LANGUAGE),
        &languages.iter().map(String::as_str).collect::<Vec<_>>(),
    );
    combo(app.get(UI_LANGUAGE), &["English", "简体中文"]);
    SendMessageW(app.get(UI_LANGUAGE), CB_SETCURSEL, usize::from(app.zh()), 0);
    app.fonts();
    app.localize();
    app.layout();
    app.refresh();
    app.tray.cbSize = size_of::<NOTIFYICONDATAW>() as u32;
    app.tray.hWnd = hwnd;
    app.tray.uID = 1;
    app.tray.uFlags = NIF_MESSAGE | NIF_ICON | NIF_TIP;
    app.tray.uCallbackMessage = TRAY;
    app.tray.hIcon = icon.0;
    app.tray.szTip = tray_tip(&app.settings.hotkey);
    if Shell_NotifyIconW(NIM_ADD, &app.tray) == 0 {
        return Err("Cannot add tray icon".into());
    }
    match Registration::new(hwnd, HOTKEY_ID, Hotkey::parse(&app.settings.hotkey)?) {
        Ok(reg) => app.registration = Some(reg),
        Err(e) => {
            error(&e);
            app.status(&e);
        }
    }
    if !args.first().is_some_and(|a| a == "--background") {
        app.show();
    }
    if let Some(warning) = recovery {
        app.status(&warning);
    }
    let mut message: MSG = zeroed();
    loop {
        let status = GetMessageW(&mut message, null_mut(), 0, 0);
        if status == 0 {
            break;
        }
        if status == -1 {
            return Err("Windows message loop failed".into());
        }
        if message.message == EXPLORER_EVENT {
            if Shell_NotifyIconW(NIM_ADD, &app.tray) == 0 {
                app.status(app.label("Cannot restore tray icon", "无法恢复托盘图标"));
            }
            continue;
        }
        if message.message == WM_HOTKEY && matches!(message.wParam, 1 | 2) {
            app.show();
            continue;
        }
        if message.message == TRAY_EVENT {
            // NIM_SETVERSION is intentionally unused: legacy callbacks carry
            // the icon ID in wParam and the whole event code in lParam.
            if message.wParam == app.tray.uID as usize {
                match message.lParam as u32 {
                    WM_LBUTTONUP | NIN_SELECT | TRAY_KEYSELECT => {
                        app.reveal();
                        app.target = None;
                        app.target_hint();
                    }
                    WM_RBUTTONUP | WM_CONTEXTMENU => app.tray_menu(),
                    _ => {}
                }
            }
            continue;
        }
        if message.message == EVENT {
            let id = message.wParam & 0xffff;
            let notification = (message.wParam >> 16) as u32;
            if app
                .mapping
                .as_ref()
                .is_some_and(|m| m.window == message.hwnd)
            {
                let action = app.mapping.as_mut().unwrap().handle(id, notification);
                match action {
                    Ok(Some(MappingAction::Palette(value))) => {
                        app.act(RESET);
                        set_text(app.get(BUFFER), &value);
                        ShowWindow(app.mapping.as_ref().unwrap().window, SW_HIDE);
                        ShowWindow(app.window, SW_SHOW);
                        SetForegroundWindow(app.window);
                        SetFocus(app.get(BUFFER));
                        app.target_hint();
                        app.status(app.label(
                            "Mapping result is the composition. Insert remains explicit.",
                            "映射结果已置入组合框；插入仍需明确操作。",
                        ));
                    }
                    Ok(Some(MappingAction::Studio(value))) => {
                        if let Err(e) = app.open_studio(&value) {
                            error(&e);
                        }
                    }
                    Err(e) => error(&e),
                    _ => {}
                }
                continue;
            }
            if id == 300 {
                app.layout();
            } else if id == 301 {
                app.fonts();
                app.layout();
            } else if id == SHOW {
                app.reveal();
                app.target = None;
                app.target_hint();
            } else if id == HIDE {
                if message.hwnd == app.settings_window {
                    ShowWindow(app.settings_window, SW_HIDE);
                } else {
                    app.hide();
                }
            } else if id == QUIT {
                PostQuitMessage(0);
            } else if (id == SEARCH && notification == EN_CHANGE)
                || (matches!(id, CATEGORY | LANGUAGE) && notification == CBN_SELCHANGE)
            {
                app.refresh();
            } else if id == GRID && notification == LBN_SELCHANGE {
                app.details();
            } else if id == GRID && notification == LBN_DBLCLK {
                app.act(app.enter_action());
            } else if id == UI_LANGUAGE && notification == CBN_SELCHANGE {
                app.settings.ui_language =
                    if SendMessageW(app.get(UI_LANGUAGE), CB_GETCURSEL, 0, 0) == 1 {
                        "zh-CN"
                    } else {
                        "en"
                    }
                    .into();
                app.persist();
                app.localize();
                app.refresh();
            } else if id == ACTION && notification == CBN_SELCHANGE {
                app.settings.action = if SendMessageW(app.get(ACTION), CB_GETCURSEL, 0, 0) == 1 {
                    "copy"
                } else {
                    "insert"
                }
                .into();
                app.persist();
                app.target_hint();
            } else if id == SETTINGS {
                if let Err(e) = app.settings() {
                    error(&e);
                }
            } else if id == MAPPING {
                if let Err(e) = app.mapping() {
                    error(&e);
                }
            } else if id == MENU {
                app.tray_menu();
            } else if id == SAVE_SETTINGS {
                if let Err(e) = app.save_hotkey() {
                    error(&e);
                }
            } else if matches!(
                id,
                COPY | INSERT | FAVORITE | COMPOSE | CLEAR | STUDIO | RESET
            ) {
                app.act(id);
            }
            continue;
        }
        let settings_active = !app.settings_window.is_null()
            && (message.hwnd == app.settings_window
                || IsChild(app.settings_window, message.hwnd) != 0);
        let mapping_active = app.mapping.as_ref().is_some_and(|m| m.active(message.hwnd));
        if message.message == WM_KEYDOWN {
            if mapping_active
                && message.wParam == VK_RETURN as usize
                && GetKeyState(VK_CONTROL as i32) < 0
            {
                if let Err(e) = app
                    .mapping
                    .as_mut()
                    .unwrap()
                    .handle(mapping_window::MAP_CONVERT, 0)
                {
                    error(&e);
                }
                continue;
            }
            if !settings_active && !mapping_active && GetKeyState(VK_CONTROL as i32) < 0 {
                if message.wParam == b'F' as usize {
                    SetFocus(app.get(SEARCH));
                    SendMessageW(app.get(SEARCH), 0x00b1, 0, -1);
                    continue;
                }
                if message.wParam == b'D' as usize {
                    app.act(FAVORITE);
                    continue;
                }
            }
            if message.wParam == VK_ESCAPE as usize {
                if mapping_active {
                    ShowWindow(app.mapping.as_ref().unwrap().window, SW_HIDE);
                } else if settings_active {
                    ShowWindow(app.settings_window, SW_HIDE);
                } else {
                    app.hide();
                }
                continue;
            }
            if !settings_active && !mapping_active && message.wParam == VK_RETURN as usize {
                let focus = GetFocus();
                let button = app.controls.iter().find(|(_, c)| *c == focus).map(|p| p.0);
                if button == Some(SETTINGS) {
                    if let Err(e) = app.settings() {
                        error(&e);
                    }
                } else if button == Some(MAPPING) {
                    if let Err(e) = app.mapping() {
                        error(&e);
                    }
                } else if button == Some(MENU) {
                    app.tray_menu();
                } else if button == Some(QUIT) {
                    PostQuitMessage(0);
                } else if button.is_some_and(|id| {
                    matches!(
                        id,
                        COPY | INSERT | FAVORITE | COMPOSE | CLEAR | STUDIO | RESET
                    )
                }) {
                    app.act(button.unwrap());
                } else {
                    app.act(if GetKeyState(VK_SHIFT as i32) < 0 {
                        COPY
                    } else {
                        app.enter_action()
                    });
                }
                continue;
            }
            if !settings_active
                && !mapping_active
                && message.hwnd == app.get(SEARCH)
                && matches!(message.wParam, 0x25..=0x28)
            {
                SendMessageW(app.get(GRID), WM_KEYDOWN, message.wParam, message.lParam);
                app.details();
                continue;
            }
        }
        let dialog = if mapping_active {
            app.mapping.as_ref().unwrap().window
        } else if settings_active {
            app.settings_window
        } else {
            hwnd
        };
        if IsDialogMessageW(dialog, &message) == 0 {
            TranslateMessage(&message);
            DispatchMessageW(&message);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_sent_and_posted_shell_callbacks_reach_the_queue() {
        // Hidden test-owned window; no tray icon, foreground, input or shell
        // restart. Exercise real Win32 delivery, including nonqueued messages.
        unsafe {
            let module = GetModuleHandleW(null());
            let name = wide(&format!("IPAFirstTrayDeliveryTest-{}", std::process::id()));
            let class = WNDCLASSW {
                lpfnWndProc: Some(procedure),
                hInstance: module,
                lpszClassName: name.as_ptr(),
                ..zeroed()
            };
            assert_ne!(RegisterClassW(&class), 0);
            let window = CreateWindowExW(
                0,
                name.as_ptr(),
                wide("Shell delivery test").as_ptr(),
                0,
                0,
                0,
                20,
                20,
                null_mut(),
                null_mut(),
                module,
                null(),
            );
            assert!(!window.is_null());
            let mut message: MSG = zeroed();
            for event in [
                WM_LBUTTONUP,
                WM_RBUTTONUP,
                WM_CONTEXTMENU,
                NIN_SELECT,
                NIN_SELECT | NINF_KEY,
            ] {
                SendMessageW(window, TRAY, 1, event as isize);
                assert_ne!(
                    PeekMessageW(&mut message, window, TRAY_EVENT, TRAY_EVENT, PM_REMOVE),
                    0,
                    "nonqueued shell callback {event:#x} was lost",
                );
                assert_eq!((message.wParam, message.lParam), (1, event as isize));
                assert_eq!(
                    PeekMessageW(&mut message, window, TRAY_EVENT, TRAY_EVENT, PM_REMOVE),
                    0
                );
            }
            assert_ne!(PostMessageW(window, TRAY, 1, WM_RBUTTONUP as isize), 0);
            assert_ne!(PeekMessageW(&mut message, window, TRAY, TRAY, PM_REMOVE), 0);
            DispatchMessageW(&message);
            assert_ne!(
                PeekMessageW(&mut message, window, TRAY_EVENT, TRAY_EVENT, PM_REMOVE),
                0
            );
            assert_eq!((message.wParam, message.lParam), (1, WM_RBUTTONUP as isize));
            let explorer = RegisterWindowMessageW(wide("TaskbarCreated").as_ptr());
            assert_ne!(explorer, 0);
            SendMessageW(window, explorer, 0, 0);
            assert_ne!(
                PeekMessageW(
                    &mut message,
                    window,
                    EXPLORER_EVENT,
                    EXPLORER_EVENT,
                    PM_REMOVE
                ),
                0
            );
            assert_ne!(DestroyWindow(window), 0);
            assert_ne!(UnregisterClassW(name.as_ptr(), module), 0);
        }
    }
    #[test]
    fn native_registration_conflict_and_cleanup() {
        // Rare shortcut reserved only for the duration of this test, no UI input.
        unsafe {
            let key = Hotkey::parse("Ctrl+Alt+Shift+F24").unwrap();
            let first = Registration::new(null_mut(), 901, key).unwrap();
            assert!(Registration::new(null_mut(), 902, key).is_err());
            drop(first);
            let next = Registration::new(null_mut(), 903, key).unwrap();
            drop(next);
        }
    }
    #[test]
    fn native_instance_mutex_excludes_duplicates_and_releases() {
        // An isolated session-local test name; never signals a live application.
        unsafe {
            let name = wide(&format!(
                "Local\\IPAFirstPalette-Mutex-Test-{}",
                std::process::id()
            ));
            let first = Handle(CreateMutexW(null(), 0, name.as_ptr()));
            assert!(!first.0.is_null());
            assert_ne!(GetLastError(), ERROR_ALREADY_EXISTS);
            let second = Handle(CreateMutexW(null(), 0, name.as_ptr()));
            assert!(!second.0.is_null());
            assert_eq!(GetLastError(), ERROR_ALREADY_EXISTS);
            drop(second);
            drop(first);
            let restarted = Handle(CreateMutexW(null(), 0, name.as_ptr()));
            assert!(!restarted.0.is_null());
            assert_ne!(GetLastError(), ERROR_ALREADY_EXISTS);
        }
    }
    #[test]
    fn invalid_target_and_packet_flags() {
        unsafe {
            assert!(Target {
                window: null_mut(),
                process: 0,
                thread: 0,
                stamp: 0
            }
            .validate()
            .is_err());
        }
        let events = packets("t͡ʃ\u{1df00}");
        assert_eq!(events.len(), 10);
        unsafe {
            assert_eq!(events[0].Anonymous.ki.wVk, 0);
            assert_eq!(
                events[1].Anonymous.ki.dwFlags,
                KEYEVENTF_UNICODE | KEYEVENTF_KEYUP
            );
        }
    }
    #[test]
    fn destroyed_or_replaced_window_identity_is_rejected() {
        // Own, hidden STATIC test window: no focus, injection or clipboard access.
        unsafe {
            let window = CreateWindowExW(
                0,
                wide("STATIC").as_ptr(),
                wide("Palette identity test").as_ptr(),
                0,
                0,
                0,
                20,
                20,
                null_mut(),
                null_mut(),
                GetModuleHandleW(null()),
                null(),
            );
            assert!(!window.is_null());
            assert_eq!(
                window_executable(window).unwrap().canonicalize().unwrap(),
                std::env::current_exe().unwrap().canonicalize().unwrap()
            );
            let mut process = 0;
            let thread = GetWindowThreadProcessId(window, &mut process);
            assert_ne!(
                SetPropW(window, target_property().as_ptr(), 7usize as HANDLE),
                0
            );
            let target = Target {
                window,
                process,
                thread,
                stamp: 7,
            };
            assert!(target.validate().is_ok());
            SetPropW(window, target_property().as_ptr(), 8usize as HANDLE);
            assert!(target.validate().is_err());
            SetPropW(window, target_property().as_ptr(), 7usize as HANDLE);
            DestroyWindow(window);
            assert!(target.validate().is_err());
        }
    }
    #[test]
    fn native_controls_resize_filter_and_persist_without_external_input() {
        // Hidden test-owned controls only. No foreground changes, input injection,
        // clipboard writes or interaction with the user's running app.
        unsafe {
            let window = CreateWindowExW(
                0,
                wide("STATIC").as_ptr(),
                wide("Palette control test").as_ptr(),
                WS_POPUP,
                0,
                0,
                600,
                700,
                null_mut(),
                null_mut(),
                GetModuleHandleW(null()),
                null(),
            );
            assert!(!window.is_null());
            let directory = std::env::temp_dir().join(format!(
                "ipa-native-controls-{}-{:?}",
                std::process::id(),
                std::thread::current().id()
            ));
            std::fs::create_dir(&directory).unwrap();
            let mut app = App {
                window,
                controls: vec![],
                catalog: Catalog::bundled().unwrap(),
                settings: Settings::default(),
                config: directory.join("palette.json"),
                target: None,
                results: vec![],
                registration: None,
                tray: zeroed(),
                font: null_mut(),
                symbol_font: null_mut(),
                studio: None,
                studio_port: None,
                settings_window: null_mut(),
                settings_controls: vec![],
                mapping: None,
            };
            for id in [SEARCH_LABEL, TARGET_LABEL, COMPOSE_LABEL, STATUS] {
                app.child(id, "STATIC", "", 0).unwrap();
            }
            for id in [SEARCH, DETAILS, BUFFER] {
                app.child(id, "EDIT", "", ES_AUTOHSCROLL as u32).unwrap();
            }
            for id in [CATEGORY, LANGUAGE, UI_LANGUAGE, ACTION] {
                app.child(id, "COMBOBOX", "", CBS_DROPDOWNLIST as u32)
                    .unwrap();
            }
            for id in [
                COPY, INSERT, FAVORITE, COMPOSE, CLEAR, STUDIO, SETTINGS, RESET, MENU, MAPPING,
                QUIT,
            ] {
                app.child(id, "BUTTON", "", 0).unwrap();
            }
            app.child(
                GRID,
                "LISTBOX",
                "",
                LBS_MULTICOLUMN as u32
                    | LBS_OWNERDRAWFIXED as u32
                    | LBS_HASSTRINGS as u32
                    | LBS_NOINTEGRALHEIGHT as u32,
            )
            .unwrap();
            app.fonts();
            app.localize();
            app.refresh();
            assert_eq!(app.results.len(), 368);
            assert_eq!(app.enter_action(), COPY);
            assert_eq!(IsWindowEnabled(app.get(INSERT)), 0);
            let dpi = GetDpiForWindow(window).max(96) as i32;
            for (width, height) in [(400, 560), (576, 700), (900, 900)] {
                SetWindowPos(
                    window,
                    null_mut(),
                    0,
                    0,
                    width * dpi / 96,
                    height * dpi / 96,
                    SWP_NOACTIVATE | SWP_NOZORDER,
                );
                app.layout();
                let mut client = zeroed();
                GetClientRect(window, &mut client);
                for id in [
                    SEARCH,
                    GRID,
                    DETAILS,
                    BUFFER,
                    COPY,
                    INSERT,
                    FAVORITE,
                    COMPOSE,
                    STUDIO,
                    SETTINGS,
                    MENU,
                    MAPPING,
                    QUIT,
                    UI_LANGUAGE,
                    STATUS,
                ] {
                    let mut rect: RECT = zeroed();
                    assert_ne!(GetWindowRect(app.get(id), &mut rect), 0);
                    let mut start = POINT {
                        x: rect.left,
                        y: rect.top,
                    };
                    let mut end = POINT {
                        x: rect.right,
                        y: rect.bottom,
                    };
                    ScreenToClient(window, &mut start);
                    ScreenToClient(window, &mut end);
                    assert!(
                        start.x >= 0
                            && start.y >= 0
                            && end.x <= client.right
                            && end.y <= client.bottom,
                        "control {id}, {width}x{height}"
                    );
                }
                let mut grid: RECT = zeroed();
                GetClientRect(app.get(GRID), &mut grid);
                let row = SendMessageW(app.get(GRID), LB_GETITEMHEIGHT, 0, 0) as i32;
                assert!(grid.bottom / row >= 3, "native grid must show >=3 rows");
            }
            set_text(app.get(SEARCH), "tsh");
            app.refresh();
            assert_eq!(app.selected().unwrap().text, "t͡ʃ");
            app.act(FAVORITE);
            app.settings.used("combinations-0074-0361-0283");
            app.persist();
            let persisted = config::load(&app.config).unwrap().0;
            assert_eq!(persisted.favorites.len(), 1);
            assert_eq!(persisted.recent.len(), 1);
            app.act(COMPOSE);
            assert_eq!(text(app.get(BUFFER)), "t͡ʃ");
            set_text(app.get(SEARCH), "no-result-999");
            app.refresh();
            assert!(app.results.is_empty());
            assert_eq!(IsWindowEnabled(app.get(COPY)), 0);
            app.act(RESET);
            assert_eq!(app.results.len(), 368);
            app.settings.ui_language = "zh-CN".into();
            app.localize();
            app.refresh();
            assert!(text(app.get(SEARCH_LABEL)).contains("搜索"));
            assert!(text(app.window).contains(env!("CARGO_PKG_VERSION")));
            assert_eq!(text(app.get(QUIT)), "退出");
            let tip = tray_tip("Ctrl+Alt+Shift+F6");
            let tip = String::from_utf16_lossy(&tip[..tip.iter().position(|c| *c == 0).unwrap()]);
            assert!(tip.contains("Ctrl+Alt+Shift+F6"));
            assert!(!tip.contains("Ctrl+Alt+Shift+I"));
            assert_eq!(text(app.get(BUFFER)), "t͡ʃ");
            drop(app);
            assert_eq!(
                IsWindow(window),
                0,
                "owned native window survives App cleanup"
            );
            // Exclusively-created test directory; no user configuration is touched.
            std::fs::remove_dir_all(directory).unwrap();
        }
    }
}
