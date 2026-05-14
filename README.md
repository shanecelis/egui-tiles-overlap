# PreUpdate Owner Lag Repro

Small Bevy + egui repro for:

- `#627` / `#637`: UI events pass through to panels behind or not visible.
- Elodin-specific cause: scene/camera input runs in `PreUpdate`, while the UI owner is resolved later by egui.

## What This Shows

The scene contains three overlapping rectangles:

- `A` is behind `B`
- `B` is behind `C`
- `C` is the topmost scene rectangle

A real UI pane named `D` is drawn on top of `C`.

When the pointer is over `D`, scene input should be blocked. If `C` changes while the pointer is over `D`, the event has passed through the UI pane to the scene behind it.

## Run

```bash
cargo run
```

## Controls

- Left click or scroll up: increment the target.
- Right click or scroll down: decrement the target.
- Hold left mouse on `C`, then drag into `D`: deterministic bug repro.
- `L` or `Space`: toggle repro mode.
- `R`: reset counters.

## Modes

### BUG: stale owner in PreUpdate

The scene input system runs in `PreUpdate`.

It gates input with the owner resolved during the previous egui pass. This can be one frame stale.

Expected visible bug:

```text
pointer over: D
PreUpdate used: scene
BUG: C changed while pointer was over D
```

### FIX: current pointer hit-test

The scene input system still runs in `PreUpdate`, but it checks the current pointer position against the latest known UI regions.

Expected behavior:

```text
pointer over: D
PreUpdate used: D
OK: D blocked the scene input
```

## Precise Test

1. Start in `BUG: stale owner in PreUpdate`.
2. Press and hold left mouse on the visible left side of `C`.
3. `C` may increment once here. That is normal because the press started on the scene.
4. While still holding, drag into pane `D`.
5. Watch the counters and banner.

If the bug is reproduced, `C` increments again even though the pointer is over `D`.

Then:

1. Press `L` or `Space` to switch to `FIX: current pointer hit-test`.
2. Repeat the same hold-and-drag movement.
3. `D` should block the scene input, and `C` should not increment when the pointer enters `D`.

## How The Evidence Works

The important contradiction is:

```text
pointer over: D
PreUpdate used: scene
C changed
```

That means the UI owner for the current pointer is `D`, but the scene/camera input path accepted the event as if the pointer were still over the scene.

This isolates the scheduling issue: the scene/camera input runs before egui has published the current frame's owner, so it can make one decision using stale owner data.
