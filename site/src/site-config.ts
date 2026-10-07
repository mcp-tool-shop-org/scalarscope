import type { SiteConfig } from '@mcptoolshop/site-theme';

export const config: SiteConfig = {
  title: 'ScalarScope',
  description: 'Compare two machine-learning runs on Windows, and see how sure the comparison is: inference traces, training histories and ASPIRE geometry, with a local workbench. Nothing leaves the machine.',
  logoBadge: 'SS',
  brandName: 'ScalarScope',
  repoUrl: 'https://github.com/mcp-tool-shop-org/scalarscope',
  footerText: 'MIT Licensed — built by <a href="https://github.com/mcp-tool-shop-org" style="color:var(--color-muted);text-decoration:underline">mcp-tool-shop-org</a>',

  hero: {
    badge: 'Version 3.1.1 · Windows',
    headline: 'Two ML runs,',
    headlineAccent: 'compared with intervals.',
    description: 'ScalarScope says how run B differs from run A, gives each difference an interval or its reason, and holds back what the data cannot support. Inference traces, training histories and ASPIRE geometry exports. It works offline. A matching SHA-256 is a content check, not a signature.',
    primaryCta: { href: 'https://apps.microsoft.com/detail/9P3HT1PHBKQK', label: 'Get from Microsoft Store' },
    secondaryCta: { href: 'handbook/', label: 'Read the Handbook' },
    previews: [
      {
        label: 'Install',
        code: '# Microsoft Store (ID: 9P3HT1PHBKQK)\n# https://apps.microsoft.com/detail/9P3HT1PHBKQK\n# Windows 10 1809 or later, x64\n\n# Or from source\ncargo run --manifest-path rust/Cargo.toml',
      },
      {
        label: 'Headline',
        code: '# The built-in sample comparison\nB/A p50 0.65 (0.65–0.66) · p90 0.65 (0.64–0.66)\nwithin one run per side; indicative.\nNot shown: p99 needs 368 steady samples per side.\n\nΔF quiet · ΔTc quiet · ΔO quiet',
      },
      {
        label: 'Open a pair',
        code: '# From the command line\nscalarscope samples/inference/baseline.runtrace.json \\\n            samples/inference/optimized.runtrace.json',
      },
    ],
  },

  sections: [
    {
      kind: 'features',
      id: 'features',
      title: 'A comparison that says how sure it is',
      subtitle: 'Every number is computed by the program. Every withheld reading says why.',
      features: [
        {
          title: 'Ratios with intervals',
          desc: 'B/A at p50, p90 and p99, each with a 95% block-bootstrap interval. Several runs per side resample whole runs too. A percentile without enough samples is not printed.',
        },
        {
          title: 'Deltas that earn their place',
          desc: 'ΔF (new anomalies), ΔTc (stabilization) and ΔO (variability) each fire on a stated rule. A tile says whether it fired, stayed quiet or was withheld, and Show me moves to the evidence.',
        },
        {
          title: 'Reviews you can reopen',
          desc: 'Save a .scbundle and reopen it exactly as saved. Bundles, settings and recent files from 2.0 carry over. A matching SHA-256 checks the bytes; it does not say who wrote them.',
        },
      ],
    },
    {
      kind: 'data-table',
      id: 'inputs',
      title: 'What it opens',
      subtitle: 'Getting Started in the handbook has a short example of each.',
      columns: ['Kind', 'Files'],
      rows: [
        ['Inference', 'Latency CSV, benchmark JSON, Chrome or PyTorch profiler trace, runtime log, RunTrace JSON, run folder'],
        ['Training', 'backpropagate run_history.json'],
        ['Geometry', 'ASPIRE geometry export from aspire-si (schema 1.x)'],
        ['Review', '.scbundle from 3.x or 2.0'],
      ],
    },
    {
      kind: 'features',
      id: 'beyond',
      title: 'Geometry, a workbench and a history',
      subtitle: 'New in 3.1.0.',
      features: [
        {
          title: 'ASPIRE geometry',
          desc: 'Two training runs side by side: trajectory, evaluator scores, eigen spectrum and failures, with five geometry deltas. When an export says its steps are not time, or its scores were replayed, the readings that depend on them are withheld with that reason.',
        },
        {
          title: 'A local workbench',
          desc: 'Open runs that differ in a setting and ask a local model, through Ollama on this computer, to measure them and propose what the setting does. The program tests every hypothesis and sets every verdict; the model\'s note is labelled as its words.',
        },
        {
          title: 'History of reviews',
          desc: 'In the Store package, finished comparisons are grouped by the runs\' dataset and model. Each measure is drawn across them, with the points where it shifted and the code or environment changes beside them.',
        },
      ],
    },
    {
      kind: 'code-cards',
      id: 'quickstart',
      title: 'Quick start',
      cards: [
        {
          title: 'Install from the Microsoft Store',
          code: '# Store ID: 9P3HT1PHBKQK\n# https://apps.microsoft.com/detail/9P3HT1PHBKQK\n\n1. Welcome → Try the sample comparison\n2. Or Compare → Open path A, Open path B\n3. Click a tile for Why and Show me\n4. Save bundle to keep the review',
        },
        {
          title: 'Build from source',
          code: 'git clone https://github.com/mcp-tool-shop-org/scalarscope\ncd scalarscope\ncargo run --manifest-path rust/Cargo.toml\n\n# Sample files: samples/ (see TESTING.md)',
        },
        {
          title: 'Keyboard',
          code: 'F1       Guide\nCtrl+,   Settings\nCtrl+H   Welcome\n1–6      Series, Warmup, Distribution,\n         Difference, Spectrum, Heat map\nEsc      Close the Why panel',
        },
      ],
    },
    {
      kind: 'features',
      id: 'privacy',
      title: 'Local by design',
      subtitle: 'No account, no telemetry, no analytics.',
      features: [
        {
          title: 'Nothing leaves the machine',
          desc: 'The only connection the app can make is the Workbench\'s Ask, to a local Ollama at 127.0.0.1. Cloud models are refused.',
        },
        {
          title: 'Your files stay yours',
          desc: 'ScalarScope reads the files you open. The Store package keeps its comparison log, settings and workbench memory in its own app folder, and writes a bundle or picture only where you choose.',
        },
        {
          title: 'Checked against 2.0',
          desc: 'The 2.0 results are kept as test oracles. Where 3.x answers differently, the changelog and the spec say what changed and why.',
        },
      ],
    },
  ],
};
