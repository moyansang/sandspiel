# Sheep Material Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a selectable multi-pixel sheep that seeks and eats plants, grows to ten pixels, dies after an 800-step mature lifespan, becomes a solid carcass, and is decomposed by fungus into dust.

**Architecture:** Keep collisions and rendering in the existing cellular grid. Represent each sheep with `Species::Sheep` cells keyed by an `ra` individual ID, and keep creature-level state in a small `Universe::sheep` table; `Species::Carcass` remains an ordinary grid material. Rebuild creature state from the grid after undo and shared-world loading so the existing pixel serialization remains compatible.

**Tech Stack:** Rust, `wasm-bindgen`, WebAssembly, React 16, WebGL/GLSL, webpack, `cargo test`, Browser plugin.

## Global Constraints

- A pointer press creates exactly one sheep, independent of brush size.
- A newborn sheep starts with one core and approximately four body pixels; one sheep never exceeds ten pixels.
- Sheep movement decisions occur every 8 to 12 simulation steps and plant consumption every 20 to 30 steps.
- Reaching ten pixels starts an 800-step mature lifespan; cumulative eating does not directly kill the sheep.
- Starvation may kill after 500 steps without energy; fire, lava, acid, and prolonged water exposure are lethal.
- Carcasses are not directly selectable, remain solid without fungus, and decompose under fungus into approximately 85% dust and 15% fungus over 300 to 600 steps.
- Existing reset, undo, pause, single-step, snapshot/share loading, and the other materials must continue working.
- Do not add a second physics engine or a new runtime dependency.

---

### Task 1: Register Sheep and Carcass Materials

**Files:**
- Modify: `crate/src/species.rs`
- Modify: `crate/src/lib.rs`
- Modify: `js/glsl/sand.glsl`
- Modify: `js/components/ui.js`
- Test: `crate/src/species.rs`

**Interfaces:**
- Produces: `Species::Sheep = 20`, `Species::Carcass = 21`, `update_sheep(cell, api)`, and `update_carcass(cell, api)`.
- Produces: toolbar label `Sheep: "羊"`; `Carcass` is filtered from selectable material buttons.

- [ ] **Step 1: Add a failing enum stability test**

Append this test module to `crate/src/species.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::Species;

    #[test]
    fn sheep_and_carcass_have_stable_serialized_ids() {
        assert_eq!(Species::Sheep as u8, 20);
        assert_eq!(Species::Carcass as u8, 21);
    }
}
```

- [ ] **Step 2: Run the test and verify RED**

Run: `cd crate && cargo test sheep_and_carcass_have_stable_serialized_ids`

Expected: compilation fails because `Species::Sheep` and `Species::Carcass` do not exist.

- [ ] **Step 3: Add the material variants and update dispatch**

Add the enum values and match arms in `crate/src/species.rs`:

```rust
Sheep = 20,
Carcass = 21,
```

```rust
Species::Sheep => update_sheep(cell, api),
Species::Carcass => update_carcass(cell, api),
```

Add temporary behavior that keeps sheep still and gives carcasses stone-like gravity:

```rust
pub fn update_sheep(cell: Cell, mut api: SandApi) {
    api.set(0, 0, cell);
}

pub fn update_carcass(cell: Cell, api: SandApi) {
    update_stone(cell, api);
}
```

Add `Species::Sheep` and `Species::Carcass` to the explicit wind-threshold match in `crate/src/lib.rs`; use threshold `70` for both so wind does not tear creatures apart.

- [ ] **Step 4: Add rendering and toolbar mapping**

Add GLSL branches after type 19:

```glsl
} else if (type == 20) { // sheep
  hue = 0.0;
  saturation = 0.04;
  lightness = 0.92 + data.b * 0.06;
} else if (type == 21) { // carcass
  hue = 0.08;
  saturation = 0.08;
  lightness = 0.58 + data.g * 0.18;
}
```

Add `Sheep: "羊"` and `Carcass: "遗骸"` to `elementNames`. Before mapping buttons, add `.filter((name) => name !== "Carcass")` so only sheep is selectable.

- [ ] **Step 5: Verify GREEN and build**

Run: `cd crate && cargo test sheep_and_carcass_have_stable_serialized_ids`

Expected: `1 passed`.

Run: `pnpm run build`

Expected: webpack exits `0`; existing asset-size warnings are acceptable.

- [ ] **Step 6: Commit**

```powershell
git add crate/src/species.rs crate/src/lib.rs js/glsl/sand.glsl js/components/ui.js
git commit -m "feat: register sheep and carcass materials"
```

### Task 2: Add Sheep State and Grid Reconstruction

**Files:**
- Modify: `crate/src/lib.rs`
- Test: `crate/src/lib.rs`

**Interfaces:**
- Produces: `SheepState { core_x, core_y, energy, size, direction, move_cooldown, eat_cooldown, mature_steps, starvation_steps, submerged_steps }`.
- Produces: `Universe::sheep: HashMap<u8, SheepState>` and `Universe::next_sheep_id: u8`.
- Produces: `Universe::rebuild_sheep_states()` used after undo and external pixel loading.

- [ ] **Step 1: Write failing reconstruction tests**

Add to a `#[cfg(test)] mod tests` at the bottom of `crate/src/lib.rs`:

```rust
#[test]
fn rebuild_groups_sheep_pixels_by_id() {
    let mut universe = Universe::new(20, 20);
    let core_index = universe.get_index(5, 5);
    let body_index = universe.get_index(6, 5);
    universe.cells[core_index] = Cell {
        species: Species::Sheep, ra: 7, rb: SHEEP_CORE, clock: 0,
    };
    universe.cells[body_index] = Cell {
        species: Species::Sheep, ra: 7, rb: SHEEP_BODY, clock: 0,
    };

    universe.rebuild_sheep_states();

    let sheep = universe.sheep.get(&7).unwrap();
    assert_eq!((sheep.core_x, sheep.core_y), (5, 5));
    assert_eq!(sheep.size, 2);
}

#[test]
fn pop_undo_rebuilds_sheep_state_from_restored_cells() {
    let mut universe = Universe::new(20, 20);
    universe.push_undo();
    universe.spawn_sheep(10, 10);
    universe.pop_undo();
    assert!(universe.sheep.is_empty());
}
```

- [ ] **Step 2: Run tests and verify RED**

Run: `cd crate && cargo test rebuild_`

Expected: compilation fails because the constants, state fields, and methods are missing.

- [ ] **Step 3: Add state types and fields**

Import `HashMap` alongside `VecDeque`. Define:

```rust
const SHEEP_BODY: u8 = 0;
const SHEEP_CORE: u8 = 1;

#[derive(Clone, Debug, PartialEq, Eq)]
struct SheepState {
    core_x: i32,
    core_y: i32,
    energy: u16,
    size: u8,
    direction: i8,
    move_cooldown: u8,
    eat_cooldown: u8,
    mature_steps: Option<u16>,
    starvation_steps: u16,
    submerged_steps: u16,
    dying_steps: Option<u8>,
}
```

Add `sheep: HashMap<u8, SheepState>` and `next_sheep_id: u8` to `Universe`, initialized with an empty map and `1`.

- [ ] **Step 4: Implement deterministic reconstruction**

Implement `rebuild_sheep_states()` by scanning the grid, grouping `Species::Sheep` cells by `ra`, choosing the marked core or the first cell if none is marked, changing the selected cell to `SHEEP_CORE`, and inserting one default state per ID. Set `size` from the group length capped at `10`; use energy `100`, direction `1`, move cooldown `8`, eat cooldown `20`, starvation `0`, submerged `0`, and mature steps `Some(800)` only when size is `10`.

Call reconstruction after `pop_undo()`. Clear the table and reset `next_sheep_id` in `reset()`. Reconstructed live sheep use `dying_steps: None`.

- [ ] **Step 5: Run tests and verify GREEN**

Run: `cd crate && cargo test rebuild_`

Expected: both reconstruction tests pass.

- [ ] **Step 6: Commit**

```powershell
git add crate/src/lib.rs
git commit -m "feat: track sheep creature state"
```

### Task 3: Spawn One Compact Sheep per Pointer Press

**Files:**
- Modify: `crate/src/lib.rs`
- Modify: `js/paint.js`
- Test: `crate/src/lib.rs`

**Interfaces:**
- Produces: exported `Universe::spawn_sheep(x: i32, y: i32) -> bool`.
- Consumes: `Species::Sheep`, `SHEEP_CORE`, `SHEEP_BODY`, and `Universe::sheep` from Tasks 1 and 2.

- [ ] **Step 1: Write failing spawn tests**

```rust
#[test]
fn spawn_sheep_creates_one_core_and_up_to_four_body_pixels() {
    let mut universe = Universe::new(20, 20);
    assert!(universe.spawn_sheep(10, 10));
    let cells: Vec<Cell> = universe.cells.iter()
        .copied()
        .filter(|cell| cell.species == Species::Sheep)
        .collect();
    assert_eq!(cells.len(), 5);
    assert_eq!(cells.iter().filter(|cell| cell.rb == SHEEP_CORE).count(), 1);
    assert!(cells.iter().all(|cell| cell.ra == cells[0].ra));
}

#[test]
fn spawn_sheep_fails_without_overwriting_occupied_cells() {
    let mut universe = Universe::new(3, 3);
    for cell in &mut universe.cells {
        cell.species = Species::Wall;
    }
    assert!(!universe.spawn_sheep(1, 1));
    assert!(universe.sheep.is_empty());
}
```

- [ ] **Step 2: Run tests and verify RED**

Run: `cd crate && cargo test spawn_sheep_`

Expected: fails because `spawn_sheep` is not implemented.

- [ ] **Step 3: Implement ID allocation and compact placement**

Expose `spawn_sheep` in the `#[wasm_bindgen] impl Universe`. Search IDs `1..=255` starting at `next_sheep_id` and choose an ID absent from `self.sheep`. Search this fixed shape around the click, in order:

```rust
const NEWBORN_SHAPE: [(i32, i32, u8); 5] = [
    (0, 0, SHEEP_CORE),
    (-1, 0, SHEEP_BODY),
    (1, 0, SHEEP_BODY),
    (0, -1, SHEEP_BODY),
    (1, -1, SHEEP_BODY),
];
```

Place only in-bounds empty cells. Require a free core cell; body cells may be omitted near obstacles. Insert state with `size` equal to placed cells and return `true`.

- [ ] **Step 4: Route sheep pointer starts through the dedicated API**

Rename the existing `paint(event)` function to `paint(event, spawnCreature = false)` without changing its bounding-rectangle or `x`/`y` calculations. Immediately after calculating `x` and `y`, insert:

```js
if (window.UI.state.selectedElement === window.species.Sheep) {
  if (spawnCreature) {
    universe.spawn_sheep(x, y);
  }
  return;
}
```

Change the direct call in `mousedown` from `paint(event)` to `paint(event, true)`. In `touchstart`, snapshot the first touch and call `paint(event.touches[0], true)` instead of routing the start through `handleTouches`; retain `handleTouches` for `touchmove`. Leave `smoothPaint`, the repeat timer, and all ordinary material calls using the default `false`. Preserve the existing `push_undo()` once per pointer press.

- [ ] **Step 5: Verify GREEN**

Run: `cd crate && cargo test spawn_sheep_`

Expected: both tests pass.

Run: `pnpm run build`

Expected: webpack exits `0`.

- [ ] **Step 6: Commit**

```powershell
git add crate/src/lib.rs js/paint.js
git commit -m "feat: spawn compact sheep from the brush"
```

### Task 4: Implement Plant Seeking and Cohesive Movement

**Files:**
- Modify: `crate/src/lib.rs`
- Modify: `crate/src/species.rs`
- Test: `crate/src/lib.rs`

**Interfaces:**
- Produces: private `Universe::nearest_plant_direction(x, y, radius) -> Option<(i32, i32)>`.
- Produces: private `Universe::update_sheep_core(id: u8, x: i32, y: i32)` called from `update_sheep` for core cells.

- [ ] **Step 1: Write failing movement tests**

```rust
#[test]
fn sheep_scan_points_toward_the_nearest_plant() {
    let mut universe = Universe::new(30, 30);
    let plant_index = universe.get_index(15, 10);
    universe.cells[plant_index].species = Species::Plant;
    assert_eq!(universe.nearest_plant_direction(10, 10, 5), Some((1, 0)));
}

#[test]
fn sheep_movement_preserves_id_and_pixel_limit() {
    let mut universe = Universe::new(30, 30);
    universe.spawn_sheep(10, 10);
    let id = *universe.sheep.keys().next().unwrap();
    let plant_index = universe.get_index(15, 10);
    universe.cells[plant_index].species = Species::Plant;
    for _ in 0..120 { universe.tick(); }
    let sheep_cells: Vec<Cell> = universe.cells.iter().copied()
        .filter(|cell| cell.species == Species::Sheep && cell.ra == id)
        .collect();
    assert!(!sheep_cells.is_empty());
    assert!(sheep_cells.len() <= 10);
}
```

- [ ] **Step 2: Run tests and verify RED**

Run: `cd crate && cargo test sheep_`

Expected: fails because the scan and movement functions do not exist.

- [ ] **Step 3: Implement bounded plant scanning**

Scan concentric Manhattan rings from distance `1` through `5`, checking bounds before indexing. Return `(-1|0|1, -1|0|1)` toward the first closest plant and stop scanning immediately. Do not change `SandApi::get()` bounds.

- [ ] **Step 4: Implement cooldown movement and body following**

On core updates, decrement `move_cooldown`; when it reaches zero, reset it to `8 + rand_int(5)`. Prefer the plant direction, otherwise preserve horizontal direction with a low random turn chance. Reject destinations containing wall, stone, carcass, water, acid, or lava.

Move the core one grid cell and move one same-ID body pixel into the old core position. For remaining body pixels, attempt one-cell movement toward the core only when their Chebyshev distance exceeds `2`. Remove same-ID body pixels that remain farther than `4` cells from the core for 30 body updates, encoded in the upper bits of `rb` while preserving the body role in bit 0.

- [ ] **Step 5: Verify GREEN and regression suite**

Run: `cd crate && cargo test`

Expected: all Rust tests pass.

- [ ] **Step 6: Commit**

```powershell
git add crate/src/lib.rs crate/src/species.rs
git commit -m "feat: make sheep seek plants and move cohesively"
```

### Task 5: Implement Eating, Growth, Maturity, and Lethal Hazards

**Files:**
- Modify: `crate/src/lib.rs`
- Modify: `crate/src/species.rs`
- Test: `crate/src/lib.rs`

**Interfaces:**
- Consumes: `SheepState` and movement update from Tasks 2 and 4.
- Produces: growth capped at 10, `mature_steps = Some(800)`, starvation and submersion counters, and gradual death through `SheepState::dying_steps`.

- [ ] **Step 1: Write failing lifecycle tests**

```rust
#[test]
fn sheep_eats_one_plant_and_grows_without_exceeding_ten_pixels() {
    let mut universe = Universe::new(30, 30);
    universe.spawn_sheep(10, 10);
    let id = *universe.sheep.keys().next().unwrap();
    for x in 8..=12 {
        for y in 8..=12 {
            if universe.get_cell(x, y).species == Species::Empty {
                let index = universe.get_index(x, y);
                universe.cells[index].species = Species::Plant;
            }
        }
    }
    for _ in 0..1000 { universe.tick(); }
    assert!(universe.sheep.get(&id).unwrap().size <= 10);
    assert_eq!(universe.sheep.get(&id).unwrap().mature_steps, Some(800));
}

#[test]
fn mature_sheep_dies_when_its_800_step_timer_expires() {
    let mut universe = Universe::new(30, 30);
    universe.spawn_sheep(10, 10);
    let id = *universe.sheep.keys().next().unwrap();
    universe.sheep.get_mut(&id).unwrap().size = 10;
    universe.sheep.get_mut(&id).unwrap().mature_steps = Some(1);
    universe.tick();
    assert!(universe.cells.iter().any(|cell| cell.species == Species::Carcass));
    for _ in 0..20 { universe.tick(); }
    assert!(!universe.sheep.contains_key(&id));
    assert!(!universe.cells.iter().any(|cell| {
        cell.species == Species::Sheep && cell.ra == id
    }));
}
```

- [ ] **Step 2: Run tests and verify RED**

Run: `cd crate && cargo test sheep_`

Expected: assertions fail because eating, maturity, and death are absent.

- [ ] **Step 3: Implement throttled eating and growth**

Every core update decrements `eat_cooldown`; reset to `20 + rand_int(11)` at zero. Search the eight adjacent cells around every same-ID sheep pixel for one plant. Replace at most one plant with a new body cell when size is below 10; when size is already 10, replace the plant with empty and add `20` energy capped at `200`. Update `size` from successful placement only.

When size first reaches 10, set `mature_steps` to `Some(800)`. Decrement it once per core update and set `dying_steps = Some(0)` at zero. A dying sheep no longer moves or eats. On each subsequent core update, convert the closest remaining same-ID sheep cell to `Species::Carcass`, increment `dying_steps`, and remove the state entry only when no same-ID sheep cells remain.

- [ ] **Step 4: Implement starvation and hazards**

Spend one energy every 20 core updates. At zero energy, increment starvation; start dying at `500`. Reset starvation after eating. Start dying immediately when any sheep pixel touches fire, lava, or acid. Increment `submerged_steps` while the core is surrounded on at least three cardinal sides by water; start dying at `180`, otherwise reset it.

Each converted carcass cell receives a varied `ra` shade and `rb = 0`. If the core is converted before the body is exhausted, move the core marker to the nearest remaining same-ID sheep cell so death propagation can finish.

- [ ] **Step 5: Verify GREEN**

Run: `cd crate && cargo test`

Expected: all lifecycle and earlier tests pass.

- [ ] **Step 6: Commit**

```powershell
git add crate/src/lib.rs crate/src/species.rs
git commit -m "feat: add sheep growth and lifecycle"
```

### Task 6: Decompose Carcasses with Fungus

**Files:**
- Modify: `crate/src/species.rs`
- Test: `crate/src/lib.rs`

**Interfaces:**
- Produces: carcass decay only when fungus is adjacent, with deterministic probability driven by `SandApi` RNG.
- Consumes: `Species::Carcass`, `Species::Dust`, and `Species::Fungus`.

- [ ] **Step 1: Write failing carcass tests**

Add these tests to the `crate/src/lib.rs` test module:

```rust
#[test]
fn carcass_does_not_decay_without_fungus() {
    let mut universe = Universe::new(20, 20);
    let carcass_index = universe.get_index(10, 10);
    let support_index = universe.get_index(10, 11);
    universe.cells[carcass_index].species = Species::Carcass;
    universe.cells[support_index].species = Species::Wall;
    for _ in 0..700 { universe.tick(); }
    assert_eq!(universe.get_cell(10, 10).species, Species::Carcass);
}

#[test]
fn fungus_eventually_converts_carcass_to_dust_or_fungus() {
    let mut universe = Universe::new(20, 20);
    let carcass_index = universe.get_index(10, 10);
    let fungus_index = universe.get_index(11, 10);
    let carcass_support = universe.get_index(10, 11);
    let fungus_support = universe.get_index(11, 11);
    universe.cells[carcass_index].species = Species::Carcass;
    universe.cells[fungus_index].species = Species::Fungus;
    universe.cells[carcass_support].species = Species::Wall;
    universe.cells[fungus_support].species = Species::Wall;
    for _ in 0..700 { universe.tick(); }
    assert_ne!(universe.get_cell(10, 10).species, Species::Carcass);
}
```

- [ ] **Step 2: Run tests and verify RED**

Run: `cd crate && cargo test carcass`

Expected: the fungus decomposition test fails because carcasses only use stone behavior.

- [ ] **Step 3: Implement fungus-triggered decay**

In `update_carcass`, inspect the eight neighbors. If none is fungus, apply stone gravity and return. If fungus is adjacent, increment `rb` with wrapping-safe arithmetic on one update in four. At `rb >= 12`, choose `Dust` when `rand_int(100) < 85`, otherwise `Fungus`, and replace the carcass cell. This yields visible decomposition over hundreds of ticks without scanning the full grid.

Before decay, check adjacent fire or lava and convert the carcass to dust immediately.

- [ ] **Step 4: Verify GREEN**

Run: `cd crate && cargo test carcass`

Expected: both tests pass with the fixed seeded RNG.

Run: `cd crate && cargo test`

Expected: the full Rust suite passes.

- [ ] **Step 5: Commit**

```powershell
git add crate/src/species.rs
git commit -m "feat: let fungus decompose sheep carcasses"
```

### Task 7: Restore State after External Loads and Run End-to-End QA

**Files:**
- Modify: `crate/src/lib.rs`
- Modify: `js/components/ui.js`
- Modify: `crate/tests/web.rs`
- Test: `crate/tests/web.rs`

**Interfaces:**
- Produces: exported `Universe::rebuild_sheep_states()` callable after direct typed-array writes.
- Verifies: toolbar selection, one-click spawning, growth, death, decay, pause, single-step, undo, reset, and shared-data reconstruction.

- [ ] **Step 1: Add a failing browser reconstruction test**

Replace the existing `pass` browser test with:

```rust
#[wasm_bindgen_test]
fn rebuilding_after_pixel_restore_recovers_a_sheep() {
    let mut universe = sandtable::Universe::new(20, 20);
    assert!(universe.spawn_sheep(10, 10));
    universe.rebuild_sheep_states();
    assert_eq!(universe.sheep_count(), 1);
}
```

Expose a read-only `sheep_count() -> u32` for verification.

- [ ] **Step 2: Run the browser test and verify RED**

Run: `cd crate && wasm-pack test --headless --chrome`

Expected: compilation fails until reconstruction and count are exported. If Chrome is unavailable, record that exact blocker and retain the native reconstruction tests as the automated gate.

- [ ] **Step 3: Export reconstruction and call it after direct memory writes**

Move `rebuild_sheep_states()` into the `#[wasm_bindgen] impl Universe` without changing its implementation. Add:

```rust
pub fn sheep_count(&self) -> u32 {
    self.sheep.len() as u32
}
```

In both `loadSVG()` and the downloaded-creation `img.onload` path in `js/components/ui.js`, call `universe.rebuild_sheep_states()` after filling `cellsData` and before pushing the undo snapshot.

Replace the commented paused tick control with a visible Chinese single-step button:

```jsx
{paused && <button onClick={() => universe.tick()}>单步</button>}
```

This button calls exactly one simulation tick and leaves `window.paused` set to `true`.

- [ ] **Step 4: Run automated verification**

Run: `cd crate && cargo test`

Expected: all native tests pass.

Run: `cd crate && cargo check --target wasm32-unknown-unknown`

Expected: exit `0`; the existing unused panic-hook warning is acceptable.

Run: `pnpm run build`

Expected: webpack exits `0`; existing Browserslist and asset-size warnings are acceptable.

- [ ] **Step 5: Run Browser plugin interaction QA**

Start the documented dev server and test this flow at `http://localhost:8080/`:

1. Page title is `物理沙盒`, DOM is nonblank, and no framework overlay appears.
2. Click the unique `羊` button, then click the canvas once; visually confirm one compact white group, not a brush-sized disk.
3. Draw plants near the sheep and observe seeking, gradual consumption, and growth capped at roughly ten pixels.
4. Pause and single-step; confirm the sheep advances only on steps while paused.
5. Trigger undo and reset; confirm no aliasing panic and no orphan sheep state.
6. Run until maturity death, place fungus next to the carcass, and observe gray-white carcass pixels become mostly dust.
7. Check console logs for Rust/WASM panic, recursive borrow errors, and application errors.
8. Capture one desktop screenshot and one mobile-width screenshot; confirm toolbar labels do not clip or overlap.

- [ ] **Step 6: Commit**

```powershell
git add crate/src/lib.rs crate/tests/web.rs js/components/ui.js
git commit -m "test: verify sheep lifecycle and shared-world recovery"
```

- [ ] **Step 7: Final diff and history check**

Run: `git diff --check`

Expected: exit `0` with no whitespace errors.

Run: `git status --short`

Expected: only the pre-existing unrelated worktree changes remain; no sheep implementation files are unstaged.

Run: `git log --oneline -7`

Expected: the sheep feature commits appear in task order after the design and implementation-plan commits.
