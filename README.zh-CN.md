<!-- SPDX-License-Identifier: CC-BY-4.0 -->
# IPA-first TTS

[English](README.md) · [v0.3.0-alpha.1 实验包](https://github.com/styayur/ipa-first-tts/releases/tag/v0.3.0-alpha.1)

**Alpha 预览版。** 当前语音仅支持英语，时间戳为估算，尚未达到稳定产品阶段。

**IPA-first TTS — An offline-first multilingual symbol palette and IPA-first speech tool.**

`v0.3.0-alpha.1` 实验预览版保留原有两个独立能力，并增加本地语言映射引擎：Windows 原生 **Symbol Palette 字符面板**，以及保留现有浏览器界面与 Rust 引擎的 **IPA Studio**。首批目录有 368 条精选记录，涉及常见 IPA，以及法语、德语、西班牙语、葡萄牙语、波兰语、捷克语、土耳其语和越南语的部分拉丁字符。目录覆盖、IPA Core 解析、后端词表支持和真实语言流程验证是四项不同能力；语音仍仅启用英语。

打开面板或执行内置映射不加载 TTS 模型、浏览器运行时或 HTTP 服务；明确选择可选 G2P 方向时才启动本地 eSpeak 进程；`ipa-core` 与 `symbol-catalog` 都是独立可复用库。[验证记录](docs/validation/README.md) 区分原生实测、用户反馈、自动化检查与未验证范围。请仅下载 Release 实际列出的资产。

## v0.3.0-alpha.1 语言映射

面板点 **Mapping / 语言映射**，或使用 Studio 新增的 Language mapping 区域。
选择语言、方向、具名方案和源文字，输入后按 Ctrl+Enter；多候选需自行选择。
来源、规则、版本和限制随结果展示。“送入面板”只填写组合框，不自动插入；
复制明确替换剪贴板，英语 IPA 可送至现有 Studio 编辑器。
轻量包新增独立 `ipa-map.exe`，无需 TTS 模型或 speech DLL。

| 已登记档案 | 罗马化 | G2P | IPA 解析 | TTS |
|---|---|---|---|---|
| en-US | 不适用 | 443 词 CMUdict 子集 / 549 个读音；可选 eSpeak en-us | 保守语法解析 | 仍仅已有英语后端 |
| fr | 未实现 | 可选 eSpeak fr-fr | 同一保守解析器 | 未开放 |
| es | 未实现 | 可选 eSpeak es（西班牙档案） | 同一保守解析器 | 未开放 |
| ja | CLDR 49 具名假名子集；不含汉字读音 | 未实现 | 不代表日语音系覆盖 | 未开放 |
| cmn-Hans | 44,498 个 NFC 汉字默认读音、3 个词语语境；仍有多音歧义 | 未实现 | 不代表普通话音系覆盖 | 未开放 |
| ru | CLDR 49 俄语字母子集及限定逆向；不宣称 ISO 9 符合性 | 未实现 | 不代表俄语音系覆盖 | 未开放 |

上述能力分别登记，不使用笼统的“支持该语言”标签。普通话输出是带调拼音形式，
不是 IPA，不处理通用多音字、分词或变调；`重` 显示候选，语境 `重庆` 才选 `chóng`。
日语 `コーヒー → kōhī`、`がっこう → gakkou`、`ぢづ → djidzu` 属于指定子集，
不是通用 Hepburn 或语音转写。俄语 `Щука → Ŝuka` 的限定往返也不是 IPA 到拼写。
英语 P2G 仅查所附词典的精确 IPA 候选，保留重音，不逐音素替换字母后声称还原原文。

```powershell
ipa-map languages
ipa-map romanize --language ja --scheme cldr49-kana-subset --input コーヒー
ipa-map map --language en-US --scheme cmudict-us-sample --input read --target ipa
ipa-map ipa-to-text --language en-US --input /sˈi/
ipa-map map --language fr --scheme espeak-fr-fr --input eau --target ipa
```

`ipa-tts` 接受相同命令并输出 JSON。内置映射无需联网；通用英语、法语、西语 G2P
需要可选 eSpeak / 完整包，不在运行时下载规则，也不引入云 API 或 Python 运行时。
未知语言/方案、文字不匹配与未覆盖字符明确失败，不静默换方案或丢字。

参考用例规模：英语 6、日语 9、俄语 5、汉字 5 个精确子集用例；法/西语各 3 个独立
词典参考按明确的“去重音/音节点后比较音段”指标验证。它们不是整种语言准确率。
详见[映射使用与限制](docs/language-mapping.md)、[权威资料与许可登记](docs/research/authority-sources.json)、
[架构决策](docs/decisions/0002-typed-language-mapping.md)。

## Windows 字符面板

```powershell
cargo build -p ipa-palette --release --locked
.\target\release\ipa-symbol-palette.exe
```

聚焦目标编辑器，按 **Ctrl+Alt+Shift+I** 呼出后直接搜索字符、英文名称、别名、语言名/代码或 `U+0252` 等码位。方向键移动选择，Enter 按当前动作插入或复制，Esc 隐藏，Tab 切换控件。托盘提供显示、隐藏、IPA Studio、设置、退出。关闭窗口只隐藏；退出会注销快捷键并停止由面板启动的 Studio。重复启动会显示已有实例。

紧凑字符格显示字形与码位，完整名称保留在详情区。重置清除搜索和筛选；Ctrl+F 返回搜索，Ctrl+D 切换收藏，Shift+Enter 复制。目标提示与 ↵ 按钮标识当前动作；无有效目标时 Enter 复制，插入按钮禁用。主面板的“退出”按钮可直接结束应用，无需寻找托盘图标。

标题显示当前版本，托盘提示显示实际保存的快捷键。快捷键也会一并隐藏映射与设置窗口。从其他目录启动副本时，会提示已有实例的位置，便于先退出旧版再切换。

分类包含最近使用、收藏、IPA、语言字符、附加符号与组合。**复制**会明确替换剪贴板内容；**插入**校验原目标窗口及前台焦点后发送目录原始 UTF-16 序列，不改动剪贴板。请从目标应用用快捷键呼出；从托盘打开时使用复制。权限、远程会话或目标应用限制可能拒绝输入；应检查错误并主动选择复制。本轮无自动剪贴板粘贴回退，也不提升权限。

**组合 +** 将完整候选追加到组合框，包括附加符号。组合框非空时，复制、插入与 Studio 使用组合内容，直到清空。附加符前的虚线圆圈及空格显示标记不会写入输出。详情码位用于核对字形，扩展字符可能需要目标应用具有适合字体。

设置可修改快捷键；冲突时保留原注册。English / 简体中文切换界面不会修改字符输出。收藏、最近使用和设置原子保存至 `%LOCALAPPDATA%/IPA-first-TTS/palette.json`；损坏文件保留为 `palette.corrupt-*`，未知未来版本不覆盖。默认不开机启动。详见[交互与安全](docs/palette.md)。

完整包中将 `ipa-tts.exe` 与面板放在同一目录，即可发送至 IPA Studio；轻量面板无需它。Studio 的 IPA 编辑器也使用同一份可搜索目录。不支持的字符仍可复制；IPA 解析与后端适配明确报错，不静默删除。

发布资产名为 `ipa-symbol-palette-v0.3.0-alpha.1-windows-x64.zip`（无模型），以及 `ipa-first-tts-v0.3.0-alpha.1-windows-x64-offline.zip`。请仅下载 [Releases](https://github.com/styayur/ipa-first-tts/releases) 实际列出的资产，并核对 SHA-256 与对应源码。SHA-256 不是代码签名；本轮无 Authenticode 证书。

首期支持英语（en-US / en-GB）。语言标识为开放字符串，`zh`、`ja`、`fr`、`de` 可以由新 backend 启用；当前入口会明确拒绝这些语言，避免声称已经支持。

IPA Studio 继续使用 Rust 提供的 loopback 浏览器界面与原生 HTML/CSS/JavaScript；字符面板使用 Rust/Win32 控件。无 npm、CDN、React 或 Tauri 运行时依赖，字体使用系统字体。字符功能无需模型、联网或 Studio 初始化。

![IPA Studio 界面](outputs/desktop.png)

## 已发布的历史 IPA Studio v0.1.0 / Windows x64

从 [GitHub Releases](https://github.com/styayur/ipa-first-tts/releases/latest) 下载 `ipa-first-tts-v0.1.0-windows-x64-offline.zip`。在 Windows 10/11 x64 上完整解压后双击 `Start-IPA-Studio.cmd`，打开终端打印的本地地址。这个包包含 Kokoro 模型、eSpeak-NG、匹配的 sherpa/ONNX Runtime DLL 和应用目录内的 Microsoft VC++ runtime，无须安装 Rust、Python、Node 或 VC++ Redistributable，也无须联网。

关闭终端或按 Ctrl+C 结束服务器。命令行操作可在解压目录运行：

```powershell
.\ipa-tts.exe doctor
.\ipa-tts.exe demo outputs\quick-brown-fox.wav kokoro
.\ipa-tts.exe synth "ˈʃiː" outputs\edited.wav kokoro
```

下载包后，可用 `Get-FileHash .\ipa-first-tts-v0.1.0-windows-x64-offline.zip -Algorithm SHA256` 与同一 Release 的 `SHA256SUMS.txt` 比对。`corresponding-source.zip` 提供应用源码、Cargo vendored 依赖和 native 上游源码；源码获取与许可证边界见 [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md)。

## Quick start / Windows

需要 PowerShell 7（`pwsh`）、Rust stable MSVC、Visual Studio C++ Build Tools / Windows SDK，以及 Windows 的 `curl.exe` 和 `tar.exe`。所有命令均在此项目根目录运行：

```powershell
git clone https://github.com/styayur/ipa-first-tts.git
cd ipa-first-tts
pwsh -File scripts/setup.ps1
cargo build --workspace --locked
cargo run -p ipa-desktop -- doctor
cargo run -p ipa-desktop -- serve
```

在浏览器打开程序打印的 [本地地址](http://127.0.0.1:17842)。源码仓库不包含模型或工具二进制；源码构建时先运行 setup，完整 Release 包则已经包含这些资产。

1. Text 中输入文本，点击 **Generate IPA**。
2. 编辑 IPA；输入时自动执行 Unicode 规范化和校验。符号按钮插入到光标处。
3. 点击 **Synthesize**；Kokoro 接收编辑后的音素。默认 speaker 0，speed 1。
4. 点击 **Play / Stop**，或 **Save WAV** 保存音频。
5. 下方显示结构化音素序列。高亮依据均匀估算，界面明确标注 `Estimated timing`。

若端口冲突，可运行 `cargo run -p ipa-desktop -- serve 17843`。UI 只绑定 `127.0.0.1`，必须使用打印的准确地址；不开放 CORS。

### Actual G2P result

From the extracted Windows offline v0.1.0 package:

```powershell
.\ipa-tts.exe g2p "The quick brown fox jumps over the lazy dog."
```

Actual stdout on 2026-10-09 (exit 0, bundled eSpeak-NG):

```text
ðə kwˈɪk bɹˈa͡ʊn fˈɑːks d͡ʒˈʌmps ˌo͡ʊvɚ ðə lˈe͡ɪzi dˈɑːɡ
```

This demonstrates text-to-IPA only; it does not claim a new Kokoro synthesis or measured phoneme alignment. The existing real UI screenshot is retained.

## Architecture

架构图唯一维护源见[英文版架构](README.md#architecture)。

The UI runs against a Rust loopback HTTP server, not Tauri. The CLI uses the same Engine directly. G2P invokes eSpeak-NG as a child process; synthesis consumes the edited IpaSequence, not the original text. Kokoro uses a vocabulary adapter, temporary phoneme lexicon and native sherpa-onnx session. eSpeak is an explicitly selected fallback, not an automatic recovery path.

Alignment follows synthesis: UniformAligner divides audio duration uniformly across phones and reports TimingQuality::Estimated. No forced aligner or measured phoneme timestamps are implemented. The server retains at most four WAV results in memory; CLI synthesis writes the selected output file. Host/Origin checks protect the loopback boundary; this is not a public authenticated service.

[Source evidence and diagram verification](docs/architecture/README.md).

```text
ipa-first-tts/
├── Cargo.toml / Cargo.lock
├── crates/
│   ├── ipa-core/         Unicode parser、IPA IR、validation、normalization、G2pBackend
│   ├── g2p-espeak/       eSpeak-NG G2P、英语音素合成 fallback
│   ├── tts-core/         TtsBackend、PhonemeAdapter、Audio、PCM16 WAV
│   ├── tts-kokoro/       Kokoro 词表 adapter、sherpa-onnx CPU 推理
│   └── alignment-core/  Aligner、TimingQuality、显式的估算 timing
├── apps/desktop/
│   ├── src/             CLI、本地 HTTP UI、应用编排
│   ├── ui/              文本和 IPA 编辑、播放、音素显示
│   └── tests/pipeline.rs 真实 eSpeak / Kokoro 集成测试
├── scripts/
│   ├── setup.ps1        Windows 便携版 eSpeak / 模型准备
│   ├── test_ui.py       使用真实 backend 的 Playwright 测试
│   └── package_release.py  离线包和源码包制作
├── licenses/           完整许可证文本、作用范围和第三方通知
├── models/README.md     模型格式和准备说明
├── tools/              本地工具（忽略提交）
└── outputs/            实际生成的 WAV、界面截图、验证结果
```

依赖方向：

```text
apps/desktop ──> g2p-espeak ──> ipa-core
             ├─> tts-kokoro ──> tts-core ──> ipa-core
             └─> alignment-core ──> tts-core / ipa-core
```

核心 trait 为 Rust trait（不是 Kotlin）：

```rust
// ipa-core
pub trait G2pBackend {
    fn name(&self) -> &str;
    fn generate(&self, text: &str, language: &str) -> Result<IpaSequence, String>;
}
// tts-core
pub trait PhonemeAdapter {
    fn adapt(&self, ipa: &IpaSequence) -> Result<ModelPhonemes, String>;
}
pub trait TtsBackend {
    fn name(&self) -> &str;
    fn synthesize(&self, ipa: &IpaSequence, options: SynthesisOptions) -> Result<Audio, String>;
}
// alignment-core
pub trait Aligner {
    fn align(&self, ipa: &IpaSequence, audio: &Audio) -> Result<Alignment, String>;
}
```

`TtsBackend` 的输入没有原始文本，从接口层面防止悄悄忽略用户编辑后重新执行 G2P。`ModelPhonemes` 保存模型 token 和来源 IR 索引；Kokoro 特有映射全部在 adapter 内。

IR：

```rust
pub struct IpaSequence {
    pub language: Option<String>,
    pub phones: Vec<IpaPhone>,
}
pub struct IpaPhone {
    pub symbol: String,
    pub stress: Option<Stress>,
    pub kind: PhoneKind, // Phone / WordBoundary / Punctuation
    pub start_ms: Option<f32>,
    pub end_ms: Option<f32>,
}
```

词边界为显式 `WordBoundary` 条目，canonical 输出为一个空格；音标修饰符属于前一个 phone。绑定音 `t͡ʃ`、双元音 `a͡ʊ` 为单个 IR 条目；无需把模型 token 粒度强加给 IPA Core。

直接依赖为 serde、serde_json、unicode-normalization、tiny_http，以及可选 sherpa-onnx；原生面板另用 windows-sys 0.61.2（MIT/Apache-2.0）调用系统 API。WAV 编码使用 Rust 标准库。Studio、IPA/TTS 核心与目录均 `forbid(unsafe_code)`；原生 Windows 模块和配置原子替换包含明确隔离的 unsafe，第三方 sherpa 封装另有 FFI。完整清单见 `licenses/rust-dependencies.json`。

## IPA pipeline

```text
Text
  -> eSpeak-NG CLI (--ipa=2, en-us / en-gb)
  -> IpaSequence::parse / Canonical IPA
  -> 用户编辑 IPA
  -> Unicode NFD / validation
  -> KokoroAdapter + tokens.txt 验证
  -> 编辑后音素的临时 lexicon
  -> sherpa-onnx / Kokoro / CPU
  -> 单声道 PCM16 WAV / 浏览器播放
```

Core 接受 `/.../` 和 `[...]` 外壳、Unicode IPA、组合附加符、主重音 `ˈ`、次重音 `ˌ`、长音 `ː`、半长音 `ˑ`、tie bar、词边界和基本标点。规范化包含 NFD、ASCII `g` → IPA `ɡ`、两种 tie bar 合并、连续空白折叠、`|` → 词边界。重音附到下一 phone，长音附到上一 phone。重新规范化会清除旧 timing。

校验拒绝空音序列、未识别字符、悬空重音/附加符、重复长音、损坏的 IR、负数/NaN/非单调 timing。它是**基础语法和结构校验**，不是完整 IPA 或语言音系校验。错误中的字符位置指向规范化后的 IPA。

Kokoro adapter 将 `t͡ʃ` → `ʧ`、`d͡ʒ` → `ʤ`、英语绑定双元音 → 模型 Unicode tokens、`ɚ` → `əɹ`、`ɝ` → `ɜɹ`。不支持的音素明确报错，不会静默删除，也不会自动切换 backend。

### 确保编辑直接进入模型

sherpa-onnx 常规 `generate(text)` 会执行 G2P。此项目用只含本次发音的临时词典，将 `ipawordaa` 等纯字母 alias 映射到模型音素。针对锁定的 1.13.8 runtime，还必须清空私有模型副本中的 `voice` 元数据，才能进入 lexicon 分支。原始 ONNX 和权重不被修改。准备器对 opaque graph 字节原样流式复制，单元测试验证只改此 metadata。

每次 synthesis 重新建立 native session，让新词典生效。推理关闭 silence scaling，减少额外音频修改。临时资源由 RAII 在正常退出时清理。详细机制见 [模型说明](models/README.md)。

## 如何安装 eSpeak-NG

Windows 推荐 `scripts/setup.ps1`：固定版本 1.52.0 MSI，由 lessmsi 2.12.9 提取为便携目录 `tools/espeak-ng/`，不需要系统安装或管理员权限。也可手动从 [官方 releases](https://github.com/espeak-ng/espeak-ng/releases/tag/1.52.0) 安装，并设置：

```powershell
$env:ESPEAK_NG = 'C:\Program Files\eSpeak NG\espeak-ng.exe'
```

必须保留 exe 旁的 `espeak-ng-data/`；程序自动传递 `--path`。官方 Windows 1.52.0 在本机执行 `--version` 会崩溃，因此 `doctor` 使用真实 `hello` G2P 检查。

Linux：`sudo apt install espeak-ng`；macOS：`brew install espeak-ng`。或者把可执行文件放入 PATH。`ESPEAK_NG` 覆盖自动发现路径。

## 如何准备 Kokoro / sherpa-onnx

Windows 运行 `scripts/setup.ps1`，下载 sherpa 的 `kokoro-multi-lang-v1_0` bundle。也可按 [models/README.md](models/README.md) 手动下载和解压。默认目录含 `model.onnx`、`voices.bin`、`tokens.txt`、`espeak-ng-data/`。此 MVP 不使用随 bundle 下载的英语/中文大型 lexicon，而只生成本次输入的极小词典。

```powershell
$env:KOKORO_MODEL_DIR = 'D:\models\kokoro-multi-lang-v1_0'
```

Rust 依赖固定 `sherpa-onnx = 1.13.8`，使用官方 **shared** CPU runtime。首次 `cargo build` 时，第三方 build script 会下载匹配的 native libraries；编译后 runtime DLL 自动复制到 `target/debug/` 或 `target/release/`。Windows 分发时，必须把生成的所有 runtime DLL 与 `ipa-tts.exe` 放在一起，不能只复制 EXE。

构建本身可以联网准备依赖；运行不会联网。完全离线构建需要提前缓存 Cargo 依赖/native archive，或设置 `SHERPA_ONNX_LIB_DIR` 指向匹配版本的本地库，随后使用 `cargo build --offline --locked`。不匹配的 ORT DLL 可能导致 native 崩溃；不要混用其他项目的 DLL。

## CLI / 示例

```powershell
cargo run -p ipa-desktop -- g2p "The quick brown fox jumps over the lazy dog."
cargo run -p ipa-desktop -- demo outputs/quick-brown-fox.wav kokoro
cargo run -p ipa-desktop -- synth "ˈʃiː" outputs/edited.wav kokoro
```

本次验证的示例 IPA：

```text
ðə kwˈɪk bɹˈa͡ʊn fˈɑːks d͡ʒˈʌmps ˌo͡ʊvɚ ðə lˈe͡ɪzi dˈɑːɡ
```

实际结果为 24 kHz、约 6.03 秒的 Kokoro 音频，保存在 `outputs/quick-brown-fox.wav`。eSpeak 版本/口音不同，IPA 可能略有不同。

### 最小 fallback

不安装模型或 native runtime 也能跑通真实语音流程：

```powershell
pwsh -File scripts/setup.ps1 -SkipModel
cargo run -p ipa-desktop --no-default-features -- serve
cargo run -p ipa-desktop --no-default-features -- demo outputs/fallback.wav espeak
```

eSpeak fallback 将编辑后的英语 IPA 转为 eSpeak mnemonic `[[...]]` 音素输入，再生成本地语音；不是假音频，也不是 Kokoro。未知音素报错。fallback 固定 en-US、speaker 0，声质明显更机械。

## Build / Test

```powershell
cargo build --workspace --locked
cargo test --workspace --locked
cargo test -p ipa-desktop --test pipeline -- --ignored --test-threads=1
cargo clippy --workspace --all-targets -- -D warnings
```

普通 `cargo test` 不要求本地模型，真实集成测试标为 ignored，需要显式运行。它们使用真实 eSpeak G2P、fallback 和 Kokoro CLI；验证 WAV、采样率、非静音、IPA 编辑影响、未知模型音素错误。Kokoro 集成用 public CLI 子进程，让 Windows 优先载入 EXE 旁的匹配 DLL，避免 `target/debug/deps` 下的测试进程误载系统自带旧 ORT。

独立测试 Core / 最小构建：

```powershell
cargo test -p ipa-core --offline
cargo test -p ipa-desktop --no-default-features --offline
```

真实 UI 测试可选：在 `ipa-tts serve` 运行期间，执行 `python scripts/test_ui.py`。需要 Python Playwright 和 Chromium（只用于开发测试，不是应用运行依赖）。该测试验证生成、修改、实际 Kokoro 合成、浏览器播放、Stop、估算高亮、错误处理和 390 px 布局，并写出 `outputs/ui-verification.json` 和截图。

## 当前限制

- 实测平台为 Windows x64。Rust 核心无 OS 专属依赖；Linux/macOS 的 native runtime 和音频播放尚未实测。
- Studio 为本地浏览器应用；新增 Win32 字符面板面向 Windows x64，真实桌面验证边界见[验证记录](docs/validation/README.md)。不宣称 Linux/macOS 原生面板支持，也未提供安装器或 Tauri 运行时。
- 语音仅英语；字符目录涉及多种语言，但不扩展后端能力。eSpeak 发音与 Kokoro 训练用的 Misaki 发音前端不完全一致。
- Core 支持常见 IPA 字符的保守 inventory，不涵盖全部历史/扩展 IPA；不检查英语音系合法性。
- 本地 G2P text 上限 4096 UTF-8 bytes，UI IPA 上限 8192 bytes，Kokoro 上限 480 model tokens；长文需拆分。
- `--ipa=2` 可能不保留原文标点。用户可在 IPA 中补入标点；MVP 没有独立 prosody IR。
- timing 全部是均匀估算，**不是模型时长或 forced alignment**。不能用于发音评分。
- 每次 utterance 重新载入 session/词典，延迟和内存占用高于缓存普通 TTS session；先保证编辑有效。可通过 `IPA_TTS_DEBUG=1` 查看 native tokens。
- 单进程顺序执行，合成期间 UI 等待；Stop 停止音频播放，不取消正在进行的 native inference。
- native runtime 对损坏/不匹配资产可能直接退出或崩溃，MVP 做路径、元数据、词表与参数检查，尚未把 UI 推理隔离到 worker 进程。
- HTTP 音频缓存只保存最新 4 条，服务器重启后失效。保存 WAV 可长期保留。

## Roadmap

1. 以稳定的音素/token 推理桥替代临时词典，缓存 Kokoro session；将推理放入可重启 worker，支持取消。
2. 验证并完善独立 Win32 字符面板与便携包；其他系统与安装器仍需各自实测。
3. 加入模型真实 duration 或 `MFA` forced aligner，通过 `Aligner` 返回 `Model` / `Forced` timing。
4. 为 zh / ja / fr / de 增加可测试的 G2P 与 backend adapter；IPA Core 保持模型无关。
5. 新增 Piper `TtsBackend`；WikiPron / Epitran `G2pBackend`，无需修改 IPA Core。

不包含训练、voice cloning、STT、AI chat、云 API、用户系统、完整语言学习功能或自建大型 pronunciation database。

## 贡献与支持

阅读 [CONTRIBUTING.md](CONTRIBUTING.md)，PR 运行适用检查并注明真实模型/设备测试范围。通过 [Issues](https://github.com/styayur/ipa-first-tts/issues) 提交脱敏复现；漏洞按 [SECURITY.md](SECURITY.md) 处理。不要提交下载模型、本地工具、生成音频或用户配置。

## Upstream references

- [eSpeak-NG](https://github.com/espeak-ng/espeak-ng)
- [sherpa-onnx Kokoro 模型说明](https://k2-fsa.github.io/sherpa/onnx/tts/pretrained_models/kokoro.html)
- [锁定 runtime 的 Kokoro frontend 调用](https://github.com/k2-fsa/sherpa-onnx/blob/v1.13.8/sherpa-onnx/csrc/offline-tts-kokoro-impl.h)
- [Kokoro lexicon frontend](https://github.com/k2-fsa/sherpa-onnx/blob/v1.13.8/sherpa-onnx/csrc/kokoro-multi-lang-lexicon.cc)

## License

依照 [styayur License Policy](https://github.com/styayur/styayur/blob/main/LICENSE_POLICY.md)，按组件用途授权：

| 组件 | 许可证 |
|---|---|
| `apps/desktop/`、`apps/palette/` 完整应用、CLI、UI、应用测试与组合可执行程序 | **GPL-3.0-or-later** |
| `crates/` 六个独立、可复用 Rust 库（目录数据另按记录标注） | **MPL-2.0** |
| `scripts/` 与 CI 自动化 | **MIT** |
| 独立编写的 README / Markdown 文档 | **CC BY 4.0** |
| Kokoro 模型、eSpeak-NG、sherpa-onnx、ONNX Runtime、依赖和第三方数据 | **保留各自原许可证** |

Copyright © 2026 **Stya Yur Open Source Studio**。根目录 [LICENSE](LICENSE) 覆盖应用；可复用库的 Cargo metadata、各 crate 的 LICENSE 和源码 SPDX header 明确标为 MPL-2.0。MPL 文件未声明不兼容 secondary licenses，组合为 GPL 应用时适用 MPL 2.0 §3.3。文档引用请标注作者和本仓库链接，注明修改，并链接 CC BY 4.0。

完整边界、授权文本和来源见 [licenses/README.md](licenses/README.md)、[NOTICE](NOTICE) 和 [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md)。第三方模型、数据、字体、代码与二进制不会因本项目许可证而被重新授权；无第三方字体随包分发。
