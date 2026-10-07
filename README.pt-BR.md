<p align="center">
  <a href="README.ja.md">日本語</a> | <a href="README.zh.md">中文</a> | <a href="README.es.md">Español</a> | <a href="README.fr.md">Français</a> | <a href="README.hi.md">हिन्दी</a> | <a href="README.it.md">Italiano</a> | <a href="README.md">English</a>
</p>

<p align="center">
  <img src="https://raw.githubusercontent.com/mcp-tool-shop-org/brand/main/logos/ScalarScope/readme.png" width="400" alt="ScalarScope">
</p>

> Parte do [MCP Tool Shop](https://mcptoolshop.com)

<p align="center">
  <a href="https://github.com/mcp-tool-shop-org/scalarscope/actions/workflows/build.yml"><img src="https://github.com/mcp-tool-shop-org/scalarscope/actions/workflows/build.yml/badge.svg" alt="Build and Test"></a>
  <a href="https://codecov.io/gh/mcp-tool-shop-org/scalarscope"><img src="https://codecov.io/gh/mcp-tool-shop-org/scalarscope/graph/badge.svg" alt="Coverage"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/License-MIT-yellow.svg?style=flat-square" alt="License: MIT"></a>
  <a href="https://mcp-tool-shop-org.github.io/scalarscope/"><img src="https://img.shields.io/badge/Landing-page-blue?style=flat-square" alt="Landing page"></a>
  <a href="https://apps.microsoft.com/detail/9P3HT1PHBKQK"><img src="https://img.shields.io/badge/Microsoft%20Store-9P3HT1PHBKQK-0078D4?style=flat-square&logo=microsoft" alt="Microsoft Store"></a>
</p>

**Compare duas execuções de aprendizado de máquina e veja o quão confiável é a comparação.** O ScalarScope abre dois rastreamentos de inferência, dois históricos de treinamento ou duas exportações de geometria ASPIRE. Ele indica como a execução B difere da execução A, atribui a cada diferença um intervalo ou sua razão e retém o que os dados não podem comprovar.

A versão **3.1.0** é uma reescrita em Rust. É o aplicativo para Windows na Microsoft Store como [9P3HT1PHBKQK](https://apps.microsoft.com/detail/9P3HT1PHBKQK), como pacote 3.1.0.0, uma atualização da mesma listagem. O nome do pacote `mcp-tool-shop.ScalarScope` e o editor `CN=5305D976-6952-4F00-9C21-3A5DB090359F` permanecem inalterados, portanto, os arquivos, avaliações e configurações salvos da versão 2.0 são mantidos.

## Modelo de confiança

- **O que ele lê.** Os arquivos que você abre. O pacote da Store também lê e grava `comparison-log.json`, `preferences.json` e `workbench.json` em sua própria pasta LocalState, a pasta que a versão 2.0 usava. Uma versão não empacotada não grava nenhum deles.
- **O que ele grava.** Um conjunto de dados, uma imagem ou um registro de sessão, apenas em um caminho que você escolher.
- **Rede.** Não há conta, telemetria ou análise. A única conexão que o aplicativo pode fazer é a partir da guia Workbench, e apenas quando você pressiona **Perguntar**: para um Ollama local em `127.0.0.1` neste computador. Os modelos em nuvem do Ollama são recusados. Nada sai da máquina.
- **Pacotes (bundles).** Um SHA-256 correspondente é uma verificação de conteúdo, não uma assinatura. Indica que os bytes estão intactos. Não indica quem escreveu o arquivo.
- **Plugins.** Os plugins deixados na pasta da versão 2.0 não são carregados.

---

## O que ele faz

**Inferência: duas execuções de latência por etapa.**
- **O título** é B/A em p50, p90 e p99. Cada razão possui um intervalo de bootstrap móvel de 95%.
- **Várias execuções por lado** podem ser abertas. O intervalo, então, também amostra execuções inteiras e é chamado de indicativo abaixo de três execuções por lado.
- **Um percentil sem amostras suficientes** não é impresso; p99 precisa de 368.
- **Três deltas,** cada um com um bloco que indica se foi ativado, permaneceu inativo ou foi retido, e por quê:
- **ΔF (novas anomalias)** conta amostras estáveis além de 5 desvios robustos. É ativado apenas quando o excesso de B está além do esperado.
- **ΔO (variabilidade)** é ativado quando o intervalo da dispersão relativa de B em relação a A exclui 1.
- **ΔTc (estabilização)** atribui a cada execução um formato. É ativado apenas quando ambas se estabilizam e seus intervalos de estabilização não se sobrepõem.
- **Seis visualizações:** Série, Aquecimento, Distribuição com um limite que você arrasta, Diferença por percentil, Espectro e Mapa de calor. Nenhuma é animada.

**Treinamento.** Uma retropropagação `run_history.json` é desenhada como perda de treinamento, com perda retida, perplexidade e métricas de tarefa ao lado. Os deltas de inferência não são calculados sobre ela.

**Geometria: duas execuções de treinamento ASPIRE.**
- **O que ele lê:** a exportação de geometria que [aspire-si](https://github.com/mcp-tool-shop-org/aspire-si) grava, com sua trajetória, pontuações do avaliador, espectro próprio e falhas.
- **Cinco deltas:** ΔF, ΔTc, ΔTd, ΔĀ (concentração do espectro) e ΔO. Eles são importados da versão 2.0 e verificados em relação aos resultados da própria versão 2.0, com as correções registradas em [a especificação](docs/parity-and-beyond.spec.md).
- **Contrato de exportação (esquema 1.1).** Uma exportação pode indicar que suas etapas são pontos de verificação × itens em vez de tempo, ou que suas pontuações foram reproduzidas. Quando o faz, os deltas que leem o tempo ou as quedas de pontuação são retidos com essa razão, e o título fala apenas dos deltas que permanecem. Exportações mais antigas são lidas como antes.

**Workbench.**
- **Abra várias execuções** que diferem em uma configuração, como tamanho do lote ou precisão.
- **Perguntar:** um modelo local que pode chamar ferramentas mede as execuções com fórmulas e propõe o que cada configuração faz.
- **O programa define cada número e veredicto.** Uma hipótese é testada por um teste de classificação exato em cada conjunto de execuções. Os veredictos entre os conjuntos vêm apenas em pontos de verificação, por e-BH em uma taxa de descoberta falsa de 5%.
- **Sua decisão primeiro.** Você pode escrever sua própria pergunta antes de perguntar. A nota do modelo é mostrada abaixo dos veredictos, rotulada como suas palavras.

**Histórico.** No pacote da Store, as comparações que você conclui são agrupadas pelo conjunto de dados e modelo do lado B. Cada medida é desenhada em relação a essas avaliações, com os pontos onde seu nível mudou. Uma mudança de código ou ambiente ao lado de uma mudança é nomeada e chamada de coincidência, não de causa. O log mantém as últimas 40 avaliações.

**Conjuntos de dados e estado salvo.**
- **Salvar conjunto de dados** grava a avaliação como um `.scbundle`. **Abrir conjunto de dados** mostra-o exatamente como foi salvo, no modo de avaliação.
- **Os conjuntos de dados da versão 2.0** são abertos: conjuntos de dados de comparação com seus deltas armazenados e avaliações de inferência marcadas como não verificadas.
- **As configurações da versão 2.0** são mantidas: tema, paletas de visão de cores, alto contraste, escala de texto, o limite de arquivos recentes e a regra de anomalia.
- **Exportar** a visualização atual como SVG ou a janela como PNG.

---

## Guia rápido

### Da Microsoft Store

1. Instale o **ScalarScope** na [Microsoft Store](https://apps.microsoft.com/detail/9P3HT1PHBKQK). Ele precisa do Windows 10 versão 1809 (compilação 17763) ou posterior, x64.
2. Em **Bem-vindo**, clique em **Experimente a comparação de exemplo**. Ou clique em **Comparar duas execuções** e abra o caminho A e o caminho B.
3. Leia o título e, em seguida, clique em um bloco para **Por quê** e **Mostre-me**.

Os arquivos de exemplo para testar, um par de inferência, um par de geometria e uma pasta de execuções para o Workbench, estão em [`samples/`](samples/). [TESTING.md](TESTING.md) indica quais arquivos abrir e onde.

### Do código-fonte

```bash
git clone https://github.com/mcp-tool-shop-org/scalarscope.git
cd scalarscope
cargo run --release --manifest-path rust/Cargo.toml
```

`scalarscope <path A> <path B>` abre um par ao iniciar.

### O que você pode abrir

| Tipo | Arquivos |
|---|---|
| Inferência | Um CSV de latência, um JSON de benchmark, um rastreamento do Chrome ou PyTorch (`.json` ou `.json.gz`), um log de tempo de execução, um JSON ScalarScope RunTrace ou uma pasta de execução |
| Treinamento | Uma retropropagação `run_history.json` |
| Geometria | Uma exportação de geometria ASPIRE (aspire-si, esquema 1.x) |
| Revisão | Um `.scbundle` da versão 3.x ou 2.0 |

O guia “Primeiros Passos” em [handbook](https://mcp-tool-shop-org.github.io/scalarscope/handbook/getting-started/) contém um exemplo breve de cada arquivo.

---

## Atalhos de teclado

| Atalho | Ação |
|---|---|
| `F1` | Guia |
| `Ctrl+,` | Configurações |
| `Ctrl+H` | Bem-vindo |
| `1`–`6` | Em Comparar: Série, Aquecimento, Distribuição, Diferença, Espectro, Mapa de calor |
| `Esc` | Fechar o painel “Por quê” |

---

## Testes

```bash
# The review: 223 tests. Line coverage must stay above 90%.
cd rust
cargo test
cargo llvm-cov --locked --all-targets --fail-under-lines 90

# The .NET fixture tests: the 2.0 oracles, bundle compatibility and the version surfaces (134 tests)
dotnet test tests/ScalarScope.FixtureTests
```

O projeto .NET em `src/ScalarScope` é o aplicativo 2.0. Ele permanece no repositório como a referência com a qual a versão 3.x é comparada. O pacote Store é criado a partir de `rust/` por `packaging/pack.ps1`, e o projeto MAUI se recusa a ser publicado dessa forma.

---

## Estrutura do projeto

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

## Relacionado

- [Guia](https://mcp-tool-shop-org.github.io/scalarscope/handbook/): o guia para a revisão
- [Paridade e além](docs/parity-and-beyond.spec.md): o que a versão 3.x mantém da 2.0, o que ela altera e por quê, com as fontes
- [CHANGELOG.md](CHANGELOG.md): histórico de lançamentos
- [PRIVACY.md](PRIVACY.md): política de privacidade
- [TESTING.md](TESTING.md): como testar esta versão
- [O ambiente de trabalho](https://github.com/mcp-tool-shop-org/runforge): compartilhado com o RunForge

---

## Licença

[MIT](LICENSE). Copyright (c) 2025-2026 Projeto ScalarScope (mcp-tool-shop-org)

<p align="center">
  Built by <a href="https://mcp-tool-shop.github.io/">MCP Tool Shop</a>
</p>
