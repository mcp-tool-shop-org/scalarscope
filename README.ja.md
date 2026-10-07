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

**2つの機械学習の実行を比較し、その比較の信頼度を確認します。** ScalarScopeは、2つの推論トレース、2つのトレーニング履歴、または2つのASPIREジオメトリエクスポートを開きます。実行Bが実行Aとどのように異なるかを示し、各相違点に間隔またはその理由を付与し、データで裏付けられない内容は表示しません。

ScalarScope 3.xはRustで書き直されています。バージョン3.1.1は、Microsoft StoreのWindowsアプリであり、[9P3HT1PHBKQK](https://apps.microsoft.com/detail/9P3HT1PHBKQK)（パッケージ3.1.1.0）として提供され、同じリストのアップデートです。パッケージ名`mcp-tool-shop.ScalarScope`と発行者`CN=5305D976-6952-4F00-9C21-3A5DB090359F`は変更されていないため、2.0から保存されたファイル、レビュー、設定を引き継ぎます。

## 信頼モデル

- **読み込むファイル。** 開くファイル。Storeパッケージは、独自のLocalStateフォルダ（2.0で使用されていたフォルダ）内の`comparison-log.json`、`preferences.json`、および`workbench.json`も読み書きします。パッケージ化されていないビルドでは、これらのファイルは書き込まれません。
- **書き込むファイル。** バンドル、画像、またはセッションレコードを、選択したパスにのみ書き込みます。
- **ネットワーク。** アカウント、テレメトリ、分析はありません。アプリが接続できるのは、Workbenchタブからであり、**Ask**ボタンを押したときに、このコンピューターのローカルのOllama（`127.0.0.1`）に接続します。Ollamaのクラウドモデルは拒否されます。データはマシンから送信されません。
- **バンドル。** バンドルには比較する実行が含まれているため、それらの実行を共有してもよい場合にのみ共有してください。一致するSHA-256は、署名ではなく、コンテンツチェックです。バイトが破損していないことを示します。誰がファイルを書き込んだかは示しません。
- **プラグイン。** 2.0フォルダに残っているプラグインはロードされません。

---

## 機能

**推論：ステップごとの遅延の2回の実行。**
- **見出し**は、p50、p90、p99におけるB/Aの比率です。各比率には、95%の移動ブロックブートストラップ間隔が付属します。
- **各側で複数の実行**を開くことができます。その場合、間隔は実行全体を再サンプリングし、3つ未満の実行がある場合は、その結果は参考として扱われます。
- **十分なサンプルがないパーセンタイル**は表示されません。p99には368個のサンプルが必要です。
- **3つのデルタ。** それぞれに、それがトリガーされたか、静止したままか、または抑制されたか、そしてその理由を示すタイルがあります。
- **ΔF（新しい異常）**は、5つのロバストな標準偏差を超えて安定したサンプルをカウントします。これは、Bの過剰な値が偶然によるものではない場合にのみトリガーされます。
- **ΔO（変動性）**は、BのAに対する相対的な広がりを示す間隔が1を含まない場合にトリガーされます。
- **ΔTc（安定化）**は、各実行に形状を与えます。これは、両方の実行が安定し、その安定範囲が重ならない場合にのみトリガーされます。
- **6つのビュー：** シリーズ、ウォームアップ、閾値をドラッグできる分布、パーセンタイルごとの差分、スペクトル、ヒートマップ。いずれもアニメーションはしません。

**トレーニング。** バックプロパゲーション`run_history.json`は、トレーニング損失として描画され、その横に保留された損失、パープレキシティ、およびタスクメトリックが表示されます。推論デルタは、これに対して計算されません。

**ジオメトリ：2つのASPIREトレーニング実行。**
- **読み込むファイル：** [aspire-si](https://github.com/mcp-tool-shop-org/aspire-si)が書き出すジオメトリエクスポート、その軌跡、評価者のスコア、固有スペクトル、および失敗。
- **5つのデルタ：** ΔF、ΔTc、ΔTd、ΔĀ（スペクトル集中度）、およびΔO。これらは2.0から移植され、2.0自体の結果と比較され、その修正は[仕様](docs/parity-and-beyond.spec.md)に記録されます。
- **エクスポート契約（スキーマ1.1）。** エクスポートは、そのステップが時間ではなく、チェックポイント×アイテムであるか、またはそのスコアが再実行されたか、アイテムごとに固定されていることを示すことができます。その場合、ステップの順序またはスコアの変動に依存するデルタは、その理由とともに抑制され、ΔĀはチェックポイントごとに比較されます。スコアがアイテムごとに固定されている場合、ΔĀは実行ではなく、評価者を比較していることを示します。見出しは、存在するデルタのみに基づいて表示されます。古いエクスポートは、以前と同様に読み込まれます。

**Workbench。**
- **バッチサイズや精度などの設定が異なる複数の実行**を開きます。
- **Ask：** ツールを呼び出すことができるローカルモデルが、実行を数式で測定し、各設定が何をするかを提案します。
- **プログラムはすべての数値と結果を設定します。** 仮説は、各実行のセットに対して正確な順位テストによってテストされます。セット間の結果は、5%の偽陽性率でe-BHを使用して、チェックポイントでのみ表示されます。
- **最初にユーザーが決定します。** Askする前に、独自の呼び出しを記述できます。モデルの注釈は、結果の下に表示され、「モデルの言葉」とラベル付けされます。

**履歴。** Storeパッケージでは、完了した比較が、B側のデータセットとモデルによってグループ化されます。各メトリックは、それらのレビュー全体に描画され、そのレベルがシフトしたポイントが示されます。シフトの横にコードまたは環境の変更がある場合、それは偶然の一致であり、原因ではないと表示されます。ログには、最後の40件のレビューが保持されます。

**バンドルと保存された状態。**
- **Save bundle**は、レビューを`.scbundle`として書き込みます。**Open bundle**は、そのハッシュをチェックし、不一致の場合は拒否し、保存されたとおりに、レビューモードで表示します。今日のルールが保存されたジオメトリレビューを異なる方法で読み込む場合、その読み込み結果は、ラベルを付けてその下に表示されます。
- **2.0バンドル**は開くことができます。これには、保存されたデルタを含む比較バンドルと、検証されていないとマークされた推論レビューが含まれます。
- **2.0の設定**は引き継がれます。テーマ、色覚パレット、高コントラスト、テキストスケール、最近のファイル制限、および異常ルール。
- **現在のビューをSVGとして、またはウィンドウをPNGとしてエクスポートします。**

---

## クイックスタート

### Microsoft Storeから

1. **ScalarScope**を[Microsoft Store](https://apps.microsoft.com/detail/9P3HT1PHBKQK)からインストールします。Windows 10バージョン1809（ビルド17763）以降、x64が必要です。
2. **Welcome**画面で、**Try the sample comparison**をクリックします。または、**Compare two runs**をクリックして、パスAとパスBを開きます。
3. 見出しを読み、次に**Why**または**Show me**のタイルをクリックします。

試すためのサンプルファイル（推論ペア、ジオメトリペア、およびWorkbench用の実行のフォルダ）は、[`samples/`](samples/)にあります。[TESTING.md](TESTING.md)には、どこを開くかについての説明があります。

### ソースから

```bash
git clone https://github.com/mcp-tool-shop-org/scalarscope.git
cd scalarscope
cargo run --release --manifest-path rust/Cargo.toml
```

`scalarscope <path A> <path B>`は、起動時にペアを開きます。

### 開くことができるもの

| 種類 | ファイル |
|---|---|
| 推論 | 遅延を示すCSVファイル、ベンチマーク用のJSONファイル、ChromeまたはPyTorchプロファイラーのトレース（`.json`または`.json.gz`）、実行時のログ、ScalarScope RunTrace JSON、または実行フォルダー |
| トレーニング | 逆伝播処理 `run_history.json` |
| ジオメトリ | ASPIREジオメトリのエクスポート（aspire-si、スキーマ1.x） |
| レビュー | 3.xまたは2.0からの `.scbundle` |

[ハンドブック](https://mcp-tool-shop-org.github.io/scalarscope/handbook/getting-started/)の「Getting Started」には、各ファイルに関する簡単な例が記載されています。

---

## キーボードショートカット

| ショートカット | アクション |
|---|---|
| `F1` | ガイド |
| `Ctrl+,` | 設定 |
| `Ctrl+H` | ようこそ |
| `1`–`6` | 「比較」機能：シリーズ、ウォームアップ、分布、差分、スペクトル、ヒートマップ |
| `Esc` | 「Why」パネルを閉じる |

---

## テスト

```bash
# The review: 229 tests. Line coverage must stay above 90%.
cd rust
cargo test
cargo llvm-cov --locked --all-targets --fail-under-lines 90

# The .NET fixture tests: the 2.0 oracles, bundle compatibility and the version surfaces (135 tests)
dotnet test tests/ScalarScope.FixtureTests
```

`src/ScalarScope`内の.NETプロジェクトは、2.0アプリケーションです。これは、3.xが参照として使用されるように、リポジトリに保持されます。ストアパッケージは、`rust/`から`packaging/pack.ps1`によってビルドされ、MAUIプロジェクトは、そのアップロードとして公開されません。

---

## プロジェクト構造

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

## 関連情報

- [ハンドブック](https://mcp-tool-shop-org.github.io/scalarscope/handbook/): レビューのガイド
- [Parity and beyond](docs/parity-and-beyond.spec.md): 3.xが2.0から保持するもの、変更するもの、およびその理由（ソースを含む）
- [CHANGELOG.md](CHANGELOG.md): リリース履歴
- [PRIVACY.md](PRIVACY.md): プライバシーポリシー
- [TESTING.md](TESTING.md): このリリースのテスト方法
- [The workbench](https://github.com/mcp-tool-shop-org/runforge): RunForgeと共有

---

## ライセンス

[MIT](LICENSE)。Copyright (c) 2025-2026 ScalarScope Project (mcp-tool-shop-org)

<p align="center">
  Built by <a href="https://mcp-tool-shop.github.io/">MCP Tool Shop</a>
</p>
