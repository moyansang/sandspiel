# Sheep Flexible Round Body Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make each sheep grow and move as a flexible, compact, approximately circular 14-pixel cluster.

**Architecture:** Add a deterministic preferred-slot model around the existing sheep core, with the final asymmetric edge slot mirrored by horizontal direction. Growth allocates into a free preferred slot, while cohesion moves at most one off-footprint body pixel one cell toward a vacant preferred slot per core update.

**Tech Stack:** Rust, wasm-bindgen, native Rust tests, WebAssembly, webpack-dev-server, in-app browser QA.

## Global Constraints

- Newborn sheep remains the existing five-pixel cross.
- Mature sheep remains exactly 14 pixels with a 1400-step mature lifetime.
- Preferred footprint remains inside a 5-by-5 area centered on the core.
- A body pixel moves at most one grid cell during one organization update.
- Occupied foreign cells are never overwritten.
- If every preferred growth slot is blocked, the plant remains and growth waits.
- Loaded irregular shapes reorganize gradually instead of teleporting.
- Movement, feeding cadence, energy, hazards, death conversion, and carcass behavior otherwise remain unchanged.
- The pending darker-color change remains outside this plan.
- Complete, review, push, and provide a playable port after each task; wait for user confirmation before continuing.

---

### Task 1: Preferred Round Slots And Growth

**Files:**
- Modify: `crate/src/lib.rs`
- Test: `crate/src/lib.rs`

**Interfaces:**
- Produces: `SHEEP_ROUND_BODY_OFFSETS: [(i32, i32); 13]`.
- Produces: `Universe::sheep_preferred_slots(&self, id: u8) -> Vec<(i32, i32)>`.
- Produces: `Universe::free_sheep_growth_slot(&self, id: u8) -> Option<(i32, i32)>`.
- Preserves: `Universe::update_sheep_lifecycle(&mut self, id: u8) -> bool`.

- [ ] **Step 1: Write failing preferred-slot tests**

Add tests that describe the footprint and blocked-growth behavior:

```rust
#[test]
fn sheep_round_slots_stay_within_two_cells_and_mirror_direction() {
    let mut universe = Universe::new(30, 30);
    assert!(universe.spawn_sheep(10, 10));

    universe.sheep.get_mut(&1).unwrap().direction = 1;
    let right_slots = universe.sheep_preferred_slots(1);
    universe.sheep.get_mut(&1).unwrap().direction = -1;
    let left_slots = universe.sheep_preferred_slots(1);

    assert_eq!(right_slots.len(), 13);
    assert!(right_slots.iter().all(|(x, y)| {
        (x - 10).abs() <= 2 && (y - 10).abs() <= 2 && (*x, *y) != (10, 10)
    }));
    assert!(right_slots.contains(&(12, 11)));
    assert!(left_slots.contains(&(8, 11)));
}

#[test]
fn blocked_round_slots_keep_plant_and_delay_growth() {
    let mut universe = Universe::new(30, 30);
    assert!(universe.spawn_sheep(10, 10));
    let slots = universe.sheep_preferred_slots(1);
    for (x, y) in slots {
        let index = universe.get_index(x, y);
        if universe.cells[index].species == Species::Empty {
            universe.cells[index].species = Species::Stone;
        }
    }
    let plant_index = universe.get_index(10, 12);
    universe.cells[plant_index].species = Species::Plant;
    universe.sheep.get_mut(&1).unwrap().eat_cooldown = 1;
    let original_size = universe.sheep[&1].size;

    universe.update_sheep_core(1, 10, 10);

    assert_eq!(universe.cells[plant_index].species, Species::Plant);
    assert_eq!(universe.sheep[&1].size, original_size);
}
```

Replace `sheep_eats_one_adjacent_plant_and_grows` with an assertion that the plant becomes empty and exactly one previously empty preferred slot becomes a sheep body.

- [ ] **Step 2: Run tests and verify RED**

Run:

```powershell
cargo test --manifest-path crate/Cargo.toml --target x86_64-pc-windows-msvc sheep_round_slots_stay_within_two_cells_and_mirror_direction
cargo test --manifest-path crate/Cargo.toml --target x86_64-pc-windows-msvc blocked_round_slots_keep_plant_and_delay_growth
```

Expected: compilation fails because the preferred-slot methods do not exist. After adding only test-local declarations if needed, blocked growth fails because current growth converts the plant position into a body.

- [ ] **Step 3: Add the preferred footprint**

Near the existing sheep constants, add the thirteen right-facing body offsets:

```rust
const SHEEP_ROUND_BODY_OFFSETS: [(i32, i32); 13] = [
    (-1, -1), (0, -1), (1, -1),
    (-1, 0), (1, 0),
    (-1, 1), (0, 1), (1, 1),
    (-2, 0), (2, 0),
    (0, -2), (0, 2),
    (2, 1),
];
```

Add methods inside `impl Universe`:

```rust
fn sheep_preferred_slots(&self, id: u8) -> Vec<(i32, i32)> {
    let state = match self.sheep.get(&id) {
        Some(state) => state,
        None => return Vec::new(),
    };
    let mirror = if state.direction < 0 { -1 } else { 1 };
    SHEEP_ROUND_BODY_OFFSETS
        .iter()
        .map(|(dx, dy)| (state.core_x + dx * mirror, state.core_y + dy))
        .filter(|(x, y)| self.checked_index(*x, *y).is_some())
        .collect()
}

fn free_sheep_growth_slot(&self, id: u8) -> Option<(i32, i32)> {
    self.sheep_preferred_slots(id).into_iter().find(|(x, y)| {
        self.checked_cell(*x, *y)
            .map(|cell| cell.species == Species::Empty)
            .unwrap_or(false)
    })
}
```

- [ ] **Step 4: Allocate growth into the footprint**

In the below-maximum branch of `update_sheep_lifecycle`, obtain `free_sheep_growth_slot(id)` before consuming the plant. When no slot exists, leave the plant and size unchanged. When a slot exists, clear the plant and create a `SHEEP_BODY` cell at the slot:

```rust
if size < SHEEP_MAX_SIZE {
    if let Some((slot_x, slot_y)) = self.free_sheep_growth_slot(id) {
        self.set_checked_cell(plant_x, plant_y, EMPTY_CELL);
        self.set_checked_cell(
            slot_x,
            slot_y,
            Cell {
                species: Species::Sheep,
                ra: id,
                rb: SHEEP_BODY,
                clock: 0,
            },
        );
        let state = self.sheep.get_mut(&id).unwrap();
        state.size += 1;
        if state.size == SHEEP_MAX_SIZE && state.mature_steps.is_none() {
            state.mature_steps = Some(SHEEP_MATURE_LIFETIME);
        }
    }
} else {
    // Preserve the existing mature plant consumption and energy restoration.
}
```

- [ ] **Step 5: Verify Task 1**

Run:

```powershell
cargo fmt --manifest-path crate/Cargo.toml
cargo test --manifest-path crate/Cargo.toml --target x86_64-pc-windows-msvc sheep_
cargo test --manifest-path crate/Cargo.toml --target x86_64-pc-windows-msvc
```

Expected: all sheep tests and the full native suite pass.

- [ ] **Step 6: Commit, review, push, and pause**

```powershell
git add crate/src/lib.rs
git commit -m "feat: grow sheep into round footprint"
git push origin agent/sheep-material
```

Start the feature worktree on a new unused port, verify the `羊` control and console, provide the URL, and wait for user confirmation before Task 2.

---

### Task 2: Flexible Body Reorganization

**Files:**
- Modify: `crate/src/lib.rs`
- Test: `crate/src/lib.rs`

**Interfaces:**
- Consumes: `Universe::sheep_preferred_slots(&self, id: u8) -> Vec<(i32, i32)>` from Task 1.
- Produces: `Universe::round_sheep_body_step(&mut self, id: u8, core_x: i32, core_y: i32) -> bool`.
- Preserves: `Universe::update_sheep_bodies(&mut self, id: u8, core_x: i32, core_y: i32)`.

- [ ] **Step 1: Write failing flexible-cohesion tests**

Add focused tests:

```rust
#[test]
fn irregular_body_moves_one_cell_toward_open_round_slot() {
    let mut universe = Universe::new(30, 30);
    assert!(universe.spawn_sheep(10, 10));
    let distant_index = universe.get_index(10, 14);
    universe.cells[distant_index] = Cell {
        species: Species::Sheep,
        ra: 1,
        rb: SHEEP_BODY,
        clock: 0,
    };
    universe.sheep.get_mut(&1).unwrap().size += 1;

    assert!(universe.round_sheep_body_step(1, 10, 10));

    assert_eq!(universe.cells[distant_index].species, Species::Empty);
    assert_eq!(universe.cells[universe.get_index(10, 13)].species, Species::Sheep);
}

#[test]
fn round_body_step_never_overwrites_foreign_material() {
    let mut universe = Universe::new(30, 30);
    assert!(universe.spawn_sheep(10, 10));
    let distant_index = universe.get_index(10, 14);
    let blocked_index = universe.get_index(10, 13);
    universe.cells[distant_index] = Cell {
        species: Species::Sheep,
        ra: 1,
        rb: SHEEP_BODY,
        clock: 0,
    };
    universe.cells[blocked_index].species = Species::Stone;

    universe.round_sheep_body_step(1, 10, 10);

    assert_eq!(universe.cells[blocked_index].species, Species::Stone);
}
```

Add a multi-update test that removes a blocking stone, calls `update_sheep_bodies` repeatedly, and asserts every surviving body finishes in `sheep_preferred_slots(1)` without changing identity or count.

- [ ] **Step 2: Run tests and verify RED**

Run:

```powershell
cargo test --manifest-path crate/Cargo.toml --target x86_64-pc-windows-msvc irregular_body_moves_one_cell_toward_open_round_slot
cargo test --manifest-path crate/Cargo.toml --target x86_64-pc-windows-msvc round_body_step_never_overwrites_foreign_material
```

Expected: compilation fails because `round_sheep_body_step` does not exist.

- [ ] **Step 3: Implement one-pixel organization**

Implement `round_sheep_body_step` with this deterministic selection:

1. Build the preferred-slot list and retain only empty slots.
2. Collect this sheep's body pixels outside the preferred-slot set.
3. Select the off-footprint body with greatest Chebyshev distance from the core.
4. Select the vacant preferred slot with smallest Chebyshev distance to that body.
5. Move the body one cell toward that slot on both axes using `signum()`.
6. Move only when the one-step destination is empty and in bounds; otherwise return `false` without overwriting anything.
7. Preserve the body's `ra`, role bits, and stranded counter bits.

The signature is:

```rust
fn round_sheep_body_step(&mut self, id: u8, core_x: i32, core_y: i32) -> bool
```

Return `true` only when one body pixel moved.

- [ ] **Step 4: Integrate with existing cohesion and cleanup**

Call `round_sheep_body_step(id, core_x, core_y)` once at the start of `update_sheep_bodies`. Keep the existing scan that updates stranded counters and removes pixels after `SHEEP_STRAY_LIMIT`, but skip a body that already moved during this update by honoring its updated `clock`. Do not allow the legacy loop to move a second body in the same organization update.

Update the existing `distant_sheep_body_moves_one_cell_toward_the_core`, `stranded_sheep_body_is_removed_after_thirty_updates`, `sheep_core_moves_before_body_cohesion`, and movement-preservation tests only where their expected compact position changes. Do not weaken their identity, count, or cleanup assertions.

- [ ] **Step 5: Verify Task 2**

Run:

```powershell
cargo fmt --manifest-path crate/Cargo.toml
cargo test --manifest-path crate/Cargo.toml --target x86_64-pc-windows-msvc sheep_
cargo test --manifest-path crate/Cargo.toml --target x86_64-pc-windows-msvc
pnpm run build
```

Expected: all native tests pass and webpack completes successfully.

- [ ] **Step 6: Browser QA**

Start webpack-dev-server on a new unused port. In the in-app browser:

1. Confirm exactly one visible `羊` button and select it.
2. Spawn a sheep and place plants around it.
3. Confirm growth adds pixels near the core rather than at the plant location.
4. Confirm a mature sheep appears as a compact near-round cluster while moving.
5. Confirm no browser console errors, including no recursive mutable borrow error.

- [ ] **Step 7: Commit, review, push, and hand off**

```powershell
git add crate/src/lib.rs
git commit -m "feat: keep sheep bodies flexibly round"
git push origin agent/sheep-material
```

Provide the live port and commit hash, then wait for user confirmation before returning to the darker-color task.

