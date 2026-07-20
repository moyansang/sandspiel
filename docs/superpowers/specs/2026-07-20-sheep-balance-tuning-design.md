# Sheep Balance Tuning Design

## Scope

Tune the existing sheep creature so that it grows larger, renders darker, and remains alive longer. This change does not alter movement, feeding cadence, energy, hazards, death conversion, or carcass decomposition.

## Behavior

- A newborn sheep remains approximately 5 pixels.
- Eating plants grows the sheep until it reaches 14 pixels.
- Reaching 14 pixels starts the mature lifetime countdown.
- The mature lifetime is 1400 sheep core updates.
- A mature sheep cannot grow beyond 14 pixels; later feeding continues to restore energy according to the existing rules.
- Rebuilt state from undo, shared seeds, or other serialized worlds treats a 14-pixel sheep as mature and starts it with 1400 mature steps.

## Rendering

- Sheep remains a low-saturation neutral white-gray.
- Its HSL lightness range changes from approximately 0.92-0.98 to 0.78-0.84.
- Core and body variation remains visible through the existing per-cell rendering data.

## Implementation

- Introduce named constants for the maximum sheep size and mature lifetime in the Rust simulation.
- Replace the existing hard-coded size 10 and lifetime 800 checks with those constants.
- Adjust the sheep lightness base in `js/glsl/sand.glsl` without changing its hue or saturation behavior.
- Keep newborn spawning and all unrelated simulation parameters unchanged.

## Verification

- A test proves a sheep grows from 13 to 14 pixels and starts with 1400 mature steps.
- A test proves rebuilt state caps sheep size at 14 and restores a 1400-step mature lifetime.
- Existing sheep and full native Rust tests pass.
- The WebAssembly build succeeds.
- Browser QA confirms the sheep label works, sheep appears darker, and no console errors occur.

