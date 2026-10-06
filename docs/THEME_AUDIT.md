# Theme Consistency Audit

This document tracks the UX consistency audit for ScalarScope across light and dark themes.

## Audit Date: 2025-02-04

## Color System

### Dark Theme (Default)
| Element | Color | Hex |
|---------|-------|-----|
| Background (deepest) | Deep Blue | `#0f0f1a` |
| Surface (cards/panels) | Dark Blue | `#1a1a2e` |
| Elevated | Navy | `#16213e` |
| Grid/Dividers | Muted Blue | `#2a2a4e` |
| Primary Accent | Cyan | `#00d9ff` |
| Success | Teal | `#4ecdc4` |
| Danger | Coral | `#ff6b6b` |
| Warning | Orange | `#ff9f43` |
| Info | Purple | `#a29bfe` |
| Text Primary | White | `#ffffff` |
| Text Secondary | White 70% | `#ffffffb3` |
| Text Disabled | White 40% | `#ffffff66` |

### Light Theme
| Element | Color | Hex |
|---------|-------|-----|
| Background | Light Gray | `#f5f5f5` |
| Surface | White | `#ffffff` |
| Elevated | White | `#ffffff` |
| Grid/Dividers | Light Gray | `#dddddd` |
| Primary Accent | Cyan (same) | `#00d9ff` |
| Success | Teal (same) | `#4ecdc4` |
| Danger | Coral (same) | `#ff6b6b` |
| Warning | Orange (same) | `#ff9f43` |
| Info | Purple (same) | `#a29bfe` |
| Text Primary | Dark Gray | `#333333` |
| Text Secondary | Medium Gray | `#666666` |
| Text Disabled | Light Gray | `#999999` |

## Page-by-Page Audit

The shell a person opens is Home, Compare, Guide, and Settings. Older page names are not this shell.

### Home
| Element | Dark Theme | Light Theme | Status |
|---------|------------|-------------|--------|
| Background | `#0f0f1a` | `#f5f5f5` | Pass |
| Cards | `#1a1a2e` | `#ffffff` | Pass |
| Primary button | White text on `#4ecdc4` | White text on `#4ecdc4` | Fail |

### Compare
| Element | Dark Theme | Light Theme | Status |
|---------|------------|-------------|--------|
| Left run accent | `#4ecdc4` | `#4ecdc4` | Pass |
| Right run accent | `#ff6b6b` | `#ff6b6b` | Pass |
| Primary button | White text on `#4ecdc4` | White text on `#4ecdc4` | Fail |

### Guide
| Element | Dark Theme | Light Theme | Status |
|---------|------------|-------------|--------|
| Background | `#0f0f1a` | `#f5f5f5` | Pass |
| Cards | `#1a1a2e` | `#ffffff` | Pass |
| Primary button | White text on `#4ecdc4` | White text on `#4ecdc4` | Fail |

### Settings
| Element | Dark Theme | Light Theme | Status |
|---------|------------|-------------|--------|
| Background | `#0f0f1a` | `#f5f5f5` | Pass |
| Cards | `#1a1a2e` | `#ffffff` | Pass |
| Primary button | White text on `#4ecdc4` | White text on `#4ecdc4` | Fail |

## Component Audit

### Buttons
| State | Dark Theme | Light Theme | Status |
|-------|------------|-------------|--------|
| Primary | White text on `#4ecdc4` | White text on `#4ecdc4` | Fail |
| Secondary | Surface bg, white text | Light bg, dark text | Pass |
| Disabled | Muted bg, disabled text | Muted bg, disabled text | Pass |
| Hover | Lightened | Darkened | N/A (touch) |

### Input Fields
| State | Dark Theme | Light Theme | Status |
|-------|------------|-------------|--------|
| Default | Surface bg, white text | White bg, dark text | Pass |
| Focused | Cyan border | Cyan border | Pass |
| Error | Red border | Red border | Pass |

### Dialogs/Frames
| Element | Dark Theme | Light Theme | Status |
|---------|------------|-------------|--------|
| Background | `#1a1a2e` | `#ffffff` | Pass |
| Border | `#2a2a4e` | `#dddddd` | Pass |
| Shadow | Subtle dark | Subtle light | Pass |

### Focus Rings
| State | Dark Theme | Light Theme | Status |
|-------|------------|-------------|--------|
| Focus visible | ✅ Cyan outline | ✅ Cyan outline | Pass |
| Tab navigation | ✅ Visible | ✅ Visible | Pass |

### Tab Bar
| Element | Dark Theme | Light Theme | Status |
|---------|------------|-------------|--------|
| Background | `#1a1a2e` | `#1a1a2e` | ⚠️ Same |
| Selected | Cyan | Cyan | Pass |
| Unselected | `#666` | `#666` | ⚠️ Same |

## Contrast Ratios

### Dark Theme
| Text Type | Foreground | Background | Ratio | WCAG |
|-----------|------------|------------|-------|------|
| Primary text | `#ffffff` | `#0f0f1a` | 18.1:1 | AAA ✅ |
| Secondary text | `#ffffffb3` | `#0f0f1a` | 12.7:1 | AAA ✅ |
| Muted text | `#666666` | `#0f0f1a` | 3.1:1 | AA ⚠️ |
| Accent on dark | `#00d9ff` | `#0f0f1a` | 8.4:1 | AAA ✅ |

### Light Theme
| Text Type | Foreground | Background | Ratio | WCAG |
|-----------|------------|------------|-------|------|
| Primary text | `#333333` | `#f5f5f5` | 9.7:1 | AAA ✅ |
| Secondary text | `#666666` | `#f5f5f5` | 5.5:1 | AA ✅ |
| Muted text | `#999999` | `#f5f5f5` | 2.8:1 | Fail ⚠️ |
| Accent on light | `#00d9ff` | `#f5f5f5` | 2.3:1 | Fail ⚠️ |

## Known Issues

1. **Tab bar doesn't change in light mode** - Uses dark theme colors consistently. This is intentional for brand identity but could be revisited.

2. **Cyan accent on light background** - `#00d9ff` on `#f5f5f5` is about 2.3:1.

3. **White text on the teal button** - White on `#4ecdc4` is about 1.9:1, under the 4.5:1 text minimum. This audit records that failure. `#4ecdc4` stays Path A. The button color is not darkened here.

4. **Muted text in both themes** - Borderline accessibility. Reserved for truly non-essential hints only.

## Recommendations

### Immediate (Before RC1)
- [x] Ensure all pages use `AppThemeBinding` for backgrounds
- [x] Verify all text uses theme-aware colors
- [x] Check frame/card borders respond to theme

### Future Improvements
- [ ] White on `#4ecdc4` stays a recorded failure. Path A is not darkened by this audit.
- [ ] Add high-contrast mode option
- [ ] Test with Windows High Contrast theme

## Audit Summary

| Category | Items Checked | Passing | Issues |
|----------|---------------|---------|--------|
| Pages | 4 | 0 | 4 |
| Components | 4 | 3 | 1 |
| Contrast (Dark) | 4 | 3 | 1 |
| Contrast (Light) | 4 | 2 | 2 |
| **Total** | **16** | **8** | **8** |

**Overall Status**: Fail. Home, Compare, Guide, and Settings each carry a primary button with white text on `#4ecdc4` (about 1.9:1). That pair does not clear the 4.5:1 text minimum. The teal is Path A and is not darkened by this audit.

---

## Theme Switching Test

1. Launch app in dark mode (default)
2. Change Windows theme to Light
3. Verify all elements update
4. Change back to Dark
5. Verify restoration

**Result**: Fail. The page inventory is Home, Compare, Guide, and Settings. White text on `#4ecdc4` does not clear 4.5:1, so the shell is not ready.
