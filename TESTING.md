# Testing ScalarScope 3.1.1

Each check below takes under a minute, using the files in [`samples/`](samples/). Nothing here needs a network connection, an account or a GPU.

## 1. The built-in sample (no files needed)

1. Start ScalarScope. It opens on **Welcome**.
2. Click **Try the sample comparison**.
3. **Compare** shows the headline "B/A p50 0.65 (0.65–0.66) · p90 0.65 (0.64–0.66)", three quiet tiles (ΔF, ΔTc, ΔO) and a series chart.
4. Click a tile to open **Why**. Press `1` to `6` to switch views.

## 2. An inference pair

1. On **Compare**, click **Open path A** and pick `samples/inference/baseline.runtrace.json`.
2. Click **Open path B** and pick `samples/inference/optimized.runtrace.json`.
3. The page names both runs. These files are short (20 steps), so the headline says a ratio needs 20 steady samples per run. ΔTc fires: the optimized run stabilizes 6 steps earlier, at the steps the files state.
4. **Save bundle** writes a `.scbundle` to a path you pick. **Open bundle** on that file shows the same review in review mode, with loading turned off until **Close review**.

## 3. A geometry pair

1. Open path A: `samples/geometry/local-teacher.drift.geometry.json`.
2. Open path B: `samples/geometry/composite-teacher.drift.geometry.json`.
3. The page shows a trajectory plot, a panel per evaluator score and five tiles. ΔF, ΔTc and ΔO say **withheld**, with the reason "steps are checkpoint × item, not time". ΔĀ fires.
4. For the same runs per training step, open `local-teacher.geometry.json` and `composite-teacher.geometry.json`. Here ΔF is withheld because the scores were replayed.

## 4. The Workbench

1. Open the **Workbench** tab and click **Open runs…**.
2. In `samples/workbench`, select all nine files (`Ctrl+A` in the file dialog) and open them.
3. The page lists 9 runs, says that batch size varies (from RunTrace metadata) and that precision is shared.
4. In **Try a formula**, type `p50` and click **Evaluate**. Each run gets a value.
5. **Ask** needs a local Ollama with a model that can call tools. Without one, the page says the local model is not running. That is the expected result on a machine without Ollama.

## 5. History and Settings

- **History** draws the reviews logged by this installation. On a fresh install it says no inference reviews are logged yet. Each comparison you finish in step 2 adds one.
- **Settings** has the theme (Follow Windows, Light, Dark), color-vision palettes, high contrast, text scale and the anomaly rule. A change applies at once and is kept for the next start.

## Keyboard

`F1` opens Guide, `Ctrl+,` opens Settings and `Ctrl+H` opens Welcome. On Compare, `1` to `6` choose the view, and `Esc` closes the Why panel.
