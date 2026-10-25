// SPDX-License-Identifier: GPL-3.0-or-later
//! Native mapping UI owns its controls; conversions run off the window thread.
use super::*;
use language_mapping::{
    espeak::EspeakG2p, Candidate, ConversionCapabilities, MappingRequest, MappingResult,
    Representation,
};
use std::sync::{mpsc, Arc};
pub const MAP_LANGUAGE: usize = 501;
pub const MAP_SCHEME: usize = 502;
const MAP_SCRIPT: usize = 503;
pub const MAP_INPUT: usize = 504;
const MAP_CONTEXT: usize = 505;
const MAP_RESULTS: usize = 506;
const MAP_DETAILS: usize = 507;
pub const MAP_CONVERT: usize = 508;
const MAP_STATUS: usize = 509;
const MAP_COPY: usize = 510;
const MAP_PALETTE: usize = 511;
const MAP_STUDIO: usize = 512;
const MAP_CLOSE: usize = 513;
pub const MAP_POLL: usize = 519;
pub enum MappingAction {
    Palette(String),
    Studio(String),
}
pub struct MappingWindow {
    pub window: HWND,
    controls: Vec<(usize, HWND)>,
    engine: Arc<language_mapping::Engine>,
    provider_path: PathBuf,
    capabilities: Vec<ConversionCapabilities>,
    scheme_ids: Vec<String>,
    result: Option<MappingResult>,
    receiver: Option<mpsc::Receiver<MappingResult>>,
    worker: Option<std::thread::JoinHandle<()>>,
    font: HFONT,
    zh: bool,
}
impl Drop for MappingWindow {
    fn drop(&mut self) {
        unsafe {
            KillTimer(self.window, MAP_POLL);
        }
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
        unsafe {
            if IsWindow(self.window) != 0 {
                DestroyWindow(self.window);
            }
            DeleteObject(self.font);
        }
    }
}
impl MappingWindow {
    fn label<'a>(&self, en: &'a str, zh: &'a str) -> &'a str {
        if self.zh {
            zh
        } else {
            en
        }
    }
    pub fn get(&self, id: usize) -> HWND {
        self.controls
            .iter()
            .find(|c| c.0 == id)
            .map(|c| c.1)
            .unwrap_or(null_mut())
    }
    pub unsafe fn new(parent: HWND, zh: bool, visible: bool) -> Result<Self, String> {
        let provider = EspeakG2p::discover();
        let engine = Arc::new(language_mapping::Engine::bundled()?);
        let capabilities = engine.capabilities(Some(&provider));
        let dpi = GetDpiForWindow(parent).max(96) as i32;
        let window = CreateWindowExW(
            WS_EX_APPWINDOW,
            wide("IPAFirstPalette").as_ptr(),
            wide(if zh {
                "语言映射"
            } else {
                "Language mapping"
            })
            .as_ptr(),
            WS_OVERLAPPEDWINDOW,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            720 * dpi / 96,
            800 * dpi / 96,
            parent,
            null_mut(),
            GetModuleHandleW(null()),
            null(),
        );
        if window.is_null() {
            return Err("Cannot create language mapping window".into());
        }
        let mut ui = Self {
            window,
            controls: vec![],
            engine,
            provider_path: provider.executable,
            capabilities,
            scheme_ids: vec![],
            result: None,
            receiver: None,
            worker: None,
            font: null_mut(),
            zh,
        };
        for (id, class, en, zh, style) in [
            (
                530,
                "STATIC",
                "Language / target stays the same",
                "语言 / 目标语言相同，不翻译",
                0,
            ),
            (MAP_LANGUAGE, "COMBOBOX", "", "", CBS_DROPDOWNLIST as u32),
            (
                531,
                "STATIC",
                "Conversion and named scheme",
                "转换方向与具名方案",
                0,
            ),
            (MAP_SCHEME, "COMBOBOX", "", "", CBS_DROPDOWNLIST as u32),
            (532, "STATIC", "Source script", "源文字系统", 0),
            (MAP_SCRIPT, "COMBOBOX", "", "", CBS_DROPDOWNLIST as u32),
            (
                533,
                "STATIC",
                "Input / original text is retained",
                "输入 / 保留原始文本",
                0,
            ),
            (
                MAP_INPUT,
                "EDIT",
                "",
                "",
                WS_BORDER
                    | WS_VSCROLL
                    | ES_MULTILINE as u32
                    | ES_AUTOVSCROLL as u32
                    | ES_WANTRETURN as u32,
            ),
            (
                534,
                "STATIC",
                "Context (optional; no general sense model)",
                "语境（可选；无通用词义消歧模型）",
                0,
            ),
            (
                MAP_CONTEXT,
                "EDIT",
                "",
                "",
                WS_BORDER | ES_AUTOHSCROLL as u32,
            ),
            (
                MAP_CONVERT,
                "BUTTON",
                "Convert (Ctrl+Enter)",
                "转换 (Ctrl+Enter)",
                BS_DEFPUSHBUTTON as u32,
            ),
            (
                MAP_RESULTS,
                "LISTBOX",
                "",
                "",
                WS_BORDER | WS_VSCROLL | LBS_NOTIFY as u32 | LBS_NOINTEGRALHEIGHT as u32,
            ),
            (
                MAP_DETAILS,
                "EDIT",
                "",
                "",
                WS_BORDER
                    | WS_VSCROLL
                    | ES_MULTILINE as u32
                    | ES_AUTOVSCROLL as u32
                    | ES_READONLY as u32,
            ),
            (MAP_COPY, "BUTTON", "Copy", "复制", 0),
            (MAP_PALETTE, "BUTTON", "Palette", "送入面板", 0),
            (MAP_STUDIO, "BUTTON", "IPA Studio", "IPA Studio", 0),
            (MAP_CLOSE, "BUTTON", "Close", "关闭", 0),
            (
                MAP_STATUS,
                "STATIC",
                "Choose a language and an available scheme. No silent fallback.",
                "选择语言和可用方案；失败时不静默切换。",
                0,
            ),
        ] {
            let control = create_control(window, id, class, if ui.zh { zh } else { en }, style)?;
            ui.controls.push((id, control));
        }
        SendMessageW(ui.get(MAP_INPUT), 0x00c5, 4096, 0);
        SendMessageW(ui.get(MAP_CONTEXT), 0x00c5, 4096, 0);
        for language in &ui.engine.registry().languages {
            SendMessageW(
                ui.get(MAP_LANGUAGE),
                CB_ADDSTRING,
                0,
                wide(&format!(
                    "{} / {} ({})",
                    language.name, language.native_name, language.tag
                ))
                .as_ptr() as isize,
            );
        }
        SendMessageW(ui.get(MAP_LANGUAGE), CB_SETCURSEL, 0, 0);
        ui.fonts();
        ui.language_changed();
        ui.layout();
        if visible {
            ui.show();
        }
        Ok(ui)
    }
    pub unsafe fn show(&self) {
        ShowWindow(self.window, SW_SHOW);
        SetForegroundWindow(self.window);
        SetFocus(self.get(MAP_INPUT));
    }
    pub unsafe fn localize(&mut self, zh: bool) {
        self.zh = zh;
        set_text(self.window, self.label("Language mapping", "语言映射"));
        for (id, en, zh) in [
            (
                530,
                "Language / target stays the same",
                "语言 / 目标语言相同，不翻译",
            ),
            (531, "Conversion and named scheme", "转换方向与具名方案"),
            (532, "Source script", "源文字系统"),
            (
                533,
                "Input / original text is retained",
                "输入 / 保留原始文本",
            ),
            (
                534,
                "Context (optional; no general sense model)",
                "语境（可选；无通用词义消歧模型）",
            ),
            (MAP_CONVERT, "Convert (Ctrl+Enter)", "转换 (Ctrl+Enter)"),
            (MAP_COPY, "Copy", "复制"),
            (MAP_PALETTE, "Palette", "送入面板"),
            (MAP_CLOSE, "Close", "关闭"),
        ] {
            set_text(self.get(id), self.label(en, zh));
        }
    }
    pub unsafe fn active(&self, hwnd: HWND) -> bool {
        hwnd == self.window || IsChild(self.window, hwnd) != 0
    }
    unsafe fn fonts(&mut self) {
        let dpi = GetDpiForWindow(self.window).max(96) as i32;
        let font = CreateFontW(
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
            0,
            0,
            wide("Segoe UI").as_ptr(),
        );
        for (_, control) in &self.controls {
            SendMessageW(*control, WM_SETFONT, font as usize, 1);
        }
        let previous = self.font;
        self.font = font;
        DeleteObject(previous);
    }
    pub unsafe fn layout(&self) {
        let mut area = zeroed();
        GetClientRect(self.window, &mut area);
        let dpi = GetDpiForWindow(self.window).max(96) as i32;
        let width = area.right * 96 / dpi;
        let height = area.bottom * 96 / dpi;
        let place = |id, x, y, w, h| {
            MoveWindow(
                self.get(id),
                x * dpi / 96,
                y * dpi / 96,
                w * dpi / 96,
                h * dpi / 96,
                1,
            );
        };
        place(530, 12, 8, width - 24, 20);
        place(MAP_LANGUAGE, 12, 30, width - 24, 220);
        place(531, 12, 62, width - 24, 20);
        place(MAP_SCHEME, 12, 84, width - 24, 260);
        place(532, 12, 116, width - 24, 20);
        place(MAP_SCRIPT, 12, 138, width - 24, 180);
        place(533, 12, 170, width - 24, 20);
        let input_height = (height / 6).clamp(64, 120);
        let context_y = 194 + input_height;
        place(MAP_INPUT, 12, 192, width - 24, input_height);
        place(534, 12, context_y, width - 24, 20);
        place(MAP_CONTEXT, 12, context_y + 22, width - 24, 27);
        place(MAP_CONVERT, 12, context_y + 55, 200, 30);
        place(MAP_RESULTS, 12, context_y + 93, width - 24, 60);
        let details_y = context_y + 161;
        place(
            MAP_DETAILS,
            12,
            details_y,
            width - 24,
            (height - 100 - details_y).max(20),
        );
        let button = (width - 48) / 4;
        for (index, id) in [MAP_COPY, MAP_PALETTE, MAP_STUDIO, MAP_CLOSE]
            .iter()
            .enumerate()
        {
            place(
                *id,
                12 + index as i32 * (button + 8),
                height - 88,
                button,
                30,
            );
        }
        place(MAP_STATUS, 12, height - 50, width - 24, 42);
    }
    fn language(&self) -> &str {
        let index =
            unsafe { SendMessageW(self.get(MAP_LANGUAGE), CB_GETCURSEL, 0, 0) }.max(0) as usize;
        &self.engine.registry().languages[index.min(self.engine.registry().languages.len() - 1)].tag
    }
    fn scheme(&self) -> Option<&language_mapping::Scheme> {
        let index = unsafe { SendMessageW(self.get(MAP_SCHEME), CB_GETCURSEL, 0, 0) };
        if index < 0 {
            return None;
        }
        let id = self.scheme_ids.get(index as usize)?;
        self.engine.registry().schemes.iter().find(|s| s.id == *id)
    }
    unsafe fn clear(&mut self) {
        self.result = None;
        SendMessageW(self.get(MAP_RESULTS), LB_RESETCONTENT, 0, 0);
        for id in [MAP_COPY, MAP_PALETTE, MAP_STUDIO] {
            EnableWindow(self.get(id), 0);
        }
        ShowWindow(self.get(MAP_STUDIO), SW_HIDE);
    }
    unsafe fn language_changed(&mut self) {
        self.clear();
        let language = self.language().to_string();
        self.scheme_ids.clear();
        SendMessageW(self.get(MAP_SCHEME), CB_RESETCONTENT, 0, 0);
        for capability in self
            .capabilities
            .iter()
            .filter(|c| c.available && c.scheme.language == language)
        {
            self.scheme_ids.push(capability.scheme.id.clone());
            let s = &capability.scheme;
            let title = format!("{:?} → {:?}: {}", s.input_type, s.output_type, s.name);
            SendMessageW(
                self.get(MAP_SCHEME),
                CB_ADDSTRING,
                0,
                wide(&title).as_ptr() as isize,
            );
        }
        SendMessageW(self.get(MAP_SCHEME), CB_SETCURSEL, 0, 0);
        self.scheme_changed();
        if self
            .capabilities
            .iter()
            .any(|c| !c.available && c.scheme.language == language)
        {
            set_text(
                self.get(MAP_STATUS),
                self.label(
                    "General G2P requires optional eSpeak/full package; available edges only.",
                    "通用 G2P 需要可选 eSpeak / 完整包；这里只列可用方向。",
                ),
            );
        }
    }
    unsafe fn scheme_changed(&mut self) {
        self.clear();
        SendMessageW(self.get(MAP_SCRIPT), CB_RESETCONTENT, 0, 0);
        if let Some(scheme) = self.scheme() {
            for script in &scheme.input_scripts {
                SendMessageW(
                    self.get(MAP_SCRIPT),
                    CB_ADDSTRING,
                    0,
                    wide(script).as_ptr() as isize,
                );
            }
            set_text(self.get(MAP_DETAILS), &scheme.limitations);
            EnableWindow(self.get(MAP_CONVERT), 1);
        } else {
            EnableWindow(self.get(MAP_CONVERT), 0);
        }
        SendMessageW(self.get(MAP_SCRIPT), CB_SETCURSEL, 0, 0);
    }
    unsafe fn convert(&mut self) -> Result<(), String> {
        if self.worker.is_some() {
            return Ok(());
        }
        let scheme = self.scheme().ok_or("Select an available scheme")?;
        let script_index = SendMessageW(self.get(MAP_SCRIPT), CB_GETCURSEL, 0, 0).max(0) as usize;
        let request = MappingRequest {
            language: self.language().into(),
            input: text(self.get(MAP_INPUT)),
            input_type: scheme.input_type,
            target: scheme.output_type,
            scheme: Some(scheme.id.clone()),
            source_script: scheme.input_scripts.get(script_index).cloned(),
            target_language: Some(self.language().into()),
            context: Some(text(self.get(MAP_CONTEXT))),
        };
        if SetTimer(self.window, MAP_POLL, 50, None) == 0 {
            return Err("Cannot start mapping completion timer".into());
        }
        self.clear();
        for id in [
            MAP_LANGUAGE,
            MAP_SCHEME,
            MAP_SCRIPT,
            MAP_INPUT,
            MAP_CONTEXT,
            MAP_CONVERT,
        ] {
            EnableWindow(self.get(id), 0);
        }
        set_text(
            self.get(MAP_STATUS),
            self.label("Converting locally…", "正在本地转换…"),
        );
        let engine = self.engine.clone();
        let path = self.provider_path.clone();
        let (sender, receiver) = mpsc::channel();
        self.receiver = Some(receiver);
        self.worker = Some(std::thread::spawn(move || {
            let provider = EspeakG2p { executable: path };
            let result = engine.convert(&request, Some(&provider));
            let _ = sender.send(result);
        }));
        Ok(())
    }
    fn selected(&self) -> Option<&Candidate> {
        let index = unsafe { SendMessageW(self.get(MAP_RESULTS), LB_GETCURSEL, 0, 0) };
        self.result.as_ref()?.candidates.get(index.max(0) as usize)
    }
    unsafe fn selected_changed(&self) {
        let candidate = self.selected();
        for id in [MAP_COPY, MAP_PALETTE] {
            EnableWindow(self.get(id), candidate.is_some() as i32);
        }
        let studio = candidate.is_some_and(|c| c.ipa_grammar_accepted == Some(true))
            && self
                .result
                .as_ref()
                .is_some_and(|r| r.output_type == Representation::Ipa && r.language == "en-US");
        EnableWindow(self.get(MAP_STUDIO), studio as i32);
        ShowWindow(self.get(MAP_STUDIO), if studio { SW_SHOW } else { SW_HIDE });
        if let Some(result) = &self.result {
            let mut details = result.warnings.join("\r\n\r\n");
            for source in &result.sources {
                details.push_str(&format!(
                    "\r\n\r\n{}\r\n{}\r\n{}\r\n{}",
                    source.title, source.version, source.license, source.url
                ));
            }
            if let Some(candidate) = candidate {
                details.push_str(&format!(
                    "\r\n\r\n{}\r\n{}",
                    candidate.transcription, candidate.ranking_basis
                ));
                for trace in &candidate.traces {
                    details.push_str(&format!(
                        "\r\n{}: {} → {} ({})",
                        trace.rule, trace.input, trace.output, trace.reference
                    ));
                }
            }
            set_text(self.get(MAP_DETAILS), &details);
        }
    }
    unsafe fn poll(&mut self) {
        let received = self.receiver.as_ref().map(|r| r.try_recv());
        let Some(received) = received else {
            return;
        };
        let result = match received {
            Ok(r) => Some(r),
            Err(mpsc::TryRecvError::Empty) => return,
            Err(mpsc::TryRecvError::Disconnected) => None,
        };
        KillTimer(self.window, MAP_POLL);
        self.receiver = None;
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
        for id in [
            MAP_LANGUAGE,
            MAP_SCHEME,
            MAP_SCRIPT,
            MAP_INPUT,
            MAP_CONTEXT,
            MAP_CONVERT,
        ] {
            EnableWindow(self.get(id), 1);
        }
        if let Some(result) = result {
            set_text(
                self.get(MAP_STATUS),
                &result.unsupported_reason.clone().unwrap_or_else(|| {
                    format!(
                        "{} candidates / 候选 · {}",
                        result.candidates.len(),
                        if result.ambiguous {
                            "ambiguous / 非唯一"
                        } else {
                            "determined within scheme / 方案内确定"
                        }
                    )
                }),
            );
            for candidate in &result.candidates {
                SendMessageW(
                    self.get(MAP_RESULTS),
                    LB_ADDSTRING,
                    0,
                    wide(&candidate.text).as_ptr() as isize,
                );
            }
            self.result = Some(result);
            SendMessageW(self.get(MAP_RESULTS), LB_SETCURSEL, 0, 0);
            self.selected_changed();
        } else {
            set_text(
                self.get(MAP_STATUS),
                "Mapping worker failed; original input is retained.",
            );
        }
    }
    pub unsafe fn handle(
        &mut self,
        id: usize,
        notification: u32,
    ) -> Result<Option<MappingAction>, String> {
        match id {
            300 => self.layout(),
            301 => {
                self.fonts();
                self.layout();
            }
            HIDE | MAP_CLOSE => {
                ShowWindow(self.window, SW_HIDE);
            }
            MAP_LANGUAGE if notification == CBN_SELCHANGE => self.language_changed(),
            MAP_SCHEME if notification == CBN_SELCHANGE => self.scheme_changed(),
            MAP_INPUT | MAP_CONTEXT if notification == EN_CHANGE => {
                if self.worker.is_none() {
                    self.clear();
                    set_text(
                        self.get(MAP_STATUS),
                        self.label(
                            "Input changed. Convert to refresh results.",
                            "输入已变化，请重新转换。",
                        ),
                    );
                }
            }
            MAP_SCRIPT if notification == CBN_SELCHANGE => self.clear(),
            MAP_RESULTS if notification == LBN_SELCHANGE => self.selected_changed(),
            MAP_CONVERT => self.convert()?,
            MAP_POLL => self.poll(),
            MAP_COPY => {
                if let Some(c) = self.selected() {
                    copy(&c.text, self.window)?;
                    set_text(
                        self.get(MAP_STATUS),
                        self.label("Copied. Clipboard replaced.", "已复制，剪贴板已替换。"),
                    );
                }
            }
            MAP_PALETTE => {
                if let Some(c) = self.selected() {
                    if c.text.len() > 4096 {
                        return Err(
                            "Result exceeds the palette composition limit; use Copy directly."
                                .into(),
                        );
                    }
                    return Ok(Some(MappingAction::Palette(c.text.clone())));
                }
            }
            MAP_STUDIO
                if self.result.as_ref().is_some_and(|r| {
                    r.language == "en-US" && r.output_type == Representation::Ipa
                }) =>
            {
                if let Some(c) = self
                    .selected()
                    .filter(|c| c.ipa_grammar_accepted == Some(true))
                {
                    return Ok(Some(MappingAction::Studio(c.text.clone())));
                }
            }
            _ => {}
        }
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn hidden_mapping_layout_conversion_and_transfer() {
        unsafe {
            let instance = GetModuleHandleW(null());
            // The class string remains alive for RegisterClassW's synchronous call.
            let name = wide("IPAFirstPalette");
            let class = WNDCLASSW {
                lpfnWndProc: Some(procedure),
                hInstance: instance,
                lpszClassName: name.as_ptr(),
                ..zeroed()
            };
            assert!(RegisterClassW(&class) != 0 || GetLastError() == ERROR_CLASS_ALREADY_EXISTS);
            let mut mapping = MappingWindow::new(null_mut(), false, false).unwrap();
            for (width, height) in [(420, 600), (720, 800), (900, 1000)] {
                SetWindowPos(
                    mapping.window,
                    null_mut(),
                    0,
                    0,
                    width,
                    height,
                    SWP_NOACTIVATE | SWP_NOZORDER,
                );
                mapping.layout();
                let mut client = zeroed();
                GetClientRect(mapping.window, &mut client);
                for id in [
                    MAP_LANGUAGE,
                    MAP_SCHEME,
                    MAP_SCRIPT,
                    MAP_INPUT,
                    MAP_CONTEXT,
                    MAP_CONVERT,
                    MAP_RESULTS,
                    MAP_DETAILS,
                    MAP_COPY,
                    MAP_PALETTE,
                    MAP_CLOSE,
                    MAP_STATUS,
                ] {
                    let mut rect: RECT = zeroed();
                    GetWindowRect(mapping.get(id), &mut rect);
                    let mut start = POINT {
                        x: rect.left,
                        y: rect.top,
                    };
                    let mut end = POINT {
                        x: rect.right,
                        y: rect.bottom,
                    };
                    ScreenToClient(mapping.window, &mut start);
                    ScreenToClient(mapping.window, &mut end);
                    // Combo popup bounds are allowed below the collapsed field.
                    assert!(
                        start.x >= 0
                            && start.y >= 0
                            && end.x <= client.right
                            && (matches!(id, MAP_LANGUAGE | MAP_SCHEME | MAP_SCRIPT)
                                || end.y <= client.bottom),
                        "control {id} {width}x{height}"
                    );
                }
            }
            SendMessageW(mapping.get(MAP_LANGUAGE), CB_SETCURSEL, 3, 0);
            mapping.language_changed();
            set_text(mapping.get(MAP_INPUT), "コーヒー");
            mapping.convert().unwrap();
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(3);
            while mapping.worker.is_some() && std::time::Instant::now() < deadline {
                std::thread::sleep(std::time::Duration::from_millis(10));
                mapping.poll();
            }
            assert_eq!(mapping.selected().unwrap().text, "kōhī");
            assert!(
                matches!(mapping.handle(MAP_PALETTE,0).unwrap(),Some(MappingAction::Palette(value)) if value=="kōhī")
            );
            assert_eq!(IsWindowVisible(mapping.get(MAP_STUDIO)), 0);
            mapping.localize(true);
            assert!(text(mapping.window).contains("语言映射"));
            set_text(mapping.get(MAP_INPUT), "学校");
            mapping.clear();
            assert!(mapping.selected().is_none());
        }
    }
}
