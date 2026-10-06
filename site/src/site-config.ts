import type { SiteConfig } from '@mcptoolshop/site-theme';

export const config: SiteConfig = {
  title: 'ScalarScope',
  description: 'ASPIRE Scalar Vortex Visualizer — .NET MAUI desktop app for comparing ML inference runs with scientific rigor.',
  logoBadge: 'SS',
  brandName: 'ScalarScope',
  repoUrl: 'https://github.com/mcp-tool-shop-org/scalarscope',
  footerText: 'MIT Licensed — built by <a href="https://github.com/mcp-tool-shop-org" style="color:var(--color-muted);text-decoration:underline">mcp-tool-shop-org</a>',

  hero: {
    badge: '.NET MAUI · Windows',
    headline: 'ML inference runs,',
    headlineAccent: 'compared with rigor.',
    description: 'Stop eyeballing logs. ScalarScope loads two TFRT traces side by side, fires canonical delta analysis only when differences are statistically meaningful, and exports .scbundle archives. A matching SHA-256 is a content check, not a signature.',
    primaryCta: { href: 'https://apps.microsoft.com/detail/9P3HT1PHBKQK', label: 'Get from Microsoft Store' },
    secondaryCta: { href: 'handbook/', label: 'Read the Handbook' },
    previews: [
      {
        label: 'Install',
        code: '# Microsoft Store (ID: 9P3HT1PHBKQK)\n# https://apps.microsoft.com/detail/9P3HT1PHBKQK\n\n# VortexKit ships in this repo at src/VortexKit.\n# It is not a NuGet package.',
      },
      {
        label: 'VortexKit',
        code: 'using VortexKit.Core;\n\n// Shared timeline across all canvases\nvar player = new PlaybackController { Duration = 10.0, Loop = true };\nplayer.TimeChanged += () =>\n{\n    trajectoryCanvas.CurrentTime = player.Time;\n    eigenCanvas.CurrentTime      = player.Time;\n    scalarsCanvas.CurrentTime    = player.Time;\n};',
      },
      {
        label: 'Export bundle',
        code: '// ComparisonBundleService.ExportAsync(bundle, outputPath)\n// writes a .scbundle (manifest, findings, repro).\n// SHA-256 of those bytes is a content check, not a signature.\n// VortexKit ExportService.ExportComparisonAsync writes a PNG,\n// not an .scbundle.',
      },
    ],
  },

  sections: [
    {
      kind: 'features',
      id: 'features',
      title: 'Replace eyeballing with structure',
      subtitle: 'Every comparison is reproducible. Every delta is earned.',
      features: [
        {
          title: 'Apples-to-apples comparison',
          desc: 'Load two TFRT inference traces side by side. The TFRT preset auto-suppresses irrelevant metrics so your analysis stays focused on what actually changed between runs.',
        },
        {
          title: 'Canonical delta analysis',
          desc: 'On an inference comparison this page shows ΔTc (steps to stable latency), ΔO (runtime instability), and ΔF (failure rate). ΔĀ (average latency) and ΔTd (total duration) stay off this page.',
        },
        {
          title: 'Reproducible .scbundle exports',
          desc: 'Export .scbundle archives with a SHA-256 content check, frozen deltas, and provenance metadata. A matching hash is not a signature. Open in Review mode and the stored result is shown, not re-derived.',
        },
      ],
    },
    {
      kind: 'data-table',
      id: 'deltas',
      title: 'Inference deltas',
      subtitle: 'ΔĀ (average latency) and ΔTd (total duration) stay off this page.',
      columns: ['Delta', 'Measures'],
      rows: [
        ['ΔTc', 'Steps to stable latency'],
        ['ΔO', 'Runtime instability'],
        ['ΔF', 'Failure rate'],
      ],
    },
    {
      kind: 'code-cards',
      id: 'quickstart',
      title: 'Quick start',
      cards: [
        {
          title: 'Install from Microsoft Store',
          code: '# Store ID: 9P3HT1PHBKQK\n# https://apps.microsoft.com/detail/9P3HT1PHBKQK\n\n1. Click "Compare Two Runs"\n2. Load baseline TFRT trace (before)\n3. Load optimized TFRT trace (after)\n4. Review deltas in the Compare tab\n5. Export .scbundle for reproducible sharing',
        },
        {
          title: 'VortexKit in this repo',
          code: '# src/VortexKit — not published to NuGet\n\n// Reusable visualization framework:\n// - Time-synced playback across canvases\n// - Animated SkiaSharp rendering\n// - Comparison views + annotation overlays\n// - SVG/PNG export\n// - Semantic color system',
        },
        {
          title: 'Custom canvas with VortexKit',
          code: 'public class MyTrajectoryCanvas : AnimatedCanvas\n{\n    protected override void OnRender(\n        SKCanvas canvas,\n        SKImageInfo info,\n        double time)\n    {\n        // SkiaSharp rendering at current timeline position\n        // Automatically synchronized with other canvases\n        // via shared PlaybackController\n    }\n}',
        },
        {
          title: 'Build from source',
          code: 'git clone https://github.com/mcp-tool-shop-org/scalarscope\ncd scalarscope\n\n# Requires .NET 9 + MAUI workload\ndotnet workload install maui-windows\ndotnet build ScalarScope.sln\n\n# Run tests\ndotnet test',
        },
      ],
    },
    {
      kind: 'features',
      id: 'design',
      title: 'Designed for serious ML workflows',
      subtitle: 'Scientific rigor without the spreadsheet.',
      features: [
        {
          title: 'Review mode',
          desc: 'Open any .scbundle and the stored review is shown, not re-derived. A matching SHA-256 checks the archived bytes. It is a content check, not a signature.',
        },
        {
          title: 'Privacy first',
          desc: 'Zero telemetry, zero analytics, zero network calls. All comparison data stays on your machine unless you explicitly export a bundle. No accounts, no subscriptions.',
        },
        {
          title: 'VortexKit for your own tools',
          desc: 'The visualization framework powering ScalarScope lives in src/VortexKit in this repo. It is not on NuGet. It builds time-synced animated canvases, comparison views, and export pipelines.',
        },
      ],
    },
  ],
};
