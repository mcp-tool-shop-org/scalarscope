<p align="center">
  <a href="README.ja.md">日本語</a> | <a href="README.zh.md">中文</a> | <a href="README.es.md">Español</a> | <a href="README.fr.md">Français</a> | <a href="README.hi.md">हिन्दी</a> | <a href="README.it.md">Italiano</a> | <a href="README.md">English</a>
</p>

<p align="center">
  <img src="https://raw.githubusercontent.com/mcp-tool-shop-org/brand/main/logos/ScalarScope/readme.png" width="400" alt="ScalarScope">
</p>

Parte do [MCP Tool Shop](https://mcptoolshop.com)

<p align="center">
  <a href="https://github.com/mcp-tool-shop-org/scalarscope/actions/workflows/build.yml"><img src="https://github.com/mcp-tool-shop-org/scalarscope/actions/workflows/build.yml/badge.svg" alt="Build and Test"></a>
  <a href="https://codecov.io/gh/mcp-tool-shop-org/scalarscope"><img src="https://codecov.io/gh/mcp-tool-shop-org/scalarscope/graph/badge.svg" alt="Coverage"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/License-MIT-yellow.svg?style=flat-square" alt="License: MIT"></a>
  <a href="https://mcp-tool-shop-org.github.io/scalarscope/"><img src="https://img.shields.io/badge/Landing-page-blue?style=flat-square" alt="Landing page"></a>
  <a href="https://apps.microsoft.com/detail/9P3HT1PHBKQK"><img src="https://img.shields.io/badge/Microsoft%20Store-9P3HT1PHBKQK-0078D4?style=flat-square&logo=microsoft" alt="Microsoft Store"></a>
</p>

**Uma análise de duas execuções de aprendizado de máquina.** O aplicativo que você cria a partir deste repositório é o programa Rust em `rust/`. O fluxo de lançamento empacota esse programa como o MSIX 3.0.0.0 não assinado. O nome do pacote e o editor permanecem os mesmos. Esse arquivo não é carregado. A cópia na loja ainda é o pacote .NET anterior até que esse upload seja feito.

Versão do pacote **3.0.0.0**. As atualizações da loja de [9P3HT1PHBKQK](https://apps.microsoft.com/detail/9P3HT1PHBKQK) mantêm o nome `mcp-tool-shop.ScalarScope` e o editor `CN=5305D976-6952-4F00-9C21-3A5DB090359F`.

## Modelo de confiança

A análise lê os dois arquivos que você abre. Uma execução empacotada também lê e grava `comparison-log.json` e `preferences.json` na pasta LocalState desse pacote. Um pacote é gravado apenas no caminho que você selecionar.

Ele não envia esses arquivos para lugar nenhum. Não há conta, nem telemetria e nem análise. Os plugins nessa pasta permanecem no local e não são carregados. O hash em um pacote verifica os bytes arquivados. É uma verificação de conteúdo, não uma assinatura, e não indica quem escreveu o arquivo.

O programa precisa de permissão para ler os arquivos que você selecionar e para gravar o pacote que você salvar.

---

## Por que ScalarScope?

A maioria das equipes de aprendizado de máquina analisa os logs visualmente. O ScalarScope substitui isso por uma comparação estruturada e reproduzível.

Os tópicos abaixo descrevem o pacote .NET publicado. A análise Rust gera uma série de inferências ou uma curva de perda de treinamento retropropagada. Em um par de inferências, ele relata ΔF e ΔO, e ΔTc apenas quando ambos os lados têm um marco de estado estável. Quando ambos os marcos existem, a série desenha uma linha vertical ali. Ele grava um `.scbundle` no layout da Fase 7.2. Reabrir esse arquivo mostra a análise armazenada. O hash é SHA-256 dos bytes do arquivo arquivado, e `integrity.json` é o selo. Um hash correspondente é uma verificação de conteúdo, não uma assinatura. Uma execução empacotada mantém `comparison-log.json` e `preferences.json` na pasta LocalState desse pacote, os mesmos arquivos que o aplicativo .NET gravou. As cores da série seguem um modo de visão de cores salvo. Arquivos recentes e visualizações salvas dessa pasta são reabertos aqui. Os plugins nessa pasta permanecem no local e não são carregados. Uma execução não empacotada não grava essa pasta.

- **Comparação direta** — Carregue duas séries de inferências lado a lado e veja exatamente o que mudou
- **Análise delta canônica** — Cinco tipos de delta (ΔTc, ΔO, ΔF, ΔĀ, ΔTd) são ativados apenas quando as diferenças são estatisticamente significativas
- **Predefinições de tempo de execução** — A predefinição TFRT suprime automaticamente as métricas irrelevantes para que você se concentre no que é importante para as cargas de trabalho TensorFlow-TRT
- **Pacotes reproduzíveis** — Exporte arquivos `.scbundle` com integridade SHA-256, deltas congelados e metadados completos de linhagem
- **Modo de análise** — Abra um pacote sem recalcular. Um SHA-256 correspondente é uma verificação de conteúdo, não uma assinatura.
- **Privacidade em primeiro lugar** — Telemetria zero, análise zero, todos os dados permanecem locais, a menos que você exporte explicitamente

---

## VortexKit

VortexKit é a biblioteca de visualização em `src/VortexKit`. Faz parte deste repositório. Não é publicado no NuGet.

Ele abrange reprodução sincronizada no tempo, telas SkiaSharp animadas, visualizações de comparação, sobreposições de anotação, exportação SVG e PNG e um sistema de cores semântico.

---

## Guia rápido

### Análise Rust

Deste repositório:

```
cargo run --manifest-path rust/Cargo.toml
```

Abra dois arquivos de inferência ou dois arquivos de retropropagação `run_history.json`. Um arquivo de inferência é um CSV de latência, um JSON de benchmark ou um rastreamento do Chrome. Um `ProfilerStep` completo nesse rastreamento é uma inferência, e as operações dentro dele não são amostras extras. Um rastreamento sem etapa ainda usa eventos cujos nomes contêm TensorRT ou inferência. Um arquivo de treinamento é desenhado como perda de treinamento. Perda retida, perplexidade e métricas de tarefa são exibidas com essa curva. `final_loss` é mostrado como seu próprio número. Um par de inferências relata ΔF e ΔO da série de latência. ΔTc é relatado apenas quando ambos os arquivos têm um marco de estado estável. Sem esse marco, a última etapa não é chamada de tempo de estabilização, e a série não desenha uma linha de estado estável. ΔTd e ΔĀ não são exibidos na página de inferência. Deltas de inferência não são calculados em um histórico de treinamento. Salvar o pacote grava a análise na página. Abrir o pacote mostra essa análise armazenada novamente. O hash corresponde à verificação da Fase 7.2 do .NET. Uma correspondência significa que os bytes estão intactos. Não é uma assinatura.

`packaging/pack.ps1` cria o `ScalarScope_3.0.0.0_x64.msix` não assinado do binário de lançamento. O nome do pacote é `mcp-tool-shop.ScalarScope`, o editor é `CN=5305D976-6952-4F00-9C21-3A5DB090359F` e a arquitetura é x64. Não é carregado. A cópia na loja ainda é o pacote .NET anterior.

### Da Microsoft Store

1. Instale o **ScalarScope** na [Microsoft Store](https://apps.microsoft.com/detail/9P3HT1PHBKQK) (ID da loja: `9P3HT1PHBKQK`)
2. Clique em **Comparar duas execuções**
3. Carregue um rastreamento de referência: um CSV de latência, um JSON de benchmark ou um perfilador `trace.json`
4. Carregue o rastreamento otimizado no mesmo tipo de arquivo
5. Analise os deltas na guia **Comparar**
6. Exporte um `.scbundle` para compartilhamento reproduzível

### Usando o VortexKit

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

## Recursos

### Análise de delta — Cinco tipos de delta canônicos

Cada comparação produz um conjunto de deltas canônicos. Cada delta é ativado apenas quando a diferença é estatisticamente significativa; deltas irrelevantes são suprimidos automaticamente.

| Delta | Nome completo | O que ele mede | Ativado quando |
|-------|-----------|------------------|------------|
| **ΔTc** | Tempo de convergência | Etapas para atingir a latência estável | Estado estável alcançado em etapas diferentes (separação de 3+ etapas) |
| **ΔO** | Variabilidade de saída | Oscilação / instabilidade em tempo de execução | A pontuação acima do limite difere além do ruído |
| **ΔF** | Taxa de falha | Frequência de anomalia | A frequência ou o tipo de falha diferem entre as execuções |
| **ΔĀ** | Latência Média | Valor médio da métrica | A diferença média é significativa (suprimida na configuração predefinida TFRT) |
| **ΔTd** | Duração Total | Tempo decorrido / emergência estrutural | A duração ou o início da dominância diferem (suprimido na configuração predefinida TFRT) |

### Configurações Predefinidas de Tempo de Execução — TFRT

A configuração predefinida integrada **TensorFlow-TRT** (`tensorflowrt-runtime-v1`) mapeia sinais específicos da inferência (latência, taxa de transferência, memória, carga da CPU/GPU) e suprime as diferenças que são relevantes apenas para o treinamento (ΔĀ, ΔTd), que não têm significado para a comparação da inferência. As salvaguardas alertam quando o período de aquecimento excede 50% da execução ou quando apenas estatísticas agregadas estão disponíveis.

### Pacotes Reproduzíveis

Exporte os resultados como arquivos `.scbundle` (ComparisonBundle v1.0.0):

- **`manifest.json`** — metadados do pacote, versão do aplicativo, rótulos de comparação, modo de alinhamento
- **`repro/repro.json`** — impressões digitais de entrada, hash da configuração predefinida, semente de determinismo, informações do ambiente
- **`findings/deltas.json`** — diferenças canônicas com pontuações de confiança, âncoras e tipos de gatilho
- **`findings/why.json`** — explicações legíveis por humanos, salvaguardas, parâmetros
- **`findings/summary.md`** — resumo gerado automaticamente em Markdown
- **Integridade** — cada arquivo tem um hash SHA-256. O hash do pacote é uma verificação de conteúdo, não uma assinatura.

### Modo de Revisão

Abra qualquer `.scbundle` sem recalcular. Um SHA-256 correspondente é uma verificação de conteúdo, não uma assinatura. As diferenças armazenadas são exibidas como armazenadas.

### Estrutura de Visualização VortexKit

VortexKit é a biblioteca de visualização em `src/VortexKit`. Ela é fornecida com este repositório. Não é um pacote NuGet.

| Componente | O que ele faz |
|-----------|-------------|
| `PlaybackController` | Linha do tempo compartilhada de 0 a 1 com reprodução/pausa/avanço/repetição, configurações predefinidas de velocidade (0,25x a 4x), taxa de atualização de ~60 fps |
| `AnimatedCanvas` | Base abstrata `SKCanvasView` com invalidação sincronizada com o tempo, desenho de grade, eventos de toque/arraste, auxiliares de coordenadas |
| `ITimeSeries<T>` / `TimeSeries<T>` | Série temporal genérica com mapeamento de índice para tempo e enumeração de trilhas |
| `ExportService` | PNG de quadro único, sequências de quadros (com dicas do ffmpeg) e exportação de comparação lado a lado |
| `SvgExportService` | Exportação SVG totalmente vetorial com camadas do Inkscape, splines de Catmull-Rom, mapas de calor, campos vetoriais e quatro paletas de cores (Padrão, Claro, Alto Contraste, Publicação) |
| `IAnnotation` | Anotações tipadas (Fase, Aviso, Informação, Falha, Personalizado) com base teórica e prioridade |
| `VortexColors` | Paleta de cores semântica — camadas de fundo, semântica de destaque, codificação de gravidade, paleta de autovalores, auxiliares de interpolação/gradiente |

---

## Instalação

### Microsoft Store (recomendado)

**ID da Loja:** `9P3HT1PHBKQK`

[Obtenha-o na Microsoft Store](https://apps.microsoft.com/detail/9P3HT1PHBKQK)

Requer Windows 10 (versão 17763) ou posterior.

### A partir do Código Fonte

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

## Estrutura do Projeto

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

## Testes

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

## Atalhos de Teclado

| Atalho | Ação |
|----------|--------|
| `Space` | Reproduzir / Pausar |
| `Left` / `Right` | Avançar / retroceder (1%) |
| `Shift+Left` / `Shift+Right` | Avanço fino (0,1%) |
| `Home` / `End` | Ir para o início / fim |
| `Up` / `+` | Aumentar a velocidade de reprodução |
| `Down` / `-` | Diminuir a velocidade de reprodução |
| `0` | Redefinir a velocidade para 1x |
| `S` ou `Ctrl+S` | Tenta gravar um PNG na pasta de exportação das Configurações ou em Documents/ScalarScope Exports quando nenhuma estiver definida. Nenhuma notificação é exibida. Ctrl+E não está mapeado. |
| `1`–`6` | Solicita a visão geral das rotas, trajetória, escalares, geometria, comparação e falhas. Não é a página inicial, Comparar, Guia ou Configurações. Pressionar 1 não abre a página inicial. |
| `?` | Abrir ajuda / guia |

---

## Relacionado

- [Manual](https://mcp-tool-shop-org.github.io/scalarscope/handbook/) — O guia para a revisão
- [RESULTS_AND_LIMITATIONS.md](docs/RESULTS_AND_LIMITATIONS.md) — Resultados experimentais completos
- [CHANGELOG.md](CHANGELOG.md) — Histórico de lançamentos
- [PRIVACY.md](PRIVACY.md) — Política de privacidade
- [ROADMAP.md](ROADMAP.md) — Um plano mais antigo não verificado, não a revisão atual

---

## Licença

[MIT](LICENSE) — Copyright (c) 2025-2026 Projeto ScalarScope (mcp-tool-shop-org)

<p align="center">
  Built by <a href="https://mcp-tool-shop.github.io/">MCP Tool Shop</a>
</p>
