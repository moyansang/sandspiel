# Sheep Balance Tuning Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Increase mature sheep size to 14 pixels, darken sheep to a white-gray lightness range of approximately 0.78-0.84, and extend mature lifetime to 1400 core updates.

**Architecture:** Keep sheep simulation rules in the existing Rust `Universe` implementation and rendering color in the existing GLSL material branch. Replace duplicated simulation literals with named constants so growth and reconstructed state cannot drift apart.

**Tech Stack:** Rust, wasm-bindgen, WebAssembly, GLSL, webpack-dev-server, native Rust tests, in-app browser QA.

## Global Constraints

- Newborn sheep remains approximately 5 pixels.
- Maximum sheep size is exactly 14 pixels.
- Mature lifetime is exactly 1400 sheep core updates and starts when size reaches 14.
- Sheep lightness is approximately 0.78-0.84 while hue and saturation behavior remain unchanged.
- Movement, feeding cadence, energy, hazards, death conversion, and carcass decomposition remain unchanged.
- Complete and push each task before asking the user to confirm the next task.

---

### Task 1: Increase Sheep Size And Lifetime

**Files:**
- Modify: `crate/src/lib.rs`
- Test: `crate/src/lib.rs`

**Interfaces:**
- Produces: `SHEEP_MAX_SIZE: u8 = 14` and `SHEEP_MATURE_LIFETIME: u16 = 1400` for sheep state reconstruction and feeding lifecycle logic.
- Preserves: `Universe::spawn_sheep`, `Universe::rebuild_sheep_states`, and `Universe::update_sheep_lifecycle` signatures.

- [ ] **Step 1: Write failing lifecycle tests**

Change the rebuild expectation and growth test so they require the new values before implementation:

```rust
#[test]
fn rebuild_uses_default_state_and_caps_size_at_fourteen() {
    let mut universe = Universe::new(20, 20);
    for x in 0..15 {
        let index = universe.get_index(x, 0);
        universe.cells[index] = Cell {
            species: Species::Sheep,
            ra: 7,
            rb: SHEEP_BODY,
            clock: 0,
        };
    }

    universe.rebuild_sheep_states();

    assert_eq!(universe.sheep[&7].size, 14);
    assert_eq!(universe.sheep[&7].mature_steps, Some(1400));
}

#[test]
fn sheep_starts_mature_lifespan_at_fourteen_pixels() {
    let mut universe = Universe::new(30, 30);
    assert!(universe.spawn_sheep(10, 10));
    universe.sheep.get_mut(&1).unwrap().size = 13;
    universe.sheep.get_mut(&1).unwrap().eat_cooldown = 1;
    let plant_index = universe.get_index(12, 10);
    universe.cells[plant_index].species = Species::Plant;

    universe.update_sheep_core(1, 10, 10);

    assert_eq!(universe.sheep[&1].size, 14);
    assert_eq!(universe.sheep[&1].mature_steps, Some(1400));
}
```

- [ ] **Step 2: Run tests and verify RED**

Run:

```powershell
cargo test --manifest-path crate/Cargo.toml --target x86_64-pc-windows-msvc sheep_starts_mature_lifespan_at_fourteen_pixels
cargo test --manifest-path crate/Cargo.toml --target x86_64-pc-windows-msvc rebuild_uses_default_state_and_caps_size_at_fourteen
```

Expected: both tests fail because the implementation still caps size at 10 and sets 800 mature steps.

- [ ] **Step 3: Add constants and use them in lifecycle logic**

Near the existing sheep role constants, add:

```rust
const SHEEP_MAX_SIZE: u8 = 14;
const SHEEP_MATURE_LIFETIME: u16 = 1400;
```

In `rebuild_sheep_states`, replace the size cap and mature state literals:

```rust
let size = cells.len().min(SHEEP_MAX_SIZE as usize) as u8;

mature_steps: if size == SHEEP_MAX_SIZE {
    Some(SHEEP_MATURE_LIFETIME)
} else {
    None
},
```

In `update_sheep_lifecycle`, replace the growth threshold and timer literals:

```rust
if size < SHEEP_MAX_SIZE {
    // Keep the existing plant-to-body conversion.
    let state = self.sheep.get_mut(&id).unwrap();
    state.size += 1;
    if state.size == SHEEP_MAX_SIZE && state.mature_steps.is_none() {
        state.mature_steps = Some(SHEEP_MATURE_LIFETIME);
    }
} else {
    // Keep the existing mature feeding and energy restoration.
}
```

Make these exact test updates in the existing test module:

```rust
// In sheep_movement_preserves_id_and_pixel_limit:
assert!(sheep_cells.len() <= SHEEP_MAX_SIZE as usize);

// Keep spawn_sheep_initializes_newborn_state unchanged:
assert_eq!(universe.sheep[&1].size, 5);
```

Rename `rebuild_uses_default_state_and_caps_size_at_ten` to `rebuild_uses_default_state_and_caps_size_at_fourteen`, use 15 input pixels, and expect size 14 with `Some(1400)`. Rename `sheep_starts_mature_lifespan_at_ten_pixels` to `sheep_starts_mature_lifespan_at_fourteen_pixels`, set its starting size to 13, and expect size 14 with `Some(1400)`.

- [ ] **Step 4: Run formatting and verify GREEN**

Run:

```powershell
cargo fmt --manifest-path crate/Cargo.toml
cargo test --manifest-path crate/Cargo.toml --target x86_64-pc-windows-msvc sheep_
cargo test --manifest-path crate/Cargo.toml --target x86_64-pc-windows-msvc
```

Expected: all sheep tests and all native tests pass.

- [ ] **Step 5: Commit, push, and pause for confirmation**

```powershell
git add crate/src/lib.rs
git commit -m "feat: enlarge sheep and extend lifetime"
git push origin agent/sheep-material
```

Report the test totals and commit hash, then wait for user confirmation before Task 2.

---

### Task 2: Darken Sheep And Verify In Browser

**Files:**
- Modify: `js/glsl/sand.glsl`

**Interfaces:**
- Consumes: serialized sheep species id `20` and the existing shader `data.b` variation.
- Produces: sheep HSL lightness range approximately 0.78-0.84.

- [ ] **Step 1: Record the pre-change shader assertion as RED**

Run:

```powershell
$shader = Get-Content js/glsl/sand.glsl -Raw
if ($shader -notmatch 'lightness = 0\.78 \+ data\.b \* 0\.06;') { throw 'Expected new sheep lightness is not implemented' }
```

Expected: command fails because the shader still uses base lightness `0.92`.

- [ ] **Step 2: Change only sheep lightness**

Keep hue and saturation unchanged and update the sheep shader branch to:

```glsl
} else if (type == 20) { // sheep
  hue = 0.0;
  saturation = 0.04;
  lightness = 0.78 + data.b * 0.06;
```

- [ ] **Step 3: Verify shader assertion and builds**

Run:

```powershell
$shader = Get-Content js/glsl/sand.glsl -Raw
if ($shader -notmatch 'lightness = 0\.78 \+ data\.b \* 0\.06;') { throw 'Sheep lightness assertion failed' }
cargo test --manifest-path crate/Cargo.toml --target x86_64-pc-windows-msvc
pnpm run build
```

Expected: shader assertion succeeds, all native tests pass, and webpack reports a successful production build.

- [ ] **Step 4: Run browser QA on a new port**

Start the server on an unused port such as 8082:

```powershell
pnpm exec webpack-dev-server --host 0.0.0.0 --port 8082
```

In the in-app browser:

1. Open `http://localhost:8082/`.
2. Confirm exactly one visible `羊` material label exists.
3. Select `羊`, click the canvas once, and confirm a compact white-gray multi-pixel sheep appears.
4. Place plants nearby and confirm the sheep can grow beyond 10 pixels toward 14.
5. Confirm browser console contains no errors.

- [ ] **Step 5: Commit, push, and hand off the live port**

```powershell
git add js/glsl/sand.glsl
git commit -m "style: darken sheep material"
git push origin agent/sheep-material
```

Provide the live port and commit hash, then wait for user confirmation before resuming the broader carcass decomposition plan.
