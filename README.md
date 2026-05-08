# egui-tiles-overlap

Small Bevy + egui test app for **overlapping “tiles”** and **click handling**.

The core question this app helps answer is:

- When multiple UI elements overlap, how do you ensure **only the topmost element** processes a click (and elements “underneath” do not)?

This repo intentionally contains both:

- a **naïve overlap click mode** (demonstrates the “bug”: all overlapping tiles react)
- a **topmost-only click mode** (demonstrates the “fix”: only the front-most tile reacts)

## Run

```bash
cargo run
```

## What you’ll see

### Overlay tiles A/B/C (overlapping)

In the main window, there are three **overlapping rounded rectangles** drawn as an overlay:

- **A** (bottom)
- **B** (middle)
- **C** (top)

Each displays a number.

### Dockable panes D/E/F (movable panes)

The UI also contains a dock layout (`egui_tiles`) with panes:

- **Scene** (contains the overlay drawing area)
- **D**, **E**, **F** (each is a movable/dockable pane with its own counter and a big rounded rectangle)
- **Debug**

You can drag the **D/E/F tabs** around, reorder them, and split them like normal `egui_tiles` panes.

## Controls / behavior

### Mouse

- **Left-click** on a tile: **increment**
- **Right-click** on a tile: **decrement**

For the overlay tiles A/B/C, behavior depends on click mode (see below).

### Keyboard

- **Space**: toggle click mode for the **overlay tiles A/B/C**
  - `topmost-only`: only the topmost tile under the cursor changes (e.g. clicking the triple-overlap changes **C** only)
  - `overlap-all`: every tile under the cursor changes (clicking the triple-overlap changes **A**, **B**, and **C**)

### UI

- **reset** button: resets overlay tile counts (A/B/C) back to 0
- The **top bar** shows the current counts for **A/B/C** and **D/E/F**, plus the current click mode.

## Why the app is structured this way

- **Overlay A/B/C** are drawn as separate floating `egui::Area`s and use manual hit-testing.
  - This makes it easy to demonstrate the difference between “overlap-all” and “topmost-only” by changing only the hit-test logic.
- **D/E/F** are *real* `egui_tiles` panes.
  - They exist to demonstrate that you can have independent, movable panes that also handle clicks and update state.

## Notes

- The overlay click-mode toggle affects **A/B/C only**.
- D/E/F panes each handle clicks locally via normal egui widget responses (left/right click changes that pane’s counter).

