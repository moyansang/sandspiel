# Sheep Flexible Round Body Design

## Goal

Keep each multi-pixel sheep as a compact, approximately circular cluster while preserving flexible movement through a material sandbox. This change pauses the pending sheep color task and does not alter the agreed 14-pixel mature size or 1400-step mature lifetime.

## Shape Model

- The core pixel is the center of the sheep cluster.
- Body pixels prefer an ordered set of slots within approximately two pixels of the core.
- The preferred slots form a rounded footprint inside a 5-by-5 area.
- A mature sheep contains one core and up to thirteen body pixels.
- Fourteen cells cannot be perfectly axis-symmetric on a square grid. The single extra edge pixel follows the current horizontal direction so the cluster does not permanently lean to one side.
- The existing five-pixel newborn remains a compact cross centered on its core.

## Flexible Cohesion

- Body organization is soft rather than rigid: only a small number of body pixels move toward preferred slots during each core update.
- A body pixel moves at most one grid cell during one organization update.
- Walls, plants, hazards, other sheep, and all occupied foreign cells are never overwritten.
- The cluster may flatten or stretch briefly beside obstacles.
- When preferred slots become available again, the body gradually reforms around the core.
- Existing stranded-pixel cleanup remains as a fallback for body pixels that cannot reconnect.

## Growth

- Eating still occurs at the existing 20-to-30-update cadence.
- When a sheep below 14 pixels eats a plant, the plant is consumed only if a free preferred body slot exists around the core.
- The new body pixel is created in that free preferred slot, not at the former plant position.
- If every preferred slot is blocked, the plant remains and growth waits for a later feeding attempt.
- Reaching 14 pixels starts the existing 1400-step mature lifetime.
- Mature feeding continues to restore energy according to the existing rules.

## Movement

- The core continues to use the existing plant-seeking and movement cadence.
- After the core moves, the old core position may become a body pixel according to the existing movement exchange.
- Body organization then fills the nearest open preferred slots and pulls outer pixels inward one cell at a time.
- Movement never changes sheep identity or total pixel count.

## State And Compatibility

- No serialized species IDs or public WebAssembly method signatures change.
- Undo, reset, and shared-seed reconstruction continue to rebuild sheep identity and core state from cells.
- A loaded irregular sheep shape is accepted and gradually reorganized instead of being replaced instantly.
- Death, carcass conversion, energy, drowning, starvation, and hazard behavior remain unchanged.

## Verification

- The five-pixel newborn occupies the core plus four adjacent slots.
- Growth from 13 to 14 creates the new body within the preferred round footprint and removes exactly one plant.
- Growth waits without consuming the plant when all preferred slots are blocked.
- An irregular body moves at most one cell per update toward an open preferred slot.
- A sheep reforms into the preferred footprint after the obstacle is removed.
- Movement preserves identity, pixel count, and the 14-pixel maximum.
- Existing sheep tests and the full native Rust suite pass.
- The WebAssembly build succeeds and browser QA shows a compact round cluster without console errors.

