<p align="center">
  <a href="README.md">English</a> | <a href="README.zh.md">中文</a> | <a href="README.es.md">Español</a> | <a href="README.fr.md">Français</a> | <a href="README.hi.md">हिन्दी</a> | <a href="README.it.md">Italiano</a> | <a href="README.pt-BR.md">Português (BR)</a>
</p>

<p align="center">
  <img src="https://raw.githubusercontent.com/mcp-tool-shop-org/brand/main/logos/ScalarScope/readme.png" width="400" alt="ScalarScope">
</p>

[MCP Tool Shop](https://mcptoolshop.com)の一部

<p align="center">
  <a href="https://github.com/mcp-tool-shop-org/scalarscope/actions/workflows/build.yml"><img src="https://github.com/mcp-tool-shop-org/scalarscope/actions/workflows/build.yml/badge.svg" alt="Build and Test"></a>
  <a href="https://codecov.io/gh/mcp-tool-shop-org/scalarscope"><img src="https://codecov.io/gh/mcp-tool-shop-org/scalarscope/graph/badge.svg" alt="Coverage"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/License-MIT-yellow.svg?style=flat-square" alt="License: MIT"></a>
  <a href="https://mcp-tool-shop-org.github.io/scalarscope/"><img src="https://img.shields.io/badge/Landing-page-blue?style=flat-square" alt="Landing page"></a>
  <a href="https://apps.microsoft.com/detail/9P3HT1PHBKQK"><img src="https://img.shields.io/badge/Microsoft%20Store-9P3HT1PHBKQK-0078D4?style=flat-square&logo=microsoft" alt="Microsoft Store"></a>
</p>

**2つの機械学習実行のレビュー。** このリポジトリからビルドするアプリケーションは、`rust/`のRustプログラムです。リリースワークフローでは、そのプログラムを署名なしの3.0.0.0 MSIXとしてパッケージ化します。パッケージ名と発行元は同じままです。そのファイルはアップロードされません。ストアにアップロードされるまでは、ストアのコピーは以前の.NETパッケージのままです。

パッケージバージョン**3.0.0.0**。ストアの[9P3HT1PHBKQK](https://apps.microsoft.com/detail/9P3HT1PHBKQK)の更新では、名前は`mcp-tool-shop.ScalarScope`、発行元は`CN=5305D976-6952-4F00-9C21-3A5DB090359F`のままです。

## 信頼モデル

レビューでは、開いた2つのファイルが読み込まれます。パッケージ化された実行では、そのパッケージのLocalStateフォルダー内の`comparison-log.json`と`preferences.json`も読み書きされます。バンドルは、選択したパスにのみ書き込まれます。

これらのファイルはどこにも送信されません。アカウント、テレメトリ、分析は一切行われません。そのフォルダー内のプラグインはそのまま残され、ロードされません。バンドルのハッシュは、アーカイブされたバイトをチェックします。これは署名ではなく、コンテンツチェックであり、誰がファイルを作成したかはわかりません。

プログラムは、選択したファイルを読み取り、保存したバンドルを書き込むためのアクセス許可が必要です。

---

## なぜScalarScopeなのか？

ほとんどのMLチームは、ログを目視で確認します。ScalarScopeは、それを構造化された再現可能な比較に置き換えます。

以下に示す箇条書きは、公開された.NETパッケージについて説明します。Rustレビューでは、推論シリーズまたはバックプロパゲーションのトレーニング損失曲線が描画されます。推論ペアの場合、ΔFとΔO、および両方の側に安定状態の基準がある場合にのみΔTcが報告されます。両方の基準が存在する場合、シリーズはその場所に垂直線を描画します。Phase 7.2レイアウトに`.scbundle`を書き込みます。そのファイルを再度開くと、保存されたレビューが表示されます。ハッシュは、アーカイブされたファイルのバイトのSHA-256であり、`integrity.json`はシールです。一致するハッシュは、署名ではなく、コンテンツチェックです。パッケージ化された実行では、そのパッケージのLocalStateフォルダーに`comparison-log.json`と`preferences.json`が保存され、.NETアプリが書き込んだものと同じファイルです。シリーズの色は、保存された色覚モードに従います。最近のファイルと、そのフォルダーから保存されたビューは、ここで再度開きます。そのフォルダー内のプラグインはそのまま残され、ロードされません。パッケージ化されていない実行では、そのフォルダーは書き込まれません。

- **比較可能な比較** — 2つの推論トレースを並べてロードし、正確に何が変更されたかを確認します。
- **標準的なデルタ分析** — 5つのデルタタイプ（ΔTc、ΔO、ΔF、ΔĀ、ΔTd）は、差が統計的に有意な場合にのみトリガーされます。
- **実行時プリセット** — TFRTプリセットは、関連性のないメトリックを自動的に抑制するため、TensorFlow-TRTワークロードで重要なものに集中できます。
- **再現可能なバンドル** — SHA-256整合性、固定されたデルタ、および完全なプロベナンスメタデータを持つ`.scbundle`アーカイブをエクスポートします。
- **レビューモード** — 再計算せずにバンドルを開きます。一致するSHA-256は、署名ではなく、コンテンツチェックです。
- **プライバシー優先** — ゼロテレメトリ、ゼロ分析、すべてのデータは明示的にエクスポートしない限りローカルに保存されます。

---

## VortexKit

VortexKitは、`src/VortexKit`の可視化ライブラリです。これは、このリポジトリの一部です。NuGetには公開されていません。

時間同期された再生、アニメーション化されたSkiaSharpキャンバス、比較ビュー、注釈オーバーレイ、SVGおよびPNGエクスポート、およびセマンティックカラーシステムをカバーします。

---

## クイックスタート

### Rustレビュー

このリポジトリから：

```
cargo run --manifest-path rust/Cargo.toml
```

2つの推論ファイル、または2つのバックプロパゲーション`run_history.json`ファイルを開きます。推論ファイルは、レイテンシCSV、ベンチマークJSON、またはChromeトレースです。そのトレース内の完全な`ProfilerStep`は1つの推論であり、その中の演算は追加のサンプルではありません。ステップがないトレースでも、名前がTensorRTまたは推論を含むイベントを使用します。トレーニングファイルは、トレーニング損失として描画されます。保留された損失、パープレキシティ、およびタスクメトリックは、その曲線とともに表示されます。`final_loss`は、独自の数値として表示されます。推論ペアは、レイテンシシリーズからΔFとΔOを報告します。ΔTcは、両方のファイルに安定状態の基準がある場合にのみ報告されます。その基準がない場合、最後のステップは安定化時間とは見なされず、シリーズは安定状態の線を描画しません。ΔTdとΔĀは、推論ページに表示されません。推論デルタは、トレーニング履歴で計算されません。バンドルを保存すると、ページ上のレビューが書き込まれます。バンドルを開くと、保存されたレビューが再度表示されます。ハッシュは、.NET Phase 7.2チェックと一致します。一致するということは、バイトが破損していないということです。これは署名ではありません。

`packaging/pack.ps1`は、リリースバイナリから署名なしの`ScalarScope_3.0.0.0_x64.msix`をビルドします。パッケージ名は`mcp-tool-shop.ScalarScope`、発行元は`CN=5305D976-6952-4F00-9C21-3A5DB090359F`、アーキテクチャはx64です。アップロードされません。ストアのコピーは、以前の.NETパッケージのままです。

### Microsoft Storeから

1. [Microsoft Store](https://apps.microsoft.com/detail/9P3HT1PHBKQK)（ストアID：`9P3HT1PHBKQK`）から**ScalarScope**をインストールします。
2. **2つの実行を比較**をクリックします。
3. ベースライントレースをロードします。レイテンシCSV、ベンチマークJSON、またはプロファイラー`trace.json`です。
4. 同じ種類のファイルで、最適化されたトレースをロードします。
5. **比較**タブでデルタを確認します。
6. 再現可能な共有のために、`.scbundle`をエクスポートします。

### VortexKitの使用

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

## 機能

### デルタ分析 — 5つの標準的なデルタタイプ

すべての比較では、一連の標準的なデルタが生成されます。各デルタは、差が統計的に有意な場合にのみトリガーされます。関連性のないデルタは自動的に抑制されます。

| デルタ | 完全な名前 | 測定するもの | トリガーされるタイミング |
|-------|-----------|------------------|------------|
| **ΔTc** | 収束時間 | 安定したレイテンシーに到達するまでのステップ数 | 安定状態が異なるステップで到達（3ステップ以上の分離） |
| **ΔO** | 出力の変動 | 振動/実行時の不安定性 | しきい値以上の領域のスコアがノイズフロアを超えて異なる |
| **ΔF** | 失敗率 | 異常の頻度 | 実行間の失敗の頻度または種類が異なる |
| **ΔĀ** | 平均遅延 | 平均メトリック値 | 平均値に有意な差がある（TFRTプリセットでは抑制） |
| **ΔTd** | 合計時間 | 経過時間/構造的な出現 | 期間または優位性の開始が異なる（TFRTプリセットでは抑制） |

### 実行時プリセット — TFRT

組み込みの**TensorFlow-TRT**プリセット（`tensorflowrt-runtime-v1`）は、推論に固有のシグナル（遅延、スループット、メモリ、CPU/GPU負荷）をマッピングし、推論の比較に意味を持たないトレーニング専用のデルタ（ΔĀ、ΔTd）を抑制します。ガードレールは、ウォームアップが実行時間の50％を超えた場合、または集計された統計のみが利用可能な場合に警告を表示します。

### 再現可能なバンドル

結果を`.scbundle`アーカイブ（ComparisonBundle v1.0.0）としてエクスポートします。

- **`manifest.json`** — バンドルメタデータ、アプリバージョン、比較ラベル、アラインメントモード
- **`repro/repro.json`** — 入力フィンガープリント、プリセットハッシュ、決定性シード、環境情報
- **`findings/deltas.json`** — 信頼度スコア、アンカー、トリガータイプ付きの標準デルタ
- **`findings/why.json`** — 人間が読める説明、ガードレール、パラメータチップ
- **`findings/summary.md`** — 自動生成されたMarkdown要約
- **整合性** — すべてのファイルはSHA-256でハッシュ化されます。バンドルのハッシュは、署名ではなくコンテンツチェックです。

### レビューモード

再計算せずに任意の`.scbundle`を開きます。一致するSHA-256は、署名ではなくコンテンツチェックです。保存されたデルタは、保存されたものとして表示されます。

### VortexKit可視化フレームワーク

VortexKitは、`src/VortexKit`の可視化ライブラリです。このリポジトリに同梱されています。NuGetパッケージではありません。

| コンポーネント | その機能 |
|-----------|-------------|
| `PlaybackController` | 共有の0→1タイムライン（再生/一時停止/ステップ/ループ、速度プリセット（0.25倍〜4倍）、約60fpsのティック） |
| `AnimatedCanvas` | 時間同期による無効化、グリッド描画、タッチ/ドラッグイベント、座標ヘルパーを備えた抽象的な`SKCanvasView`ベース |
| `ITimeSeries<T>` / `TimeSeries<T>` | インデックス↔時間のマッピングとトレイル列挙を備えた汎用的な時系列 |
| `ExportService` | 単一フレームのPNG、フレームシーケンス（ffmpegのヒント付き）、および並べて比較できるエクスポート |
| `SvgExportService` | Inkscapeレイヤー、Catmull-Romスプライン、ヒートマップ、ベクトルフィールド、および4つのカラーパレット（デフォルト、ライト、ハイコントラスト、公開）を備えたフルベクトルのSVGエクスポート |
| `IAnnotation` | 理論的根拠と優先順位を備えた型付き注釈（フェーズ、警告、洞察、失敗、カスタム） |
| `VortexColors` | セマンティックカラーパレット — 背景レイヤー、アクセントセマンティクス、重大度コーディング、固有値パレット、線形補間/グラデーションヘルパー |

---

## インストール

### Microsoft Store（推奨）

**ストアID:** `9P3HT1PHBKQK`

[Microsoft Storeから入手](https://apps.microsoft.com/detail/9P3HT1PHBKQK)

Windows 10（ビルド17763）以降が必要です。

### ソースからのインストール

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

## プロジェクト構造

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

## テスト

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

## キーボードショートカット

| ショートカット | アクション |
|----------|--------|
| `Space` | 再生/一時停止 |
| `Left` / `Right` | 後退/前進（1％） |
| `Shift+Left` / `Shift+Right` | 微調整ステップ（0.1％） |
| `Home` / `End` | 開始/終了にジャンプ |
| `Up` / `+` | 再生速度を上げる |
| `Down` / `-` | 再生速度を下げる |
| `0` | 速度を1倍にリセット |
| `S`または`Ctrl+S` | 設定またはドキュメント/ScalarScope ExportsフォルダにPNGを書き込みます（設定されていない場合）。通知は表示されません。Ctrl+Eはマッピングされていません。 |
| `1`–`6` | ルート、軌跡、スカラー、ジオメトリ、比較、および失敗を要求します。ホーム、比較、ガイド、または設定ではありません。1を押すとホームは開かない。 |
| `?` | ヘルプ/ガイドを開く |

---

## 関連情報

- [ハンドブック](https://mcp-tool-shop-org.github.io/scalarscope/handbook/) — レビューのガイド
- [RESULTS_AND_LIMITATIONS.md](docs/RESULTS_AND_LIMITATIONS.md) — 完全な実験結果
- [CHANGELOG.md](CHANGELOG.md) — リリース履歴
- [PRIVACY.md](PRIVACY.md) — プライバシーポリシー
- [ROADMAP.md](ROADMAP.md) — 古い未チェックの計画、現在のレビューではない

---

## ライセンス

[MIT](LICENSE) — Copyright (c) 2025-2026 ScalarScope Project (mcp-tool-shop-org)

<p align="center">
  Built by <a href="https://mcp-tool-shop.github.io/">MCP Tool Shop</a>
</p>
