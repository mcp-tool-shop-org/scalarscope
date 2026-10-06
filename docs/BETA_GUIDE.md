# ScalarScope Beta Guide

Welcome to the ScalarScope beta program! This guide explains what to test, how to report issues, and what to include in your feedback.

## What is ScalarScope?

ScalarScope is a visualization tool for machine learning training dynamics. It lets you:
- Visualize 2D training trajectories in real-time
- Compare runs side-by-side with synchronized playback
- Analyze eigenvalue spectra and geometry
- Export publication-quality screenshots
- Identify failures and anomalies

## Beta Goals

During this beta period, we're looking for feedback on:
1. **Stability** - Does the app crash or freeze?
2. **Performance** - Is playback smooth? Is export fast?
3. **Usability** - Is the UI intuitive? Are features discoverable?
4. **Correctness** - Do visualizations match expected behavior?
5. **Missing Features** - What would make this tool more useful?

## What to Test

### Basic Workflow
The shell has four tabs: Home, Compare, Guide, and Settings. Trajectory, Scalars, and Geometry are not tabs.
1. Open Home
2. Open Compare and load two runs
3. Open Guide and read a delta
4. Open Settings and check About
5. Export from Compare when that control is on the page

### Comparison Workflow
1. Open the Compare tab
2. Load two different runs
3. Read the deltas that fired
4. Open the Why panel on one row

### Stress Testing
1. Load large files (1000+ timesteps)
2. Run for extended periods (30+ minutes)
3. Export multiple screenshots rapidly
4. Switch themes multiple times

### Edge Cases
1. Load malformed or incomplete JSON
2. Resize window to extreme dimensions
3. Use with screen readers or high-contrast modes

## How to Report Issues

### Before Reporting
1. Check [existing issues](https://github.com/mcp-tool-shop-org/scalarscope/issues)
2. Create a support bundle from the button on the Guide page. Recovery has the same button when the app opens there. There is no Help menu.

### What to Include
- **Steps to reproduce**: Exactly what you did
- **Expected behavior**: What should have happened
- **Actual behavior**: What actually happened
- **Support bundle**: Attach the generated file
- **Screenshots/recordings**: If visual issue

### Issue Template
```markdown
## Bug Description
[Brief description]

## Steps to Reproduce
1. ...
2. ...
3. ...

## Expected Behavior
[What should happen]

## Actual Behavior
[What actually happens]

## Environment
- OS: [e.g., Windows 11 23H2]
- Version: [e.g., 3.0.0]
- GPU: [e.g., NVIDIA RTX 5080]

## Support Bundle
[Attach file]

## Screenshots
[If applicable]
```

## What Logs to Include

When reporting issues, include:
1. **Support bundle** (the button on the Guide page) - contains:
   - System information
   - Memory usage
   - Recent crash logs
   - App configuration
2. **Screenshots or screen recordings** of the issue
3. **The training run file** (if reproducible with specific data)

## Known Limitations

For the current release (display version 3.0.0):
- **Windows only** - macOS/Linux not yet tested
- **Unsigned upload** - Do not double-click the MSIX. Windows will not install that file. Wait for the Partner Center signed install. Developer Mode is not the fix.
- **No auto-update** - Manual download for new versions
- **No cloud features** - All data is local

## Feedback Channels

- **Bug reports**: [GitHub Issues](https://github.com/mcp-tool-shop-org/scalarscope/issues)
- **Feature requests**: [GitHub Issues](https://github.com/mcp-tool-shop-org/scalarscope/issues) with `[FEATURE]` prefix
- **Questions**: [GitHub Discussions](https://github.com/mcp-tool-shop-org/scalarscope/discussions)
- **Security issues**: See SECURITY.md

## Timeline

| Phase | Dates | Focus |
|-------|-------|-------|
| RC1 Beta | Feb 2025 | Stability, core features |
| RC2 | TBD | Bug fixes, polish |
| Store | current | Display version 3.0.0 is on the Store (9P3HT1PHBKQK). Do not report 1.0.0-rc.1. |

## Thank You!

Your feedback directly shapes the product. Every bug report, feature request, and usability observation helps make ScalarScope better for the research community.

We're particularly interested in hearing from:
- ML researchers analyzing training dynamics
- Educators teaching optimization
- Anyone working with learned critics

Happy testing! 🚀
