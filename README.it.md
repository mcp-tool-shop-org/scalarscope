<p align="center">
  <a href="README.ja.md">日本語</a> | <a href="README.zh.md">中文</a> | <a href="README.es.md">Español</a> | <a href="README.fr.md">Français</a> | <a href="README.hi.md">हिन्दी</a> | <a href="README.md">English</a> | <a href="README.pt-BR.md">Português (BR)</a>
</p>

<p align="center">
  <img src="https://raw.githubusercontent.com/mcp-tool-shop-org/brand/main/logos/ScalarScope/readme.png" width="400" alt="ScalarScope">
</p>

> Parte di [MCP Tool Shop](https://mcptoolshop.com)

<p align="center">
  <a href="https://github.com/mcp-tool-shop-org/scalarscope/actions/workflows/build.yml"><img src="https://github.com/mcp-tool-shop-org/scalarscope/actions/workflows/build.yml/badge.svg" alt="Build and Test"></a>
  <a href="https://codecov.io/gh/mcp-tool-shop-org/scalarscope"><img src="https://codecov.io/gh/mcp-tool-shop-org/scalarscope/graph/badge.svg" alt="Coverage"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/License-MIT-yellow.svg?style=flat-square" alt="License: MIT"></a>
  <a href="https://mcp-tool-shop-org.github.io/scalarscope/"><img src="https://img.shields.io/badge/Landing-page-blue?style=flat-square" alt="Landing page"></a>
  <a href="https://apps.microsoft.com/detail/9P3HT1PHBKQK"><img src="https://img.shields.io/badge/Microsoft%20Store-9P3HT1PHBKQK-0078D4?style=flat-square&logo=microsoft" alt="Microsoft Store"></a>
</p>

**Confronta due esecuzioni di machine learning e verifica l'affidabilità del confronto.** ScalarScope apre due tracce di inferenza, due cronologie di addestramento o due esportazioni di geometria ASPIRE. Indica in cosa l'esecuzione B differisce dall'esecuzione A, assegna a ciascuna differenza un intervallo o una motivazione e omette ciò che i dati non possono supportare.

ScalarScope 3.x è una riscrittura in Rust. La versione **3.1.1** è l'app per Windows disponibile su Microsoft Store all'indirizzo [9P3HT1PHBKQK](https://apps.microsoft.com/detail/9P3HT1PHBKQK), pacchetto 3.1.1.0, un aggiornamento della stessa versione. Il nome del pacchetto `mcp-tool-shop.ScalarScope` e l'editore `CN=5305D976-6952-4F00-9C21-3A5DB090359F` rimangono invariati, quindi i file, le recensioni salvate e le impostazioni della versione 2.0 vengono mantenuti.

## Modello di affidabilità

- **Cosa legge.** I file che si aprono. Il pacchetto dello Store legge e scrive anche `comparison-log.json`, `preferences.json` e `workbench.json` nella propria cartella LocalState, la stessa cartella utilizzata dalla versione 2.0. Una versione non impacchettata non scrive nessuno di questi file.
- **Cosa scrive.** Un pacchetto, un'immagine o una registrazione di sessione, solo in un percorso specificato.
- **Rete.** Non sono presenti account, telemetria o analisi. L'unica connessione che l'app può stabilire è dalla scheda Workbench, e solo quando si preme **Ask**: a un'istanza locale di Ollama all'indirizzo `127.0.0.1` su questo computer. I modelli cloud di Ollama non sono supportati. Nessun dato lascia la macchina.
- **Pacchetti (bundle).** Un pacchetto contiene le esecuzioni che confronta: condividetelo solo se condividereste quelle esecuzioni. Un SHA-256 corrispondente è un controllo del contenuto, non una firma. Indica che i byte sono intatti. Non indica chi ha scritto il file.
- **Plugin.** I plugin presenti nella cartella 2.0 non vengono caricati.

---

## Cosa fa

**Inferenza: due esecuzioni di latenza per passaggio.**
- **Il titolo** è B/A a p50, p90 e p99. Ogni rapporto include un intervallo bootstrap a blocchi mobili del 95%.
- **È possibile aprire diverse esecuzioni per ciascun lato.** L'intervallo quindi ricampiona anche le intere esecuzioni e, quando sono inferiori a tre per lato, viene definito indicativo.
- **Un percentile con un numero di campioni insufficiente** non viene visualizzato; p99 richiede 368.
- **Tre delta,** ciascuno con una casella che indica se è stato attivato, è rimasto inattivo o è stato omesso, e perché:
- **ΔF (nuove anomalie)** conta i campioni stabili oltre 5 deviazioni robuste. Si attiva solo quando l'eccesso di B supera il valore casuale.
- **ΔO (variabilità)** si attiva quando l'intervallo sulla variazione relativa di B rispetto ad A esclude 1.
- **ΔTc (stabilizzazione)** assegna a ciascuna esecuzione una forma. Si attiva solo quando entrambe si stabilizzano e i loro intervalli di stabilizzazione non si sovrappongono.
- **Sei viste:** Serie, Riscaldamento, Distribuzione con una soglia che si può trascinare, Differenza per percentile, Spettro e Mappa di calore. Nessuna di esse è animata.

**Addestramento.** Una retropropagazione `run_history.json` viene visualizzata come perdita di addestramento, con la perdita, la perplessità e le metriche delle attività mantenute separate. I delta di inferenza non vengono calcolati su di essa.

**Geometria: due esecuzioni di addestramento ASPIRE.**
- **Cosa legge:** l'esportazione di geometria che [aspire-si](https://github.com/mcp-tool-shop-org/aspire-si) scrive, con la sua traiettoria, i punteggi dell'evaluator, lo spettro degli autovalori e i fallimenti.
- **Cinque delta:** ΔF, ΔTc, ΔTd, ΔĀ (concentrazione dello spettro) e ΔO. Sono stati portati dalla versione 2.0 e confrontati con i risultati della versione 2.0, con le correzioni registrate in [questa specifica](docs/parity-and-beyond.spec.md).
- **Contratto di esportazione (schema 1.1).** Un'esportazione può indicare che i suoi passaggi sono checkpoint × elementi anziché tempo, oppure che i suoi punteggi sono stati riprodotti o sono fissi per elemento. In tal caso, i delta che dipendono dall'ordine dei passaggi o dai cali dei punteggi vengono omessi con tale motivazione e ΔĀ viene confrontato checkpoint per checkpoint. Quando i punteggi sono fissi per elemento, ΔĀ indica che confronta gli evaluator, non le esecuzioni. Il titolo si riferisce solo ai delta che vengono visualizzati. Le esportazioni più vecchie vengono lette come prima.

**Workbench.**
- **Aprire molte esecuzioni** che differiscono in un'impostazione, come la dimensione del batch o la precisione.
- **Ask:** un modello locale in grado di chiamare strumenti misura le esecuzioni con formule e propone cosa fa ciascuna impostazione.
- **Il programma imposta ogni numero e verdetto.** Un'ipotesi viene testata mediante un test di rango esatto su ciascun set di esecuzioni. I verdetti tra i set vengono forniti solo ai checkpoint, mediante e-BH con un tasso di falsi positivi del 5%.
- **Prima la tua decisione.** È possibile scrivere la propria richiesta prima di chiedere. La nota del modello viene visualizzata sotto i verdetti, con l'etichetta "parole del modello".

**Cronologia.** Nel pacchetto dello Store, i confronti che si completano vengono raggruppati in base al set di dati e al modello del lato B. Ogni misura viene visualizzata su queste recensioni, con i punti in cui il suo livello è cambiato. Una modifica del codice o dell'ambiente accanto a una modifica viene indicata e definita come coincidenza, non come causa. Il registro conserva le ultime 40 recensioni.

**Pacchetti e stato salvato.**
- **Salva pacchetto** scrive la recensione come un file `.scbundle`. **Apri pacchetto** controlla il suo hash, lo rifiuta in caso di mancata corrispondenza e lo visualizza esattamente come salvato, in modalità di recensione. Se le regole di oggi leggono una recensione di geometria salvata in modo diverso, tale lettura viene visualizzata sotto di essa, con l'etichetta appropriata.
- **I pacchetti della versione 2.0** si aprono: pacchetti di confronto con i loro delta salvati e recensioni di inferenza contrassegnate come non verificate.
- **Le impostazioni della versione 2.0** vengono mantenute: tema, palette di colori, contrasto elevato, scala del testo, limite dei file recenti e regola delle anomalie.
- **Esporta** la vista corrente come SVG o la finestra come PNG.

---

## Avvio rapido

### Dal Microsoft Store

1. Installare **ScalarScope** da [Microsoft Store](https://apps.microsoft.com/detail/9P3HT1PHBKQK). Richiede Windows 10 versione 1809 (build 17763) o successiva, x64.
2. Nella schermata **Benvenuto**, fare clic su **Prova il confronto di esempio**. Oppure, fare clic su **Confronta due esecuzioni** e aprire il percorso A e il percorso B.
3. Leggere il titolo, quindi fare clic su una casella per visualizzare **Perché** e **Mostrami**.

I file di esempio da provare, una coppia di inferenza, una coppia di geometria e una cartella di esecuzioni per Workbench, sono disponibili in [`samples/`](samples/). [TESTING.md](TESTING.md) indica quali file aprire e dove.

### Dal codice sorgente

```bash
git clone https://github.com/mcp-tool-shop-org/scalarscope.git
cd scalarscope
cargo run --release --manifest-path rust/Cargo.toml
```

`scalarscope <path A> <path B>` apre una coppia all'avvio.

### Cosa è possibile aprire

| Tipo | File |
|---|---|
| Inferenza | Un file CSV con dati sulla latenza, un file JSON con dati di riferimento, una traccia di un profiler Chrome o PyTorch (`.json` o `.json.gz`), un log di runtime, un file JSON ScalarScope RunTrace o una cartella di esecuzione. |
| Addestramento | Una retropropagazione `run_history.json` |
| Geometria | Un'esportazione di geometria ASPIRE (aspire-si, schema 1.x) |
| Revisione | Un `.scbundle` dalla versione 3.x o 2.0 |

La sezione "Inizia" del [manuale](https://mcp-tool-shop-org.github.io/scalarscope/handbook/getting-started/) contiene un breve esempio per ciascun file.

---

## Scorciatoie da tastiera

| Scorciatoia | Azione |
|---|---|
| `F1` | Guida |
| `Ctrl+,` | Impostazioni |
| `Ctrl+H` | Benvenuto |
| `1`–`6` | Nella sezione "Confronta": Serie, Riscaldamento, Distribuzione, Differenza, Spettro, Mappa di calore |
| `Esc` | Chiudi il pannello "Perché" |

---

## Test

```bash
# The review: 235 tests. Line coverage must stay above 90%.
cd rust
cargo test
cargo llvm-cov --locked --all-targets --fail-under-lines 90

# The .NET fixture tests: the 2.0 oracles, bundle compatibility and the version surfaces (135 tests)
dotnet test tests/ScalarScope.FixtureTests
```

Il progetto .NET in `src/ScalarScope` è l'applicazione 2.0. Rimane nel repository come riferimento rispetto al quale viene verificata la versione 3.x. Il pacchetto Store viene creato da `rust/` tramite `packaging/pack.ps1` e il progetto MAUI si rifiuta di essere pubblicato con tale caricamento.

---

## Struttura del progetto

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

## Correlati

- [Manuale](https://mcp-tool-shop-org.github.io/scalarscope/handbook/): la guida alla revisione
- [Parità e oltre](docs/parity-and-beyond.spec.md): cosa la versione 3.x mantiene dalla 2.0, cosa cambia e perché, con i relativi riferimenti
- [CHANGELOG.md](CHANGELOG.md): cronologia delle versioni
- [PRIVACY.md](PRIVACY.md): informativa sulla privacy
- [TESTING.md](TESTING.md): come testare questa versione
- [L'ambiente di lavoro](https://github.com/mcp-tool-shop-org/runforge): condiviso con RunForge

---

## Licenza

[MIT](LICENSE). Copyright (c) 2025-2026 Progetto ScalarScope (mcp-tool-shop-org)

<p align="center">
  Built by <a href="https://mcp-tool-shop.github.io/">MCP Tool Shop</a>
</p>
