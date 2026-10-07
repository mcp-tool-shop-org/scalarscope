<p align="center">
  <a href="README.ja.md">日本語</a> | <a href="README.md">English</a> | <a href="README.es.md">Español</a> | <a href="README.fr.md">Français</a> | <a href="README.hi.md">हिन्दी</a> | <a href="README.it.md">Italiano</a> | <a href="README.pt-BR.md">Português (BR)</a>
</p>

<p align="center">
  <img src="https://raw.githubusercontent.com/mcp-tool-shop-org/brand/main/logos/ScalarScope/readme.png" width="400" alt="ScalarScope">
</p>

> [MCP Tool Shop](https://mcptoolshop.com) 的一部分。

<p align="center">
  <a href="https://github.com/mcp-tool-shop-org/scalarscope/actions/workflows/build.yml"><img src="https://github.com/mcp-tool-shop-org/scalarscope/actions/workflows/build.yml/badge.svg" alt="Build and Test"></a>
  <a href="https://codecov.io/gh/mcp-tool-shop-org/scalarscope"><img src="https://codecov.io/gh/mcp-tool-shop-org/scalarscope/graph/badge.svg" alt="Coverage"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/License-MIT-yellow.svg?style=flat-square" alt="License: MIT"></a>
  <a href="https://mcp-tool-shop-org.github.io/scalarscope/"><img src="https://img.shields.io/badge/Landing-page-blue?style=flat-square" alt="Landing page"></a>
  <a href="https://apps.microsoft.com/detail/9P3HT1PHBKQK"><img src="https://img.shields.io/badge/Microsoft%20Store-9P3HT1PHBKQK-0078D4?style=flat-square&logo=microsoft" alt="Microsoft Store"></a>
</p>

**比较两次机器学习运行，并查看比较结果的可信度。** ScalarScope 会打开两个推理轨迹、两个训练历史记录或两个 ASPIRE 几何导出文件。它会说明运行 B 与运行 A 的不同之处，为每个差异提供一个区间或其原因，并保留数据无法支持的内容。

版本 **3.1.0** 是用 Rust 重新编写的版本。它是 Microsoft Store 上的 Windows 应用程序，编号为 [9P3HT1PHBKQK](https://apps.microsoft.com/detail/9P3HT1PHBKQK)，软件包版本为 3.1.0.0，是对同一版本的更新。软件包名称 `mcp-tool-shop.ScalarScope` 和发布者 `CN=5305D976-6952-4F00-9C21-3A5DB090359F` 未更改，因此从 2.0 版本保存的文件、评论和设置将被保留。

## 信任模型

- **它读取的内容。** 您打开的文件。Store 软件包还会在其自己的 LocalState 文件夹中读取和写入 `comparison-log.json`、`preferences.json` 和 `workbench.json`，该文件夹是 2.0 版本使用的文件夹。未打包的构建不会写入任何内容。
- **它写入的内容。** 一个包、一张图片或一个会话记录，仅写入您选择的路径。
- **网络。** 没有帐户、没有遥测数据，也没有分析数据。该应用程序可以建立的唯一连接来自 Workbench 选项卡，并且仅在您按下“询问”按钮时才会连接到本地的 Ollama，地址为 `127.0.0.1`。Ollama 的云模型将被拒绝。没有任何数据会离开该机器。
- **包。** 匹配的 SHA-256 是内容检查，而不是签名。它表明字节是完整的。它没有说明谁编写了该文件。
- **插件。** 留在 2.0 文件夹中的插件不会被加载。

---

## 它所做的事情

**推理：两次运行的每步延迟。**
- **标题** 是 B/A，分别对应于 p50、p90 和 p99。每个比率都包含 95% 的移动块 Bootstrap 区间。
- **可以打开多个运行。** 然后，该区间还会重新采样整个运行，并且在下方被称为指示性，前提是每侧至少有三个运行。
- **如果某个百分位数没有足够的样本，** 则不会打印；p99 需要 368 个样本。
- **三个差异，** 每个差异都带有一个图块，指示它是否触发、保持静默或被保留，以及原因：
- **ΔF（新的异常）** 统计超出 5 个稳健偏差范围的稳定样本。它仅在 B 的超额值超出偶然范围时触发。
- **ΔO（可变性）** 在 B 相对于 A 的相对差异的区间不包含 1 时触发。
- **ΔTc（稳定化）** 为每个运行提供一个形状。它仅在两者都稳定并且它们的稳定范围不重叠时触发。
- **六个视图：** 系列、预热、带有您拖动阈值的分布、按百分位数显示的差异、频谱和热图。没有动画。

**训练。** 反向传播 `run_history.json` 被绘制为训练损失，旁边是保留损失、困惑度和任务指标。不会对其进行推理差异计算。

**几何：两次 ASPIRE 训练运行。**
- **它读取的内容：** [aspire-si](https://github.com/mcp-tool-shop-org/aspire-si) 编写的几何导出文件，包括其轨迹、评估器分数、特征向量和失败情况。
- **五个差异：** ΔF、ΔTc、ΔTd、ΔĀ（频谱浓度）和 ΔO。它们是从 2.0 版本移植而来，并与 2.0 版本的自身结果进行比较，更正记录在 [规范](docs/parity-and-beyond.spec.md) 中。
- **导出协议（架构 1.1）。** 导出可以声明其步骤是检查点 × 项目，而不是时间，或者其分数已重新播放。如果它这样做，则读取时间或分数下降的差异将被保留，并附带该原因，并且标题仅基于剩余的差异。较旧的导出文件将按原样读取。

**Workbench（工作台）。**
- **打开多个运行，** 这些运行在设置上有所不同，例如批处理大小或精度。
- **询问：** 一个可以调用工具的本地模型会根据公式测量这些运行，并提出每个设置的作用。
- **程序设置每个数字和结论。** 通过对每个运行集进行精确的秩检验来测试一个假设。跨集的结论仅在检查点处得出，采用 5% 的假发现率的 e-BH 方法。
- **您首先做出决定。** 您可以在询问之前编写自己的调用。模型的注释将显示在结论下方，并标记为它的内容。

**历史记录。** 在 Store 软件包中，您完成的比较将按 B 侧的数据集和模型进行分组。每个度量都绘制在这些评论中，并显示其水平发生变化的时刻。如果某个代码或环境更改与某个变化同时发生，则将其命名为巧合，而不是原因。日志会保留最近 40 条评论。

**包和保存的状态。**
- **保存包** 将评论写入 `.scbundle`。**打开包** 会以与保存时完全相同的方式显示它，处于评论模式。
- **2.0 包** 可以打开：包含其存储差异的比较包，以及标记为未验证的推理评论。
- **2.0 的设置** 将被保留：主题、色彩视觉调色板、高对比度、文本比例、最近文件限制和异常规则。
- **导出** 当前视图为 SVG，或将窗口导出为 PNG。

---

## 快速入门

### 从 Microsoft Store

1. 从 [Microsoft Store](https://apps.microsoft.com/detail/9P3HT1PHBKQK) 安装 **ScalarScope**。它需要 Windows 10 版本 1809（构建 17763）或更高版本，x64。
2. 在 **欢迎界面** 上，单击 **试用示例比较**。或者，单击 **比较两个运行** 并打开路径 A 和路径 B。
3. 阅读标题，然后单击一个图块以查看 **原因** 和 **显示给我**。

可以在 [`samples/`](samples/) 中找到一些示例文件，包括一个推理对、一个几何对和一个用于 Workbench 的运行文件夹。 [TESTING.md](TESTING.md) 说明了应该在哪里打开这些文件。

### 从源代码

```bash
git clone https://github.com/mcp-tool-shop-org/scalarscope.git
cd scalarscope
cargo run --release --manifest-path rust/Cargo.toml
```

`scalarscope <path A> <path B>` 在启动时打开一个对。

### 您可以打开的内容

| 类型 | 文件 |
|---|---|
| 推理 | 延迟 CSV 文件、基准 JSON 文件、Chrome 或 PyTorch 分析器跟踪文件（`.json` 或 `.json.gz`）、运行时日志、ScalarScope RunTrace JSON 文件或运行文件夹 |
| 训练 | 反向传播 `run_history.json` |
| 几何 | ASPIRE 几何数据导出（aspire-si，模式 1.x） |
| 审查 | `.scbundle`（来自 3.x 或 2.0） |

在[手册](https://mcp-tool-shop-org.github.io/scalarscope/handbook/getting-started/)的“入门”部分，每个文件都有一个简短的示例。

---

## 键盘快捷键

| 快捷键 | 操作 |
|---|---|
| `F1` | 指南 |
| `Ctrl+,` | 设置 |
| `Ctrl+H` | 欢迎 |
| `1`–`6` | 关于“比较”：系列、预热、分布、差异、频谱、热图 |
| `Esc` | 关闭“原因”面板 |

---

## 测试

```bash
# The review: 227 tests. Line coverage must stay above 90%.
cd rust
cargo test
cargo llvm-cov --locked --all-targets --fail-under-lines 90

# The .NET fixture tests: the 2.0 oracles, bundle compatibility and the version surfaces (134 tests)
dotnet test tests/ScalarScope.FixtureTests
```

`src/ScalarScope`中的 .NET 项目是 2.0 应用程序。它保留在仓库中，作为 3.x 版本进行检查的参考。存储包由 `rust/` 通过 `packaging/pack.ps1` 构建，并且 MAUI 项目拒绝以这种方式发布。

---

## 项目结构

```
scalarscope/
├── rust/                 # The app: ScalarScope 3.x (egui)
│   ├── src/              # review, geometry, workbench, history, bundles, settings, UI
│   └── tests/            # Rust tests and fixtures (real and simulated aspire-si exports, the knob folder)
├── samples/              # Files a first-time user or tester can open
├── packaging/            # AppxManifest.xml and pack.ps1 (the Store package)
├── src/ScalarScope/      # The 2.0 .NET app, kept as the reference
├── src/VortexKit/        # 2.0's visualization library
├── tests/                # .NET fixture tests and the 2.0 fixtures
├── site/                 # Landing page and handbook
└── docs/                 # The 3.x spec, receipts and release notes
```

---

## 相关

- [手册](https://mcp-tool-shop-org.github.io/scalarscope/handbook/)：审查指南
- [兼容性和后续发展](docs/parity-and-beyond.spec.md)：3.x 版本保留了 2.0 版本的哪些内容，它又做了哪些更改以及原因，并附有源代码
- [CHANGELOG.md](CHANGELOG.md)：发布历史
- [PRIVACY.md](PRIVACY.md)：隐私政策
- [TESTING.md](TESTING.md)：如何测试此版本
- [工作台](https://github.com/mcp-tool-shop-org/runforge)：与 RunForge 共享

---

## 许可证

[MIT](LICENSE)。版权所有，2025-2026，ScalarScope 项目 (mcp-tool-shop-org)

<p align="center">
  Built by <a href="https://mcp-tool-shop.github.io/">MCP Tool Shop</a>
</p>
