<p align="center">
  <a href="README.ja.md">日本語</a> | <a href="README.zh.md">中文</a> | <a href="README.es.md">Español</a> | <a href="README.fr.md">Français</a> | <a href="README.md">English</a> | <a href="README.it.md">Italiano</a> | <a href="README.pt-BR.md">Português (BR)</a>
</p>

<p align="center">
  <img src="https://raw.githubusercontent.com/mcp-tool-shop-org/brand/main/logos/ScalarScope/readme.png" width="400" alt="ScalarScope">
</p>

[MCP टूल शॉप](https://mcptoolshop.com) का हिस्सा।

<p align="center">
  <a href="https://github.com/mcp-tool-shop-org/scalarscope/actions/workflows/build.yml"><img src="https://github.com/mcp-tool-shop-org/scalarscope/actions/workflows/build.yml/badge.svg" alt="Build and Test"></a>
  <a href="https://codecov.io/gh/mcp-tool-shop-org/scalarscope"><img src="https://codecov.io/gh/mcp-tool-shop-org/scalarscope/graph/badge.svg" alt="Coverage"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/License-MIT-yellow.svg?style=flat-square" alt="License: MIT"></a>
  <a href="https://mcp-tool-shop-org.github.io/scalarscope/"><img src="https://img.shields.io/badge/Landing-page-blue?style=flat-square" alt="Landing page"></a>
  <a href="https://apps.microsoft.com/detail/9P3HT1PHBKQK"><img src="https://img.shields.io/badge/Microsoft%20Store-9P3HT1PHBKQK-0078D4?style=flat-square&logo=microsoft" alt="Microsoft Store"></a>
</p>

**दो मशीन-लर्निंग रनों की समीक्षा।** इस रिपॉजिटरी से आप जो ऐप बनाते हैं, वह `rust/` में रस्ट प्रोग्राम है। रिलीज़ वर्कफ़्लो उस प्रोग्राम को बिना हस्ताक्षरित 3.0.0.0 MSIX के रूप में पैक करता है। पैकेज का नाम और प्रकाशक समान रहते हैं। वह फ़ाइल अपलोड नहीं की जाती है। स्टोर कॉपी अभी भी पिछली .NET पैकेज है जब तक कि वह अपलोड न हो जाए।

पैकेज संस्करण **3.0.0.0**। [9P3HT1PHBKQK](https://apps.microsoft.com/detail/9P3HT1PHBKQK) के स्टोर अपडेट नाम `mcp-tool-shop.ScalarScope` और प्रकाशक `CN=5305D976-6952-4F00-9C21-3A5DB090359F` को बनाए रखते हैं।

## ट्रस्ट मॉडल

समीक्षा उन दो फ़ाइलों को पढ़ती है जिन्हें आप खोलते हैं। एक पैकेज किए गए रन में उस पैकेज के LocalState फ़ोल्डर में `comparison-log.json` और `preferences.json` भी पढ़ा और लिखा जाता है। एक बंडल केवल उस पथ पर लिखा जाता है जिसे आप चुनते हैं।

यह उन फ़ाइलों को कहीं भी नहीं भेजता है। कोई खाता, कोई टेलीमेट्री और कोई एनालिटिक्स नहीं है। उस फ़ोल्डर में प्लगइन्स अपनी जगह पर छोड़ दिए जाते हैं और लोड नहीं किए जाते हैं। बंडल पर हैश संग्रहीत बाइट्स की जांच करता है। यह एक सामग्री जांच है, हस्ताक्षर नहीं, और यह नहीं बताता है कि फ़ाइल किसने लिखी।

प्रोग्राम को उन फ़ाइलों को पढ़ने और उस बंडल को लिखने की अनुमति की आवश्यकता होती है जिसे आप सहेजते हैं।

---

## स्केलरस्कोप क्यों?

अधिकांश एमएल टीमें लॉग को देखकर विश्लेषण करती हैं। स्केलरस्कोप इसे संरचित, पुनरुत्पादनीय तुलना से बदल देता है।

नीचे दी गई बुलेट बिंदुओं में प्रकाशित .NET पैकेज का वर्णन किया गया है। रस्ट समीक्षा एक अनुमान श्रृंखला या एक बैकप्रोपैगेट प्रशिक्षण-हानि वक्र बनाती है। एक अनुमान जोड़ी पर यह ΔF और ΔO, और केवल तभी ΔTc की रिपोर्ट करता है जब दोनों पक्षों में एक स्थिर-अवस्था मील का पत्थर हो। जब दोनों मील के पत्थर मौजूद होते हैं, तो श्रृंखला वहां एक ऊर्ध्वाधर रेखा खींचती है। यह चरण 7.2 लेआउट में `.scbundle` लिखता है। उस फ़ाइल को फिर से खोलने से संग्रहीत समीक्षा दिखाई देती है। हैश संग्रहीत फ़ाइल बाइट्स का SHA-256 है, और `integrity.json` सील है। मिलान हैश एक सामग्री जांच है, हस्ताक्षर नहीं। एक पैकेज किए गए रन में उस पैकेज के LocalState फ़ोल्डर में `comparison-log.json` और `preferences.json` होते हैं, जो वही फ़ाइलें हैं जिन्हें .NET ऐप ने लिखा था। श्रृंखला रंग सहेजे गए रंग-दृष्टि मोड का पालन करते हैं। उस फ़ोल्डर से हालिया फ़ाइलें और सहेजे गए दृश्य यहां फिर से खुलते हैं। उस फ़ोल्डर में प्लगइन्स अपनी जगह पर छोड़ दिए जाते हैं और लोड नहीं किए जाते हैं। एक अनपैकेज किए गए रन में वह फ़ोल्डर नहीं लिखा जाता है।

- **एक-से-एक तुलना** — दो अनुमान ट्रेस को एक साथ लोड करें और देखें कि वास्तव में क्या बदला
- **मानक डेल्टा विश्लेषण** — पांच डेल्टा प्रकार (ΔTc, ΔO, ΔF, ΔĀ, ΔTd) केवल तभी सक्रिय होते हैं जब अंतर सांख्यिकीय रूप से सार्थक होते हैं
- **रनटाइम प्रीसेट** — TFRT प्रीसेट अप्रासंगिक मेट्रिक्स को स्वचालित रूप से दबा देता है ताकि आप TensorFlow-TRT वर्कलोड के लिए महत्वपूर्ण चीज़ों पर ध्यान केंद्रित कर सकें
- **पुनरुत्पादनीय बंडल** — SHA-256 अखंडता, जमे हुए डेल्टा और पूर्ण उत्पत्ति मेटाडेटा के साथ `.scbundle` अभिलेखागार निर्यात करें
- **समीक्षा मोड** — पुनर्गणना किए बिना एक बंडल खोलें। मिलान SHA-256 एक सामग्री जांच है, हस्ताक्षर नहीं।
- **गोपनीयता पहले** — शून्य टेलीमेट्री, शून्य एनालिटिक्स, सभी डेटा स्थानीय रूप से रहता है जब तक कि आप स्पष्ट रूप से निर्यात नहीं करते

---

## वोर्टेक्सकिट

वोर्टेक्सकिट `src/VortexKit` में विज़ुअलाइज़ेशन लाइब्रेरी है। यह इस रिपॉजिटरी का हिस्सा है। इसे NuGet पर प्रकाशित नहीं किया गया है।

इसमें समय-सिंक्रनाइज़ प्लेबैक, एनिमेटेड SkiaSharp कैनवस, तुलना दृश्य, एनोटेशन ओवरले, SVG और PNG निर्यात और एक सिमेंटिक रंग प्रणाली शामिल है।

---

## त्वरित शुरुआत

### रस्ट समीक्षा

इस रिपॉजिटरी से:

```
cargo run --manifest-path rust/Cargo.toml
```

दो अनुमान फ़ाइलें या दो बैकप्रोपैगेट `run_history.json` फ़ाइलें खोलें। एक अनुमान फ़ाइल एक विलंबता CSV, एक बेंचमार्क JSON या एक क्रोम ट्रेस है। उस ट्रेस में एक पूर्ण `ProfilerStep` एक अनुमान है, और इसके अंदर के ऑप अतिरिक्त नमूने नहीं हैं। बिना किसी चरण के ट्रेस में भी उन घटनाओं का उपयोग किया जाता है जिनके नामों में TensorRT या अनुमान शामिल है। एक प्रशिक्षण फ़ाइल को प्रशिक्षण हानि के रूप में खींचा जाता है। होल्ड-आउट हानि, जटिलता और कार्य मेट्रिक्स उस वक्र के साथ होते हैं। `final_loss` को स्वयं एक संख्या के रूप में दिखाया गया है। एक अनुमान जोड़ी विलंबता श्रृंखला से ΔF और ΔO की रिपोर्ट करती है। ΔTc की रिपोर्ट केवल तभी की जाती है जब दोनों फ़ाइलों में एक स्थिर-अवस्था मील का पत्थर हो। उस मील के पत्थर के बिना, अंतिम चरण को स्थिरीकरण समय नहीं कहा जाता है, और श्रृंखला एक स्थिर-अवस्था रेखा नहीं खींचती है। ΔTd और ΔĀ अनुमान पृष्ठ से दूर रहते हैं। अनुमान डेल्टा की गणना प्रशिक्षण इतिहास पर नहीं की जाती है। सहेजें बंडल पृष्ठ पर समीक्षा लिखता है। खुला बंडल उस संग्रहीत समीक्षा को फिर से दिखाता है। हैश .NET चरण 7.2 जांच से मेल खाता है। एक मिलान का मतलब है कि बाइट्स अक्षुण्ण हैं। यह एक हस्ताक्षर नहीं है।

`packaging/pack.ps1` बिना हस्ताक्षरित `ScalarScope_3.0.0.0_x64.msix` को रिलीज़ बाइनरी से बनाता है। पैकेज का नाम `mcp-tool-shop.ScalarScope` है, प्रकाशक `CN=5305D976-6952-4F00-9C21-3A5DB090359F` है, और आर्किटेक्चर x64 है। यह अपलोड नहीं किया गया है। स्टोर पर कॉपी अभी भी पिछली .NET पैकेज है।

### माइक्रोसॉफ्ट स्टोर से

1. [माइक्रोसॉफ्ट स्टोर](https://apps.microsoft.com/detail/9P3HT1PHBKQK) (स्टोर आईडी: `9P3HT1PHBKQK`) से **स्केलरस्कोप** स्थापित करें
2. **दो रन की तुलना करें** पर क्लिक करें
3. एक आधारभूत ट्रेस लोड करें: एक विलंबता CSV, एक बेंचमार्क JSON या एक प्रोफाइलर `trace.json`
4. उसी प्रकार की फ़ाइल में अनुकूलित ट्रेस लोड करें
5. **तुलना** टैब में डेल्टा की समीक्षा करें
6. पुनरुत्पादनीय साझाकरण के लिए `.scbundle` निर्यात करें

### वोर्टेक्सकिट का उपयोग करना

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

## विशेषताएं

### डेल्टा विश्लेषण — पांच मानक डेल्टा प्रकार

प्रत्येक तुलना मानक डेल्टा का एक सेट उत्पन्न करती है। प्रत्येक डेल्टा केवल तभी सक्रिय होता है जब अंतर सांख्यिकीय रूप से सार्थक होता है; अप्रासंगिक डेल्टा स्वचालित रूप से दबा दिए जाते हैं।

| डेल्टा | पूर्ण नाम | यह क्या मापता है | कब सक्रिय होता है |
|-------|-----------|------------------|------------|
| **ΔTc** | अभिसरण समय | स्थिर विलंबता तक पहुंचने के लिए चरण | स्थिर अवस्था विभिन्न चरणों पर पहुंचती है (3+ चरण का अंतर) |
| **ΔO** | आउटपुट परिवर्तनशीलता | दोलन / रनटाइम अस्थिरता | थ्रेशोल्ड-से-ऊपर क्षेत्र स्कोर शोर तल से परे भिन्न होता है |
| **ΔF** | विफलता दर | विसंगति आवृत्ति | विफलता आवृत्ति या प्रकार रन के बीच भिन्न होते हैं |
| **ΔĀ** | औसत विलंबता | माध्य मीट्रिक मान | माध्य में सार्थक अंतर (TFRT प्रीसेट में दबाया गया) |
| **ΔTd** | कुल अवधि | वास्तविक समय / संरचनात्मक उद्भव | अवधि या प्रमुखता की शुरुआत में अंतर (TFRT प्रीसेट में दबाया गया) |

### रनटाइम प्रीसेट — TFRT

अंतर्निहित **TensorFlow-TRT** प्रीसेट (`tensorflowrt-runtime-v1`) अनुमान-विशिष्ट संकेतों (विलंबता, थ्रूपुट, मेमोरी, CPU/GPU लोड) को मैप करता है और केवल प्रशिक्षण-विशिष्ट अंतरों (ΔĀ, ΔTd) को दबा देता है जिनका अनुमान तुलना के लिए कोई अर्थ नहीं है। गार्डरेल चेतावनी देते हैं जब वार्मअप रन के 50% से अधिक हो जाता है या जब केवल एकत्रित आंकड़े उपलब्ध होते हैं।

### पुनरुत्पादनीय बंडल

परिणामों को `.scbundle` अभिलेखागार के रूप में निर्यात करें (ComparisonBundle v1.0.0):

- **`manifest.json`** — बंडल मेटाडेटा, ऐप संस्करण, तुलना लेबल, संरेखण मोड
- **`repro/repro.json`** — इनपुट फिंगरप्रिंट, प्रीसेट हैश, नियतिवादी बीज, पर्यावरण जानकारी
- **`findings/deltas.json`** — आत्मविश्वास स्कोर, एंकर और ट्रिगर प्रकार के साथ विहित अंतर
- **`findings/why.json`** — मानव-पठनीय स्पष्टीकरण, गार्डरेल, पैरामीटर स्निपेट
- **`findings/summary.md`** — स्वचालित रूप से उत्पन्न मार्कडाउन सारांश
- **अखंडता** — प्रत्येक फ़ाइल SHA-256 के साथ हैश की गई। बंडल हैश एक सामग्री जांच है, हस्ताक्षर नहीं।

### समीक्षा मोड

किसी भी `.scbundle` को पुनर्गणना किए बिना खोलें। मिलान करने वाला SHA-256 एक सामग्री जांच है, हस्ताक्षर नहीं। संग्रहीत अंतरों को संग्रहीत के रूप में दिखाया जाता है।

### वोर्टेक्सकिट विज़ुअलाइज़ेशन फ्रेमवर्क

वोर्टेक्सकिट `src/VortexKit` में विज़ुअलाइज़ेशन लाइब्रेरी है। यह इस रिपॉजिटरी के साथ आता है। यह NuGet पैकेज नहीं है।

| घटक | यह क्या करता है |
|-----------|-------------|
| `PlaybackController` | साझा 0→1 टाइमलाइन जिसमें प्ले/पॉज़/स्टेप/लूप, गति प्रीसेट (0.25x—4x), ~60 एफपीएस टिक शामिल हैं |
| `AnimatedCanvas` | समय-सिंक्रनाइज़ अमान्यकरण, ग्रिड ड्राइंग, टच/ड्रैग इवेंट, समन्वय सहायकों के साथ सार `SKCanvasView` आधार |
| `ITimeSeries<T>` / `TimeSeries<T>` | इंडेक्स↔समय मैपिंग और ट्रेल गणना के साथ सामान्य समय-श्रृंखला |
| `ExportService` | एकल-फ़्रेम PNG, फ़्रेम अनुक्रम (ffmpeg संकेतों के साथ), और साइड-बाय-साइड तुलना निर्यात |
| `SvgExportService` | इन्केस्केप लेयर्स, कैटमुल-रोम स्प्लिन, हीटमैप, वेक्टर फ़ील्ड और चार रंग पैलेट (डिफ़ॉल्ट, लाइट, हाईकंट्रास्ट, प्रकाशन) के साथ पूर्ण-वेक्टर SVG निर्यात |
| `IAnnotation` | टाइप किए गए एनोटेशन (फेज, चेतावनी, अंतर्दृष्टि, विफलता, कस्टम) जिसमें सैद्धांतिक आधार और प्राथमिकता शामिल है |
| `VortexColors` | सिमेंटिक रंग पैलेट — पृष्ठभूमि परतें, उच्चारण सिमेंटिक्स, गंभीरता कोडिंग, आइगेनवैल्यू पैलेट, लेर्प/ग्रेडिएंट सहायक |

---

## स्थापना

### माइक्रोसॉफ्ट स्टोर (अनुशंसित)

**स्टोर आईडी:** `9P3HT1PHBKQK`

[इसे माइक्रोसॉफ्ट स्टोर से प्राप्त करें](https://apps.microsoft.com/detail/9P3HT1PHBKQK)

विंडोज 10 (बिल्ड 17763) या बाद का संस्करण आवश्यक है।

### स्रोत से

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

## परियोजना संरचना

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

## परीक्षण

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

## कीबोर्ड शॉर्टकट

| शॉर्टकट | क्रिया |
|----------|--------|
| `Space` | प्ले / पॉज़ |
| `Left` / `Right` | पिछली ओर / आगे की ओर कदम (1%) |
| `Shift+Left` / `Shift+Right` | बारीक कदम (0.1%) |
| `Home` / `End` | शुरुआत / अंत पर जाएं |
| `Up` / `+` | प्लेबैक गति बढ़ाएँ |
| `Down` / `-` | प्लेबैक गति घटाएँ |
| `0` | गति को 1x पर रीसेट करें |
| `S` या `Ctrl+S` | सेटिंग्स से निर्यात फ़ोल्डर में PNG लिखने का प्रयास करता है, या जब कोई सेट नहीं है तो Documents/ScalarScope Exports में। कोई सूचना नहीं दिखाई जाती है। Ctrl+E मैप नहीं किया गया है। |
| `1`–`6` | मार्गों, प्रक्षेपवक्र, स्केलर, ज्यामिति, तुलना और विफलताओं का अवलोकन अनुरोध करें। होम, तुलना, गाइड या सेटिंग्स नहीं। 1 दबाने से होम नहीं खुलता है। |
| `?` | सहायता / गाइड खोलें |

---

## संबंधित

- [हैंडबुक](https://mcp-tool-shop-org.github.io/scalarscope/handbook/) — समीक्षा के लिए गाइड
- [RESULTS_AND_LIMITATIONS.md](docs/RESULTS_AND_LIMITATIONS.md) — पूर्ण प्रयोगात्मक परिणाम
- [CHANGELOG.md](CHANGELOG.md) — रिलीज़ इतिहास
- [PRIVACY.md](PRIVACY.md) — गोपनीयता नीति
- [ROADMAP.md](ROADMAP.md) — एक पुरानी अप्रयुक्त योजना, वर्तमान समीक्षा नहीं

---

## लाइसेंस

[MIT](LICENSE) — कॉपीराइट (c) 2025-2026 स्केलरस्कोप प्रोजेक्ट (mcp-tool-shop-org)

<p align="center">
  Built by <a href="https://mcp-tool-shop.github.io/">MCP Tool Shop</a>
</p>
