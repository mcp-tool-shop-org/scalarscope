<p align="center">
  <a href="README.ja.md">日本語</a> | <a href="README.md">English</a> | <a href="README.es.md">Español</a> | <a href="README.fr.md">Français</a> | <a href="README.hi.md">हिन्दी</a> | <a href="README.it.md">Italiano</a> | <a href="README.pt-BR.md">Português (BR)</a>
</p>

<p align="center">
  <img src="https://raw.githubusercontent.com/mcp-tool-shop-org/brand/main/logos/ScalarScope/readme.png" width="400" alt="ScalarScope">
</p>

> 这是 [MCP 工具商店](https://mcptoolshop.com) 的一部分。

<p align="center">
  <a href="https://github.com/mcp-tool-shop-org/scalarscope/actions/workflows/build.yml"><img src="https://github.com/mcp-tool-shop-org/scalarscope/actions/workflows/build.yml/badge.svg" alt="Build and Test"></a>
  <a href="https://codecov.io/gh/mcp-tool-shop-org/scalarscope"><img src="https://codecov.io/gh/mcp-tool-shop-org/scalarscope/graph/badge.svg" alt="Coverage"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/License-MIT-yellow.svg?style=flat-square" alt="License: MIT"></a>
  <a href="https://mcp-tool-shop-org.github.io/scalarscope/"><img src="https://img.shields.io/badge/Landing-page-blue?style=flat-square" alt="Landing page"></a>
  <a href="https://apps.microsoft.com/detail/9P3HT1PHBKQK"><img src="https://img.shields.io/badge/Microsoft%20Store-9P3HT1PHBKQK-0078D4?style=flat-square&logo=microsoft" alt="Microsoft Store"></a>
</p>

**对两次机器学习运行的评估。** 您从这个仓库构建的应用程序是 `rust/` 中的 Rust 程序。发布流程将该程序打包为未签名的 3.0.0.0 MSIX。包名称和发布者保持不变。该文件未上传。在上传之前，商店中的版本仍然是之前的 .NET 包。

包版本 **3.0.0.0**。商店中 [9P3HT1PHBKQK](https://apps.microsoft.com/detail/9P3HT1PHBKQK) 的更新保留了名称 `mcp-tool-shop.ScalarScope` 和发布者 `CN=5305D976-6952-4F00-9C21-3A5DB090359F`。

## 信任模型

评估会读取您打开的两个文件。打包后的运行还会读取并写入该包的 LocalState 文件夹中的 `comparison-log.json` 和 `preferences.json`。捆绑包仅写入您选择的路径。

它不会将这些文件发送到任何地方。没有帐户、没有遥测数据，也没有分析数据。该文件夹中的插件会保留在原位，并且不会加载。捆绑包上的哈希值会检查存档字节。这是一个内容检查，而不是签名，它不会说明谁编写了该文件。

该程序需要权限才能读取您选择的文件并写入您保存的捆绑包。

---

## 为什么选择 ScalarScope？

大多数机器学习团队会手动查看日志。ScalarScope 用结构化、可重现的比较取代了这种方式。

以下要点描述了已发布的 .NET 包。Rust 评估会绘制一个推理序列或一个反向传播训练损失曲线。对于一个推理对，它会报告 ΔF 和 ΔO，并且仅在双方都具有稳定状态里程碑时才会报告 ΔTc。当两个里程碑都存在时，该序列会在该处绘制一条垂直线。它会在第 7.2 阶段的布局中写入一个 `.scbundle`。重新打开该文件会显示存储的评估。哈希值是存档文件字节的 SHA-256 值，而 `integrity.json` 是密封。匹配的哈希值是一个内容检查，而不是签名。打包后的运行会在该包的 LocalState 文件夹中保留 `comparison-log.json` 和 `preferences.json`，这些文件与 .NET 应用程序写入的文件相同。序列颜色遵循保存的色彩视觉模式。该文件夹中的最近文件和保存的视图会在此处重新打开。该文件夹中的插件会保留在原位，并且不会加载。未打包的运行不会写入该文件夹。

- **苹果对苹果的比较** — 并排加载两个推理轨迹，并查看确切的变化
- **规范 delta 分析** — 五种 delta 类型（ΔTc、ΔO、ΔF、ΔĀ、ΔTd）仅在差异具有统计意义时触发
- **运行时预设** — TFRT 预设会自动抑制不相关的指标，以便您专注于对 TensorFlow-TRT 工作负载而言重要的内容
- **可重现的捆绑包** — 导出带有 SHA-256 完整性、冻结 delta 和完整来源元数据的 `.scbundle` 存档
- **评估模式** — 在不重新计算的情况下打开捆绑包。匹配的 SHA-256 是内容检查，而不是签名。
- **首先考虑隐私** — 零遥测，零分析，所有数据都保留在本地，除非您明确导出

---

## VortexKit

VortexKit 是 `src/VortexKit` 中的可视化库。它是此仓库的一部分。它未发布到 NuGet。

它涵盖了时间同步播放、动画 SkiaSharp 画布、比较视图、注释叠加、SVG 和 PNG 导出以及语义色彩系统。

---

## 快速入门

### Rust 评估

来自此仓库：

```
cargo run --manifest-path rust/Cargo.toml
```

打开两个推理文件或两个反向传播 `run_history.json` 文件。推理文件是一个延迟 CSV、一个基准 JSON 或一个 Chrome 轨迹。该轨迹中的一个完整的 `ProfilerStep` 是一个推理，并且其中的操作不是额外的样本。即使轨迹中没有步骤，它仍然使用名称中包含 TensorRT 或推理的事件。训练文件绘制为训练损失。保留损失、困惑度和任务指标与该曲线一起显示。`final_loss` 显示为它自己的数字。一个推理对会报告延迟序列中的 ΔF 和 ΔO。仅当两个文件都具有稳定状态里程碑时，才会报告 ΔTc。如果没有该里程碑，则最后一个步骤不会被视为稳定时间，并且该序列不会绘制稳定状态线。ΔTd 和 ΔĀ 不会显示在推理页面上。不会对训练历史记录计算推理 delta。保存捆绑包会将评估写入页面。打开捆绑包会再次显示存储的评估。哈希值与 .NET 第 7.2 阶段的检查匹配。匹配意味着字节完好无损。它不是签名。

`packaging/pack.ps1` 从发布二进制文件中构建未签名的 `ScalarScope_3.0.0.0_x64.msix`。包名称为 `mcp-tool-shop.ScalarScope`，发布者为 `CN=5305D976-6952-4F00-9C21-3A5DB090359F`，架构为 x64。它未上传。商店中的版本仍然是之前的 .NET 包。

### 来自 Microsoft Store

1. 从 [Microsoft Store](https://apps.microsoft.com/detail/9P3HT1PHBKQK)（商店 ID：`9P3HT1PHBKQK`）安装 **ScalarScope**
2. 单击 **比较两次运行**
3. 加载基准轨迹：延迟 CSV、基准 JSON 或分析器 `trace.json`
4. 以相同类型的文件加载优化后的轨迹
5. 在 **比较** 选项卡中查看 delta
6. 导出 `.scbundle` 以进行可重现的共享

### 使用 VortexKit

```csharp
using VortexKit.Core;

// 1. Create a shared playback controller (0.0 -> 1.0 timeline)
var player = new PlaybackController { Duration = 10.0, Loop = true };

// 2. Bind multiple animated canvases to the same controller
player.TimeChanged += () =>
{
    trajectoryCanvas.CurrentTime = player.Time;
    eigenCanvas.CurrentTime      = player.Time;
    scalarsCanvas.CurrentTime    = player.Time;
};

// 3. Subclass AnimatedCanvas for custom rendering
public class MyTrajectoryCanvas : AnimatedCanvas
{
    protected override void OnRender(SKCanvas canvas, SKImageInfo info, double time)
    {
        // Your SkiaSharp rendering at the current time position
    }
}

// 4. Export a side-by-side comparison as PNG
var exporter = new ExportService();
await exporter.ExportComparisonAsync(
    leftRender, rightRender, time: 0.5,
    outputPath: "comparison.png",
    new ComparisonExportOptions
    {
        Width = 1920, Height = 1080,
        LeftLabel = "Baseline", RightLabel = "Optimized",
        ShowLabels = true
    });

// 5. Export as layered SVG (Inkscape-compatible)
var svgExporter = new SvgExportService();
await svgExporter.ExportSvgAsync(svgData, "trajectory.svg",
    new SvgExportOptions
    {
        Palette = SvgColorPalette.Publication,
        UseCatmullRomSplines = true,
        EnableGlow = false
    });
```

---

## 功能

### Delta 分析 — 五种规范 Delta 类型

每次比较都会生成一组规范 delta。每当差异具有统计意义时，每个 delta 才会触发；不相关的 delta 会自动抑制。

| Delta | 完整名称 | 它衡量的内容 | 触发时间 |
|-------|-----------|------------------|------------|
| **ΔTc** | 收敛时间 | 达到稳定延迟所需的步骤 | 稳定状态在不同的步骤（3+ 步的间隔）处达到 |
| **ΔO** | 输出可变性 | 振荡/运行时不稳定 | 高于阈值的区域得分超过噪声阈值 |
| **ΔF** | 故障率 | 异常频率 | 两次运行之间的故障频率或类型不同 |
| **ΔĀ** | 平均延迟 | 平均指标值 | 平均值存在显著差异（在 TFRT 预设中已抑制） |
| **ΔTd** | 总时长 | 实际运行时间/结构出现时间 | 持续时间或主导性开始时间存在差异（在 TFRT 预设中已抑制） |

### 运行时预设 — TFRT

内置的 **TensorFlow-TRT** 预设 (`tensorflowrt-runtime-v1`) 映射与推理相关的信号（延迟、吞吐量、内存、CPU/GPU 负载），并抑制仅用于训练的差异（ΔĀ、ΔTd），这些差异对于推理比较没有意义。当预热时间超过运行时间的 50% 或仅提供聚合统计数据时，会显示警告。

### 可重现的软件包

将结果导出为 `.scbundle` 存档（ComparisonBundle v1.0.0）：

- **`manifest.json`** — 软件包元数据、应用程序版本、比较标签、对齐模式
- **`repro/repro.json`** — 输入指纹、预设哈希值、确定性种子、环境信息
- **`findings/deltas.json`** — 带有置信度评分、锚点和触发类型的规范差异
- **`findings/why.json`** — 易于理解的说明、安全措施、参数提示
- **`findings/summary.md`** — 自动生成的 Markdown 摘要
- **完整性** — 每个文件都使用 SHA-256 进行哈希处理。软件包哈希值是对内容进行检查，而不是签名。

### 审查模式

打开任何 `.scbundle`，无需重新计算。匹配的 SHA-256 是对内容进行检查，而不是签名。存储的差异将显示为已存储。

### VortexKit 可视化框架

VortexKit 是 `src/VortexKit` 中的可视化库。它与此仓库一起发布。它不是 NuGet 包。

| 组件 | 其功能 |
|-----------|-------------|
| `PlaybackController` | 共享的 0→1 时间线，具有播放/暂停/步进/循环功能、速度预设（0.25 倍 — 4 倍）、~60 fps 刷新率 |
| `AnimatedCanvas` | 抽象的 `SKCanvasView` 基类，具有时间同步的失效机制、网格绘制、触摸/拖动事件、坐标辅助功能 |
| `ITimeSeries<T>` / `TimeSeries<T>` | 通用的时间序列，具有索引↔时间映射和轨迹枚举功能 |
| `ExportService` | 单帧 PNG、帧序列（带有 ffmpeg 提示）以及并排比较导出 |
| `SvgExportService` | 完整的矢量 SVG 导出，带有 Inkscape 图层、Catmull-Rom 样条曲线、热图、矢量场和四种颜色调色板（默认、浅色、高对比度、出版） |
| `IAnnotation` | 带有理论基础和优先级的类型化注释（阶段、警告、见解、失败、自定义） |
| `VortexColors` | 语义颜色调色板 — 背景图层、强调语义、严重程度编码、特征值调色板、线性插值/渐变辅助功能 |

---

## 安装

### Microsoft Store（推荐）

**商店 ID：** `9P3HT1PHBKQK`

[从 Microsoft Store 获取](https://apps.microsoft.com/detail/9P3HT1PHBKQK)

需要 Windows 10（版本 17763）或更高版本。

### 从源代码

```bash
# Prerequisites:
#   .NET 9.0 SDK (global.json pins 9.0.100)
#   Visual Studio 2022 with MAUI workload, or:
#     dotnet workload install maui-windows

git clone https://github.com/mcp-tool-shop-org/scalarscope.git
cd scalarscope
dotnet restore
dotnet build

# Run the desktop app
dotnet run --project src/ScalarScope
```

---

## 项目结构

```
scalarscope/
├── src/
│   ├── ScalarScope/                    # .NET MAUI desktop app
│   │   ├── Models/                     # GeometryRun, InsightEvent
│   │   ├── ViewModels/                 # Welcome, Comparison, Export, Settings, TrajectoryPlayer, VortexSession
│   │   ├── Views/                      # XAML pages + 19 custom controls
│   │   │   ├── WelcomePage.xaml        # First-60-seconds onboarding (Home tab)
│   │   │   ├── ComparisonPage.xaml     # Side-by-side delta comparison (Compare tab)
│   │   │   ├── HelpPage.xaml           # Interpretation guide (Guide tab)
│   │   │   ├── SettingsPage.xaml       # Preferences and about (Settings tab)
│   │   │   └── Controls/              # DeltaZone, BundleExportPanel, PlaybackControl, etc.
│   │   ├── Services/
│   │   │   ├── Connectors/            # RunTraceComparer, TfrtRuntimePreset, validation
│   │   │   ├── Bundles/               # BundleBuilder, BundleExporter, integrity, schemas
│   │   │   ├── Evidence/              # Comparison evidence reports, detector diagnostics
│   │   │   ├── Plugins/               # PluginManager
│   │   │   ├── CanonicalDeltaService.cs
│   │   │   ├── DeltaTypes.cs          # 5 canonical deltas + detector configs
│   │   │   ├── DeterminismService.cs  # Reproducible seed management
│   │   │   ├── FlowFieldService.cs    # Vector field computation
│   │   │   └── ...                    # 70+ service files
│   │   └── Resources/
│   │       ├── Styles/DesignSystem.xaml # Unified visual grammar
│   │       └── Raw/Samples/            # Built-in example traces
│   │
│   └── VortexKit/                      # Visualization library in this repo
│       ├── Core/
│       │   ├── AnimatedCanvas.cs       # Time-synced SkiaSharp canvas base
│       │   ├── PlaybackController.cs   # Shared playback timeline
│       │   ├── ITimeSeries.cs          # Generic time-series interface
│       │   ├── ExportService.cs        # PNG frame/sequence export
│       │   └── SvgExportService.cs     # Layered SVG export
│       ├── Annotations/
│       │   └── IAnnotation.cs          # Typed annotation system
│       └── Theme/
│           └── VortexColors.cs         # Semantic color palette
│
├── tests/
│   ├── ScalarScope.FixtureTests/       # Golden-file fixture tests
│   ├── ScalarScope.DeterminismTests/   # Reproducibility verification
│   ├── ScalarScope.SoakTests/          # Long-running stability tests
│   └── Fixtures/                       # Shared test data
│
├── docs/                               # Design docs, results, limitations
├── .github/workflows/
│   ├── build.yml                       # CI: restore, build, format check, pack, artifacts
│   ├── publish.yml                     # NuGet publish
│   └── release.yml                     # GitHub Release + Store submission
├── global.json                         # .NET SDK 9.0.100
├── ScalarScope.sln                     # Solution file
├── CHANGELOG.md                        # Keep-a-Changelog format
├── PRIVACY.md                          # Privacy policy (no telemetry)
├── SECURITY.md                         # Security policy
└── STORE_LISTING.md                    # Microsoft Store listing copy
```

---

## 测试

```bash
# Run all tests
dotnet test

# Fixture smoke tests only
dotnet test --filter Category=FixtureSmoke

# Determinism tests (verifies reproducible deltas)
dotnet test --filter Category=Determinism

# With coverage
dotnet test --collect:"XPlat Code Coverage"

# Rust review. Line coverage has to stay above 90%.
cd rust
cargo llvm-cov --offline --locked --all-targets --fail-under-lines 90
```

---

## 键盘快捷键

| 快捷键 | 操作 |
|----------|--------|
| `Space` | 播放/暂停 |
| `Left` / `Right` | 向后/向前步进（1%） |
| `Shift+Left` / `Shift+Right` | 微步进（0.1%） |
| `Home` / `End` | 跳转到开始/结束 |
| `Up` / `+` | 增加播放速度 |
| `Down` / `-` | 降低播放速度 |
| `0` | 将速度重置为 1 倍 |
| `S` 或 `Ctrl+S` | 尝试将 PNG 写入“设置”中的导出文件夹，或者如果没有设置，则写入“文档/ScalarScope Exports”文件夹。不会显示任何通知。Ctrl+E 未映射。 |
| `1`–`6` | 请求显示路线概览、轨迹、标量、几何图形、比较和失败。不显示“主页”、“比较”、“指南”或“设置”。按下 1 不会打开“主页”。 |
| `?` | 打开帮助/指南 |

---

## 相关内容

- [手册](https://mcp-tool-shop-org.github.io/scalarscope/handbook/) — 审查指南
- [RESULTS_AND_LIMITATIONS.md](docs/RESULTS_AND_LIMITATIONS.md) — 完整的实验结果
- [CHANGELOG.md](CHANGELOG.md) — 发布历史记录
- [PRIVACY.md](PRIVACY.md) — 隐私政策
- [ROADMAP.md](ROADMAP.md) — 较旧的未验证计划，不是当前的审查

---

## 许可证

[MIT](LICENSE) — 版权所有 (c) 2025-2026 ScalarScope 项目 (mcp-tool-shop-org)

<p align="center">
  Built by <a href="https://mcp-tool-shop.github.io/">MCP Tool Shop</a>
</p>
