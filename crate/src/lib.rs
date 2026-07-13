extern crate cfg_if;
extern crate js_sys;
extern crate rand;
extern crate rand_xoshiro;
extern crate wasm_bindgen;
extern crate web_sys;

mod species;
mod utils;

use rand::{Rng, SeedableRng};
use rand_xoshiro::SplitMix64;
use species::Species;
use std::collections::{HashMap, VecDeque};
use wasm_bindgen::prelude::*;
// use web_sys::console;

const SHEEP_BODY: u8 = 0;
const SHEEP_CORE: u8 = 1;
const SHEEP_ROLE_MASK: u8 = 1;
const SHEEP_STRAY_LIMIT: u8 = 30;
const NEWBORN_SHAPE: [(i32, i32, u8); 5] = [
    (0, 0, SHEEP_CORE),
    (-1, 0, SHEEP_BODY),
    (1, 0, SHEEP_BODY),
    (0, -1, SHEEP_BODY),
    (1, -1, SHEEP_BODY),
];

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

#[wasm_bindgen]
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Wind {
    dx: u8,
    dy: u8,
    pressure: u8,
    density: u8,
}

#[wasm_bindgen]
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cell {
    species: Species,
    ra: u8,
    rb: u8,
    clock: u8,
}

impl Cell {
    pub fn new(species: Species) -> Cell {
        Cell {
            species: species,
            ra: 100 + (js_sys::Math::random() * 50.) as u8,
            rb: 0,
            clock: 0,
        }
    }
    pub fn update(&self, api: SandApi) {
        self.species.update(*self, api);
    }
}

static EMPTY_CELL: Cell = Cell {
    species: Species::Empty,
    ra: 0,
    rb: 0,
    clock: 0,
};

#[wasm_bindgen]
pub struct Universe {
    width: i32,
    height: i32,
    cells: Vec<Cell>,
    undo_stack: VecDeque<Vec<Cell>>,
    winds: Vec<Wind>,
    burns: Vec<Wind>,
    generation: u8,
    rng: SplitMix64,
    sheep: HashMap<u8, SheepState>,
    next_sheep_id: u8,
}

pub struct SandApi<'a> {
    x: i32,
    y: i32,
    universe: &'a mut Universe,
}

impl<'a> SandApi<'a> {
    pub fn get(&mut self, dx: i32, dy: i32) -> Cell {
        if dx > 2 || dx < -2 || dy > 2 || dy < -2 {
            panic!("oob set");
        }
        let nx = self.x + dx;
        let ny = self.y + dy;
        if nx < 0 || nx > self.universe.width - 1 || ny < 0 || ny > self.universe.height - 1 {
            return Cell {
                species: Species::Wall,
                ra: 0,
                rb: 0,
                clock: self.universe.generation,
            };
        }
        self.universe.get_cell(nx, ny)
    }
    pub fn set(&mut self, dx: i32, dy: i32, v: Cell) {
        if dx > 2 || dx < -2 || dy > 2 || dy < -2 {
            panic!("oob set");
        }
        let nx = self.x + dx;
        let ny = self.y + dy;

        if nx < 0 || nx > self.universe.width - 1 || ny < 0 || ny > self.universe.height - 1 {
            return;
        }
        let i = self.universe.get_index(nx, ny);
        // v.clock += 1;
        self.universe.cells[i] = v;
        self.universe.cells[i].clock = self.universe.generation.wrapping_add(1);
    }
    pub fn get_fluid(&mut self) -> Wind {
        let idx = self.universe.get_index(self.x, self.y);

        self.universe.winds[idx]
    }
    pub fn set_fluid(&mut self, v: Wind) {
        let idx = self.universe.get_index(self.x, self.y);

        self.universe.burns[idx] = v;
    }

    pub fn rand_int(&mut self, n: i32) -> i32 {
        self.universe.rng.gen_range(0..n)
    }

    pub fn once_in(&mut self, n: i32) -> bool {
        self.rand_int(n) == 0
    }
    pub fn rand_dir(&mut self) -> i32 {
        let i = self.rand_int(1000);
        (i % 3) - 1
    }
    pub fn rand_dir_2(&mut self) -> i32 {
        let i = self.rand_int(1000);
        if (i % 2) == 0 {
            -1
        } else {
            1
        }
    }

    pub fn rand_vec(&mut self) -> (i32, i32) {
        let i = self.rand_int(2000);
        match i % 9 {
            0 => (1, 1),
            1 => (1, 0),
            2 => (1, -1),
            3 => (0, -1),
            4 => (-1, -1),
            5 => (-1, 0),
            6 => (-1, 1),
            7 => (0, 1),
            _ => (0, 0),
        }
    }

    pub fn rand_vec_8(&mut self) -> (i32, i32) {
        let i = self.rand_int(8);
        match i {
            0 => (1, 1),
            1 => (1, 0),
            2 => (1, -1),
            3 => (0, -1),
            4 => (-1, -1),
            5 => (-1, 0),
            6 => (-1, 1),
            _ => (0, 1),
        }
    }
}

#[wasm_bindgen]
impl Universe {
    pub fn reset(&mut self) {
        for x in 0..self.width {
            for y in 0..self.height {
                let idx = self.get_index(x, y);
                self.cells[idx] = EMPTY_CELL;
            }
        }
        self.sheep.clear();
        self.next_sheep_id = 1;
    }
    pub fn tick(&mut self) {
        // let mut next = self.cells.clone();
        // let dx = self.winds[(self.width * self.height / 2) as usize].dx;
        // let js: JsValue = (dx).into();
        // console::log_2(&"dx: ".into(), &js);

        for x in 0..self.width {
            for y in 0..self.height {
                let cell = self.get_cell(x, y);
                let wind = self.get_wind(x, y);
                Universe::blow_wind(
                    cell,
                    wind,
                    SandApi {
                        universe: self,
                        x,
                        y,
                    },
                )
            }
        }
        self.generation = self.generation.wrapping_add(1);
        for x in 0..self.width {
            let scanx = if self.generation % 2 == 0 {
                self.width - (1 + x)
            } else {
                x
            };

            for y in 0..self.height {
                let idx = self.get_index(scanx, y);
                let cell = self.get_cell(scanx, y);

                self.burns[idx] = Wind {
                    dx: 0,
                    dy: 0,
                    pressure: 0,
                    density: 0,
                };
                Universe::update_cell(
                    cell,
                    SandApi {
                        universe: self,
                        x: scanx,
                        y,
                    },
                );
            }
        }

        self.generation = self.generation.wrapping_add(1);
    }

    pub fn width(&self) -> i32 {
        self.width
    }

    pub fn height(&self) -> i32 {
        self.height
    }

    pub fn cells(&self) -> *const Cell {
        self.cells.as_ptr()
    }

    pub fn winds(&self) -> *const Wind {
        self.winds.as_ptr()
    }

    pub fn burns(&self) -> *const Wind {
        self.burns.as_ptr()
    }

    pub fn spawn_sheep(&mut self, x: i32, y: i32) -> bool {
        if x < 0 || x >= self.width || y < 0 || y >= self.height {
            return false;
        }

        let core_index = self.get_index(x, y);
        if self.cells[core_index].species != Species::Empty {
            return false;
        }

        let sheep_id = (0..255)
            .map(|offset| ((self.next_sheep_id as u16 - 1 + offset) % 255 + 1) as u8)
            .find(|id| !self.sheep.contains_key(id));
        let sheep_id = match sheep_id {
            Some(id) => id,
            None => return false,
        };

        let mut size = 0;
        for &(dx, dy, part) in &NEWBORN_SHAPE {
            let cell_x = x + dx;
            let cell_y = y + dy;
            if cell_x < 0 || cell_x >= self.width || cell_y < 0 || cell_y >= self.height {
                continue;
            }

            let index = self.get_index(cell_x, cell_y);
            if self.cells[index].species != Species::Empty {
                continue;
            }

            self.cells[index] = Cell {
                species: Species::Sheep,
                ra: sheep_id,
                rb: part,
                clock: self.generation,
            };
            size += 1;
        }

        self.sheep.insert(
            sheep_id,
            SheepState {
                core_x: x,
                core_y: y,
                energy: 100,
                size,
                direction: 1,
                move_cooldown: 8,
                eat_cooldown: 20,
                mature_steps: None,
                starvation_steps: 0,
                submerged_steps: 0,
                dying_steps: None,
            },
        );
        self.next_sheep_id = if sheep_id == 255 { 1 } else { sheep_id + 1 };
        true
    }

    pub fn paint(&mut self, x: i32, y: i32, size: i32, species: Species) {
        let size = size;
        let radius: f64 = (size as f64) / 2.0;

        let floor = (radius + 1.0) as i32;
        let ciel = (radius + 1.5) as i32;

        for dx in -floor..ciel {
            for dy in -floor..ciel {
                if (((dx * dx) + (dy * dy)) as f64) > (radius * radius) {
                    continue;
                };
                let px = x + dx;
                let py = y + dy;
                let i = self.get_index(px, py);

                if px < 0 || px > self.width - 1 || py < 0 || py > self.height - 1 {
                    continue;
                }
                if self.get_cell(px, py).species == Species::Empty || species == Species::Empty {
                    self.cells[i] = Cell {
                        species: species,
                        ra: 60
                            + (size as u8)
                            + (self.rng.gen::<f32>() * 30.) as u8
                            + ((self.generation % 127) as i8 - 60).abs() as u8,
                        rb: 0,
                        clock: self.generation,
                    }
                }
            }
        }
    }

    pub fn push_undo(&mut self) {
        self.undo_stack.push_front(self.cells.clone());
        self.undo_stack.truncate(50);
    }

    pub fn pop_undo(&mut self) {
        let old_state = self.undo_stack.pop_front();
        match old_state {
            Some(state) => self.cells = state,
            None => (),
        };
        self.rebuild_sheep_states();
    }

    pub fn flush_undos(&mut self) {
        self.undo_stack.clear();
    }

    pub fn new(width: i32, height: i32) -> Universe {
        let cells = (0..width * height).map(|_i| EMPTY_CELL).collect();
        let winds: Vec<Wind> = (0..width * height)
            .map(|_i| Wind {
                dx: 0,
                dy: 0,
                pressure: 0,
                density: 0,
            })
            .collect();

        let burns: Vec<Wind> = (0..width * height)
            .map(|_i| Wind {
                dx: 0,
                dy: 0,
                pressure: 0,
                density: 0,
            })
            .collect();
        let rng: SplitMix64 = SeedableRng::seed_from_u64(0x734f6b89de5f83cc);
        Universe {
            width,
            height,
            cells,
            undo_stack: VecDeque::with_capacity(50),
            burns,
            winds,
            generation: 0,
            rng,
            sheep: HashMap::new(),
            next_sheep_id: 1,
        }
    }

    pub fn rebuild_sheep_states(&mut self) {
        let mut sheep_cells: HashMap<u8, Vec<(usize, i32, i32, bool)>> = HashMap::new();

        for x in 0..self.width {
            for y in 0..self.height {
                let index = self.get_index(x, y);
                let cell = self.cells[index];
                if cell.species == Species::Sheep {
                    let is_core = cell.rb & SHEEP_ROLE_MASK == SHEEP_CORE;
                    sheep_cells
                        .entry(cell.ra)
                        .or_default()
                        .push((index, x, y, is_core));
                }
            }
        }

        self.sheep.clear();
        for (id, cells) in sheep_cells {
            let (core_index, core_x, core_y, _) = cells
                .iter()
                .find(|(_, _, _, is_core)| *is_core)
                .unwrap_or(&cells[0]);
            let size = cells.len().min(10) as u8;
            self.cells[*core_index].rb = SHEEP_CORE;
            self.sheep.insert(
                id,
                SheepState {
                    core_x: *core_x,
                    core_y: *core_y,
                    energy: 100,
                    size,
                    direction: 1,
                    move_cooldown: 8,
                    eat_cooldown: 20,
                    mature_steps: if size == 10 { Some(800) } else { None },
                    starvation_steps: 0,
                    submerged_steps: 0,
                    dying_steps: None,
                },
            );
        }
    }
}

//private methods
impl Universe {
    fn in_bounds(&self, x: i32, y: i32) -> bool {
        x >= 0 && x < self.width && y >= 0 && y < self.height
    }

    fn checked_index(&self, x: i32, y: i32) -> Option<usize> {
        self.in_bounds(x, y).then(|| self.get_index(x, y))
    }

    fn checked_cell(&self, x: i32, y: i32) -> Option<Cell> {
        self.checked_index(x, y).map(|index| self.cells[index])
    }

    fn set_checked_cell(&mut self, x: i32, y: i32, mut cell: Cell) -> bool {
        let index = match self.checked_index(x, y) {
            Some(index) => index,
            None => return false,
        };
        cell.clock = self.generation.wrapping_add(1);
        self.cells[index] = cell;
        true
    }

    fn nearest_plant_direction(&self, x: i32, y: i32, radius: i32) -> Option<(i32, i32)> {
        for distance in 1..=radius.clamp(0, 5) {
            for dx in -distance..=distance {
                let dy_magnitude = distance - dx.abs();
                let dy_candidates = if dy_magnitude == 0 {
                    [0, 0]
                } else {
                    [-dy_magnitude, dy_magnitude]
                };

                for (candidate_index, dy) in dy_candidates.iter().copied().enumerate() {
                    if candidate_index == 1 && dy_magnitude == 0 {
                        continue;
                    }
                    if self
                        .checked_cell(x + dx, y + dy)
                        .map(|cell| cell.species == Species::Plant)
                        .unwrap_or(false)
                    {
                        return Some((dx.signum(), dy.signum()));
                    }
                }
            }
        }
        None
    }

    fn adjacent_plant_for_sheep(&self, id: u8) -> Option<(i32, i32)> {
        for x in 0..self.width {
            for y in 0..self.height {
                let cell = self.get_cell(x, y);
                if cell.species != Species::Sheep || cell.ra != id {
                    continue;
                }
                for dx in -1..=1 {
                    for dy in -1..=1 {
                        if dx == 0 && dy == 0 {
                            continue;
                        }
                        if self
                            .checked_cell(x + dx, y + dy)
                            .map(|neighbor| neighbor.species == Species::Plant)
                            .unwrap_or(false)
                        {
                            return Some((x + dx, y + dy));
                        }
                    }
                }
            }
        }
        None
    }

    fn sheep_touches_hazard(&self, id: u8) -> bool {
        for x in 0..self.width {
            for y in 0..self.height {
                let cell = self.get_cell(x, y);
                if cell.species != Species::Sheep || cell.ra != id {
                    continue;
                }
                for dx in -1..=1 {
                    for dy in -1..=1 {
                        let species = self
                            .checked_cell(x + dx, y + dy)
                            .map(|neighbor| neighbor.species);
                        if matches!(
                            species,
                            Some(Species::Fire) | Some(Species::Lava) | Some(Species::Acid)
                        ) {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    fn advance_sheep_death(&mut self, id: u8) {
        let mut body = None;
        let mut core = None;
        for x in 0..self.width {
            for y in 0..self.height {
                let cell = self.get_cell(x, y);
                if cell.species == Species::Sheep && cell.ra == id {
                    if cell.rb & SHEEP_ROLE_MASK == SHEEP_CORE {
                        core = Some((x, y));
                    } else if body.is_none() {
                        body = Some((x, y));
                    }
                }
            }
        }

        if let Some((x, y)) = body.or(core) {
            let shade = 90 + self.rng.gen_range(0..40) as u8;
            self.set_checked_cell(
                x,
                y,
                Cell {
                    species: Species::Carcass,
                    ra: shade,
                    rb: 0,
                    clock: 0,
                },
            );
            if let Some(state) = self.sheep.get_mut(&id) {
                state.size = state.size.saturating_sub(1);
                state.dying_steps = Some(state.dying_steps.unwrap_or(0).saturating_add(1));
            }
        }

        if !self
            .cells
            .iter()
            .any(|cell| cell.species == Species::Sheep && cell.ra == id)
        {
            self.sheep.remove(&id);
        }
    }

    fn update_sheep_lifecycle(&mut self, id: u8) -> bool {
        if self
            .sheep
            .get(&id)
            .and_then(|state| state.dying_steps)
            .is_some()
        {
            self.advance_sheep_death(id);
            return true;
        }

        if self.sheep_touches_hazard(id) {
            self.sheep.get_mut(&id).unwrap().dying_steps = Some(0);
            self.advance_sheep_death(id);
            return true;
        }

        let (core_x, core_y) = {
            let state = &self.sheep[&id];
            (state.core_x, state.core_y)
        };
        let water_sides = [(1, 0), (-1, 0), (0, 1), (0, -1)]
            .iter()
            .filter(|(dx, dy)| {
                self.checked_cell(core_x + dx, core_y + dy)
                    .map(|cell| cell.species == Species::Water)
                    .unwrap_or(false)
            })
            .count();

        let mut should_die = false;
        {
            let state = self.sheep.get_mut(&id).unwrap();
            state.submerged_steps = if water_sides >= 3 {
                state.submerged_steps.saturating_add(1)
            } else {
                0
            };
            if state.submerged_steps >= 180 {
                should_die = true;
            }

            if let Some(steps) = state.mature_steps.as_mut() {
                *steps = steps.saturating_sub(1);
                if *steps == 0 {
                    should_die = true;
                }
            }

            if self.generation % 20 == 0 {
                state.energy = state.energy.saturating_sub(1);
            }
            if state.energy == 0 {
                state.starvation_steps = state.starvation_steps.saturating_add(1);
                if state.starvation_steps >= 500 {
                    should_die = true;
                }
            } else {
                state.starvation_steps = 0;
            }
        }

        if should_die {
            self.sheep.get_mut(&id).unwrap().dying_steps = Some(0);
            self.advance_sheep_death(id);
            return true;
        }

        let eat_now = {
            let state = self.sheep.get_mut(&id).unwrap();
            if state.eat_cooldown > 1 {
                state.eat_cooldown -= 1;
                false
            } else {
                true
            }
        };
        if !eat_now {
            return false;
        }

        let next_eat = 20 + self.rng.gen_range(0..11) as u8;
        self.sheep.get_mut(&id).unwrap().eat_cooldown = next_eat;
        if let Some((plant_x, plant_y)) = self.adjacent_plant_for_sheep(id) {
            let size = self.sheep[&id].size;
            if size < 10 {
                self.set_checked_cell(
                    plant_x,
                    plant_y,
                    Cell {
                        species: Species::Sheep,
                        ra: id,
                        rb: SHEEP_BODY,
                        clock: 0,
                    },
                );
                let state = self.sheep.get_mut(&id).unwrap();
                state.size += 1;
                state.energy = state.energy.saturating_add(20).min(200);
                if state.size == 10 && state.mature_steps.is_none() {
                    state.mature_steps = Some(800);
                }
            } else {
                self.set_checked_cell(plant_x, plant_y, EMPTY_CELL);
                let state = self.sheep.get_mut(&id).unwrap();
                state.energy = state.energy.saturating_add(20).min(200);
            }
        }
        false
    }

    fn update_sheep_core(&mut self, id: u8, x: i32, y: i32) {
        let (core_x, core_y, move_cooldown, mut direction) = match self.sheep.get(&id) {
            Some(state) => (
                state.core_x,
                state.core_y,
                state.move_cooldown,
                state.direction,
            ),
            None => return,
        };
        if (x, y) != (core_x, core_y) {
            return;
        }

        if self.update_sheep_lifecycle(id) {
            return;
        }

        if move_cooldown > 1 {
            self.sheep.get_mut(&id).unwrap().move_cooldown -= 1;
            self.update_sheep_bodies(id, core_x, core_y);
            return;
        }

        let plant_direction = self.nearest_plant_direction(core_x, core_y, 5);
        if plant_direction.is_none() && self.rng.gen_range(0..20) == 0 {
            direction = -direction;
        }
        let (dx, dy) = plant_direction.unwrap_or((direction as i32, 0));
        if dx != 0 {
            direction = dx as i8;
        }

        let can_enter = |cell: Cell| {
            cell.species == Species::Empty
                || (cell.species == Species::Sheep
                    && cell.ra == id
                    && cell.rb & SHEEP_ROLE_MASK == SHEEP_BODY)
        };
        let mut movement = self
            .checked_cell(core_x + dx, core_y + dy)
            .filter(|cell| can_enter(*cell))
            .map(|cell| (core_x + dx, core_y + dy, cell));

        if movement.is_none() && plant_direction.is_some() {
            let fallback_offsets = match (dx, dy) {
                (step_x, 0) if step_x != 0 => [(step_x, -1), (step_x, 1), (0, -1), (0, 1)],
                (0, step_y) if step_y != 0 => [(-1, step_y), (1, step_y), (-1, 0), (1, 0)],
                (step_x, step_y) if step_x != 0 && step_y != 0 => [
                    (step_x, 0),
                    (0, step_y),
                    (step_x, -step_y),
                    (-step_x, step_y),
                ],
                _ => [(0, 0), (0, 0), (0, 0), (0, 0)],
            };

            movement = fallback_offsets
                .iter()
                .copied()
                .find_map(|(offset_x, offset_y)| {
                    let candidate_x = core_x + offset_x;
                    let candidate_y = core_y + offset_y;
                    self.checked_cell(candidate_x, candidate_y)
                        .filter(|cell| can_enter(*cell))
                        .map(|cell| (candidate_x, candidate_y, cell))
                });
        }

        let next_cooldown = 8 + self.rng.gen_range(0..5) as u8;

        let (new_x, new_y, destination) = match movement {
            Some(movement) => movement,
            None => {
                let state = self.sheep.get_mut(&id).unwrap();
                state.direction = -direction;
                state.move_cooldown = next_cooldown;
                self.update_sheep_bodies(id, core_x, core_y);
                return;
            }
        };

        if new_x != core_x {
            direction = (new_x - core_x) as i8;
        }

        if !can_enter(destination) {
            let state = self.sheep.get_mut(&id).unwrap();
            state.direction = -direction;
            state.move_cooldown = next_cooldown;
            self.update_sheep_bodies(id, core_x, core_y);
            return;
        }

        let destination_is_body = destination.species == Species::Sheep && destination.ra == id;
        let replacement_body = if destination_is_body {
            Some((new_x, new_y))
        } else {
            self.nearest_sheep_body(id, core_x, core_y)
        };
        if let Some((body_x, body_y)) = replacement_body {
            if (body_x, body_y) != (new_x, new_y) {
                self.set_checked_cell(body_x, body_y, EMPTY_CELL);
            }
            self.set_checked_cell(
                core_x,
                core_y,
                Cell {
                    species: Species::Sheep,
                    ra: id,
                    rb: SHEEP_BODY,
                    clock: 0,
                },
            );
        } else {
            self.set_checked_cell(core_x, core_y, EMPTY_CELL);
        }
        self.set_checked_cell(
            new_x,
            new_y,
            Cell {
                species: Species::Sheep,
                ra: id,
                rb: SHEEP_CORE,
                clock: 0,
            },
        );

        let state = self.sheep.get_mut(&id).unwrap();
        state.core_x = new_x;
        state.core_y = new_y;
        state.direction = direction;
        state.move_cooldown = next_cooldown;
        self.update_sheep_bodies(id, new_x, new_y);
    }

    fn update_sheep_bodies(&mut self, id: u8, core_x: i32, core_y: i32) {
        let mut body_positions = Vec::new();
        for body_x in 0..self.width {
            for body_y in 0..self.height {
                let cell = self.cells[self.get_index(body_x, body_y)];
                if cell.species == Species::Sheep
                    && cell.ra == id
                    && cell.rb & SHEEP_ROLE_MASK == SHEEP_BODY
                {
                    body_positions.push((body_x, body_y));
                }
            }
        }

        for (body_x, body_y) in body_positions {
            let body_index = self.get_index(body_x, body_y);
            let body = self.cells[body_index];
            if body.species != Species::Sheep
                || body.ra != id
                || body.rb & SHEEP_ROLE_MASK != SHEEP_BODY
            {
                continue;
            }

            let distance = (body_x - core_x).abs().max((body_y - core_y).abs());
            if distance <= 2 {
                self.cells[body_index].rb &= SHEEP_ROLE_MASK;
                continue;
            }

            let new_x = body_x + (core_x - body_x).signum();
            let new_y = body_y + (core_y - body_y).signum();
            let destination_index = self.checked_index(new_x, new_y);
            let can_move = destination_index
                .map(|index| self.cells[index].species == Species::Empty)
                .unwrap_or(false);
            let remaining_distance = (new_x - core_x).abs().max((new_y - core_y).abs());
            let remains_far = if can_move {
                remaining_distance > 4
            } else {
                distance > 4
            };
            let stranded_count = if remains_far {
                (body.rb >> 1).saturating_add(1)
            } else {
                0
            };

            if stranded_count >= SHEEP_STRAY_LIMIT {
                self.set_checked_cell(body_x, body_y, EMPTY_CELL);
                if let Some(state) = self.sheep.get_mut(&id) {
                    state.size = state.size.saturating_sub(1);
                }
                continue;
            }

            let mut updated_body = body;
            updated_body.rb = (stranded_count << 1) | (body.rb & SHEEP_ROLE_MASK);
            if can_move {
                self.set_checked_cell(body_x, body_y, EMPTY_CELL);
                self.set_checked_cell(new_x, new_y, updated_body);
            } else {
                self.set_checked_cell(body_x, body_y, updated_body);
            }
        }
    }

    fn nearest_sheep_body(&self, id: u8, x: i32, y: i32) -> Option<(i32, i32)> {
        let mut nearest = None;
        for cell_x in 0..self.width {
            for cell_y in 0..self.height {
                let cell = self.cells[self.get_index(cell_x, cell_y)];
                if cell.species != Species::Sheep
                    || cell.ra != id
                    || cell.rb & SHEEP_ROLE_MASK != SHEEP_BODY
                {
                    continue;
                }

                let distance = (cell_x - x).abs().max((cell_y - y).abs());
                if nearest
                    .map(|(_, _, nearest_distance)| distance < nearest_distance)
                    .unwrap_or(true)
                {
                    nearest = Some((cell_x, cell_y, distance));
                }
            }
        }
        nearest.map(|(cell_x, cell_y, _)| (cell_x, cell_y))
    }

    fn get_index(&self, x: i32, y: i32) -> usize {
        (x * self.height + y) as usize
    }

    fn get_cell(&self, x: i32, y: i32) -> Cell {
        let i = self.get_index(x, y);
        return self.cells[i];
    }

    fn get_wind(&self, x: i32, y: i32) -> Wind {
        let i = self.get_index(x, y);
        return self.winds[i];
    }

    fn blow_wind(cell: Cell, wind: Wind, mut api: SandApi) {
        if cell.clock.wrapping_sub(api.universe.generation) == 1 {
            return;
        }
        if cell.species == Species::Empty {
            return;
        }
        if cell.species == Species::Sheep {
            return;
        }
        let mut dx = 0;
        let mut dy = 0;

        let threshold = match cell.species {
            Species::Empty => 500,
            Species::Wall => 500,
            Species::Cloner => 500,

            Species::Stone => 70,
            Species::Wood => 70,
            Species::Sheep => 70,
            Species::Carcass => 70,

            Species::Plant => 60,
            Species::Lava => 60,
            Species::Ice => 60,

            Species::Fungus => 54,

            Species::Oil => 50,

            // Intentionally left out and covered by the default case
            // Species::Water => 40,
            // Species::Acid => 40,
            Species::Seed => 35,

            Species::Sand => 30,
            Species::Mite => 30,
            Species::Rocket => 30,

            Species::Dust => 10,
            Species::Fire => 5,
            Species::Gas => 5,
            /*
             Some hacked species values exist outside of the enum values.
             Making sure the default case is emitted allows "BELP" to have a defined wind threshold.
             Originally, threshold was a hardcoded value, so this preserves that original glitch behavior.
             See: https://sandspiel.club/#eMlYGC52XIto0NM1WjaJ
            */
            _ => 40,
        };

        let wx = (wind.dy as i32) - 126;
        let wy = (wind.dx as i32) - 126;

        if wx > threshold {
            dx = 1;
        }
        if wy > threshold {
            dy = 1;
        }
        if wx < -threshold {
            dx = -1;
        }
        if wy < -threshold {
            dy = -1;
        }
        if (dx != 0 || dy != 0) && api.get(dx, dy).species == Species::Empty {
            api.set(0, 0, EMPTY_CELL);
            if dy == -1
                && api.get(dx, -2).species == Species::Empty
                && (cell.species == Species::Sand
                    || cell.species == Species::Water
                    || cell.species == Species::Lava
                    || cell.species == Species::Acid
                    || cell.species == Species::Mite
                    || cell.species == Species::Dust
                    || cell.species == Species::Oil
                    || cell.species == Species::Rocket)
            {
                dy = -2;
            }
            api.set(dx, dy, cell);
            return;
        }
    }
    fn update_cell(cell: Cell, api: SandApi) {
        if cell.clock.wrapping_sub(api.universe.generation) == 1 {
            return;
        }

        cell.update(api);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spawn_sheep_creates_one_core_and_up_to_four_body_pixels() {
        let mut universe = Universe::new(20, 20);
        assert!(universe.spawn_sheep(10, 10));
        let cells: Vec<Cell> = universe
            .cells
            .iter()
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

    #[test]
    fn spawn_sheep_uses_only_free_in_bounds_cells_at_an_edge() {
        let mut universe = Universe::new(2, 2);

        assert!(universe.spawn_sheep(0, 0));

        let sheep_cells: Vec<Cell> = universe
            .cells
            .iter()
            .copied()
            .filter(|cell| cell.species == Species::Sheep)
            .collect();
        assert_eq!(sheep_cells.len(), 2);
        assert_eq!(universe.cells[universe.get_index(0, 0)].rb, SHEEP_CORE);
        assert_eq!(universe.cells[universe.get_index(1, 0)].rb, SHEEP_BODY);
        assert_eq!(universe.sheep.get(&1).unwrap().size, 2);
    }

    #[test]
    fn spawn_sheep_allocates_unique_sequential_ids() {
        let mut universe = Universe::new(20, 20);

        assert!(universe.spawn_sheep(5, 5));
        assert!(universe.spawn_sheep(15, 15));

        assert!(universe.sheep.contains_key(&1));
        assert!(universe.sheep.contains_key(&2));
        assert_eq!(universe.next_sheep_id, 3);
    }

    #[test]
    fn spawn_sheep_initializes_newborn_state() {
        let mut universe = Universe::new(20, 20);

        assert!(universe.spawn_sheep(10, 10));

        assert_eq!(
            universe.sheep.get(&1),
            Some(&SheepState {
                core_x: 10,
                core_y: 10,
                energy: 100,
                size: 5,
                direction: 1,
                move_cooldown: 8,
                eat_cooldown: 20,
                mature_steps: None,
                starvation_steps: 0,
                submerged_steps: 0,
                dying_steps: None,
            })
        );
    }

    #[test]
    fn rebuild_groups_sheep_pixels_by_id() {
        let mut universe = Universe::new(20, 20);
        let core_index = universe.get_index(5, 5);
        let body_index = universe.get_index(6, 5);
        universe.cells[core_index] = Cell {
            species: Species::Sheep,
            ra: 7,
            rb: SHEEP_CORE,
            clock: 0,
        };
        universe.cells[body_index] = Cell {
            species: Species::Sheep,
            ra: 7,
            rb: SHEEP_BODY,
            clock: 0,
        };

        universe.rebuild_sheep_states();

        let sheep = universe.sheep.get(&7).unwrap();
        assert_eq!((sheep.core_x, sheep.core_y), (5, 5));
        assert_eq!(sheep.size, 2);
    }

    #[test]
    fn rebuild_marks_the_first_pixel_as_core_when_none_is_marked() {
        let mut universe = Universe::new(20, 20);
        let first_index = universe.get_index(3, 4);
        let second_index = universe.get_index(3, 5);
        universe.cells[first_index] = Cell {
            species: Species::Sheep,
            ra: 7,
            rb: SHEEP_BODY,
            clock: 0,
        };
        universe.cells[second_index] = Cell {
            species: Species::Sheep,
            ra: 7,
            rb: SHEEP_BODY,
            clock: 0,
        };

        universe.rebuild_sheep_states();

        let sheep = universe.sheep.get(&7).unwrap();
        assert_eq!((sheep.core_x, sheep.core_y), (3, 4));
        assert_eq!(universe.cells[first_index].rb, SHEEP_CORE);
    }

    #[test]
    fn pop_undo_rebuilds_sheep_state_from_restored_cells() {
        let mut universe = Universe::new(20, 20);
        universe.push_undo();
        let sheep_index = universe.get_index(10, 10);
        universe.cells[sheep_index] = Cell {
            species: Species::Sheep,
            ra: 7,
            rb: SHEEP_CORE,
            clock: 0,
        };
        universe.rebuild_sheep_states();
        assert!(universe.sheep.contains_key(&7));

        universe.pop_undo();

        assert!(universe.sheep.is_empty());
    }

    #[test]
    fn reset_clears_sheep_state_and_resets_the_next_id() {
        let mut universe = Universe::new(20, 20);
        let sheep_index = universe.get_index(10, 10);
        universe.cells[sheep_index] = Cell {
            species: Species::Sheep,
            ra: 7,
            rb: SHEEP_CORE,
            clock: 0,
        };
        universe.rebuild_sheep_states();
        universe.next_sheep_id = 7;

        universe.reset();

        assert!(universe.sheep.is_empty());
        assert_eq!(universe.next_sheep_id, 1);
    }

    #[test]
    fn rebuild_uses_default_state_and_caps_size_at_ten() {
        let mut universe = Universe::new(20, 20);
        for x in 0..11 {
            let index = universe.get_index(x, 0);
            universe.cells[index] = Cell {
                species: Species::Sheep,
                ra: 7,
                rb: SHEEP_BODY,
                clock: 0,
            };
        }

        universe.rebuild_sheep_states();

        assert_eq!(
            universe.sheep.get(&7),
            Some(&SheepState {
                core_x: 0,
                core_y: 0,
                energy: 100,
                size: 10,
                direction: 1,
                move_cooldown: 8,
                eat_cooldown: 20,
                mature_steps: Some(800),
                starvation_steps: 0,
                submerged_steps: 0,
                dying_steps: None,
            })
        );
    }

    #[test]
    fn sheep_scan_points_toward_the_nearest_plant() {
        let mut universe = Universe::new(30, 30);
        let farther_plant = universe.get_index(15, 10);
        let nearer_plant = universe.get_index(8, 11);
        universe.cells[farther_plant].species = Species::Plant;
        universe.cells[nearer_plant].species = Species::Plant;

        assert_eq!(universe.nearest_plant_direction(10, 10, 5), Some((-1, 1)));
    }

    #[test]
    fn sheep_scan_stays_in_bounds_and_respects_radius() {
        let mut universe = Universe::new(6, 6);
        let outside_radius = universe.get_index(5, 5);
        universe.cells[outside_radius].species = Species::Plant;

        assert_eq!(universe.nearest_plant_direction(0, 0, 5), None);
    }

    #[test]
    fn sheep_core_moves_on_cooldown_and_keeps_its_id() {
        let mut universe = Universe::new(30, 30);
        assert!(universe.spawn_sheep(10, 10));
        let id = 1;
        let plant_index = universe.get_index(15, 10);
        universe.cells[plant_index].species = Species::Plant;

        for _ in 0..7 {
            universe.update_sheep_core(id, 10, 10);
        }
        assert_eq!(
            (universe.sheep[&id].core_x, universe.sheep[&id].core_y),
            (10, 10)
        );

        universe.update_sheep_core(id, 10, 10);

        let state = &universe.sheep[&id];
        assert_eq!((state.core_x, state.core_y), (11, 10));
        assert!((8..=12).contains(&state.move_cooldown));
        let old_core = universe.cells[universe.get_index(10, 10)];
        let new_core = universe.cells[universe.get_index(11, 10)];
        assert_eq!(
            (old_core.species, old_core.ra, old_core.rb & 1),
            (Species::Sheep, id, SHEEP_BODY)
        );
        assert_eq!(
            (new_core.species, new_core.ra, new_core.rb & 1),
            (Species::Sheep, id, SHEEP_CORE)
        );
    }

    #[test]
    fn sheep_core_rejects_hazardous_destinations() {
        let mut universe = Universe::new(30, 30);
        assert!(universe.spawn_sheep(10, 10));
        let id = 1;
        let destination = universe.get_index(11, 10);
        let plant_index = universe.get_index(15, 10);
        universe.cells[destination] = Cell {
            species: Species::Water,
            ra: 0,
            rb: 0,
            clock: 0,
        };
        for (x, y) in [
            (9, 9),
            (9, 10),
            (9, 11),
            (10, 9),
            (10, 11),
            (11, 9),
            (11, 11),
        ] {
            let index = universe.get_index(x, y);
            universe.cells[index].species = Species::Wall;
        }
        universe.cells[plant_index].species = Species::Plant;

        for _ in 0..8 {
            universe.update_sheep_core(id, 10, 10);
        }

        let state = &universe.sheep[&id];
        assert_eq!((state.core_x, state.core_y), (10, 10));
        assert_eq!(state.direction, -1);
        assert_eq!(universe.cells[destination].species, Species::Water);
    }

    #[test]
    fn sheep_changes_course_when_plant_path_is_blocked() {
        let mut universe = Universe::new(30, 30);
        assert!(universe.spawn_sheep(10, 10));
        let blocker_index = universe.get_index(11, 10);
        let plant_index = universe.get_index(15, 10);
        universe.cells[blocker_index].species = Species::Wall;
        universe.cells[plant_index].species = Species::Plant;
        universe.sheep.get_mut(&1).unwrap().move_cooldown = 1;

        universe.update_sheep_core(1, 10, 10);

        let state = &universe.sheep[&1];
        assert_ne!((state.core_x, state.core_y), (10, 10));
        assert_ne!((state.core_x, state.core_y), (11, 10));
        assert_eq!(universe.cells[blocker_index].species, Species::Wall);
    }

    #[test]
    fn sheep_core_moves_before_body_cohesion() {
        let mut universe = Universe::new(30, 30);
        let core_index = universe.get_index(10, 10);
        let near_body_index = universe.get_index(9, 10);
        let distant_body_index = universe.get_index(10, 13);
        let plant_index = universe.get_index(15, 10);
        universe.cells[core_index] = Cell {
            species: Species::Sheep,
            ra: 7,
            rb: SHEEP_CORE,
            clock: 0,
        };
        universe.cells[near_body_index] = Cell {
            species: Species::Sheep,
            ra: 7,
            rb: SHEEP_BODY,
            clock: 0,
        };
        universe.cells[distant_body_index] = Cell {
            species: Species::Sheep,
            ra: 7,
            rb: SHEEP_BODY,
            clock: 0,
        };
        universe.cells[plant_index].species = Species::Plant;
        universe.rebuild_sheep_states();
        universe.sheep.get_mut(&7).unwrap().move_cooldown = 1;

        universe.update_sheep_core(7, 10, 10);

        let cohesive_body = universe.cells[universe.get_index(11, 12)];
        assert_eq!(
            (cohesive_body.species, cohesive_body.ra, cohesive_body.rb),
            (Species::Sheep, 7, SHEEP_BODY)
        );
        assert_eq!(cohesive_body.clock, universe.generation.wrapping_add(1));
        assert_eq!(universe.cells[distant_body_index].species, Species::Empty);
    }

    #[test]
    fn distant_sheep_body_moves_one_cell_toward_the_core() {
        let mut universe = Universe::new(30, 30);
        let core_index = universe.get_index(10, 10);
        let body_index = universe.get_index(15, 10);
        universe.cells[core_index] = Cell {
            species: Species::Sheep,
            ra: 7,
            rb: SHEEP_CORE,
            clock: 0,
        };
        universe.cells[body_index] = Cell {
            species: Species::Sheep,
            ra: 7,
            rb: SHEEP_BODY,
            clock: 0,
        };
        universe.rebuild_sheep_states();
        universe.sheep.get_mut(&7).unwrap().move_cooldown = 100;

        universe.update_sheep_core(7, 10, 10);

        assert_eq!(universe.cells[body_index].species, Species::Empty);
        let moved_body = universe.cells[universe.get_index(14, 10)];
        assert_eq!((moved_body.species, moved_body.ra), (Species::Sheep, 7));
        assert_eq!(moved_body.rb, SHEEP_BODY);
    }

    #[test]
    fn stranded_sheep_body_is_removed_after_thirty_updates() {
        let mut universe = Universe::new(30, 30);
        let core_index = universe.get_index(10, 10);
        let body_index = universe.get_index(15, 10);
        let blocker_index = universe.get_index(14, 10);
        universe.cells[core_index] = Cell {
            species: Species::Sheep,
            ra: 7,
            rb: SHEEP_CORE,
            clock: 0,
        };
        universe.cells[body_index] = Cell {
            species: Species::Sheep,
            ra: 7,
            rb: SHEEP_BODY,
            clock: 0,
        };
        universe.cells[blocker_index].species = Species::Wall;
        universe.rebuild_sheep_states();
        universe.sheep.get_mut(&7).unwrap().move_cooldown = 100;

        for _ in 0..29 {
            universe.update_sheep_core(7, 10, 10);
        }
        assert_eq!(universe.cells[body_index].rb >> 1, 29);

        universe.update_sheep_core(7, 10, 10);

        assert_eq!(universe.cells[body_index].species, Species::Empty);
        assert_eq!(universe.sheep[&7].size, 1);
    }

    #[test]
    fn sheep_movement_preserves_id_and_pixel_limit() {
        let mut universe = Universe::new(30, 30);
        assert!(universe.spawn_sheep(10, 10));
        let id = *universe.sheep.keys().next().unwrap();
        let plant_index = universe.get_index(15, 10);
        universe.cells[plant_index].species = Species::Plant;

        for _ in 0..120 {
            universe.tick();
        }

        let sheep_cells: Vec<Cell> = universe
            .cells
            .iter()
            .copied()
            .filter(|cell| cell.species == Species::Sheep && cell.ra == id)
            .collect();
        assert!(!sheep_cells.is_empty());
        assert!(sheep_cells.len() <= 10);
        assert!(sheep_cells.iter().all(|cell| cell.ra == id));
    }

    #[test]
    fn unobstructed_sheep_reaches_plant_adjacency_without_losing_pixels() {
        let mut universe = Universe::new(30, 30);
        assert!(universe.spawn_sheep(10, 10));
        let id = 1;
        let original_count = universe
            .cells
            .iter()
            .filter(|cell| cell.species == Species::Sheep && cell.ra == id)
            .count();
        let plant_index = universe.get_index(15, 10);
        universe.cells[plant_index].species = Species::Plant;

        for _ in 0..8 {
            let (core_x, core_y) = {
                let state = &mut universe.sheep.get_mut(&id).unwrap();
                state.move_cooldown = 1;
                (state.core_x, state.core_y)
            };
            universe.update_sheep_core(id, core_x, core_y);
        }

        let state = &universe.sheep[&id];
        assert_eq!((state.core_x, state.core_y), (14, 10));
        assert_eq!(universe.cells[plant_index].species, Species::Plant);
        let final_count = universe
            .cells
            .iter()
            .filter(|cell| cell.species == Species::Sheep && cell.ra == id)
            .count();
        assert_eq!(final_count, original_count);
    }

    #[test]
    fn foreign_sheep_body_is_neither_consumed_nor_retagged() {
        let mut universe = Universe::new(30, 30);
        assert!(universe.spawn_sheep(10, 10));
        let foreign_index = universe.get_index(11, 10);
        let plant_index = universe.get_index(15, 10);
        universe.cells[foreign_index] = Cell {
            species: Species::Sheep,
            ra: 2,
            rb: SHEEP_BODY,
            clock: 0,
        };
        universe.cells[plant_index].species = Species::Plant;
        universe.sheep.get_mut(&1).unwrap().move_cooldown = 1;

        universe.update_sheep_core(1, 10, 10);

        let foreign_body = universe.cells[foreign_index];
        assert_eq!(
            (foreign_body.species, foreign_body.ra, foreign_body.rb),
            (Species::Sheep, 2, SHEEP_BODY)
        );
    }

    #[test]
    fn live_sheep_cannot_be_displaced_by_wind() {
        let mut universe = Universe::new(30, 30);
        assert!(universe.spawn_sheep(10, 10));
        let core = universe.cells[universe.get_index(10, 10)];
        let original_downwind = universe.cells[universe.get_index(11, 10)];

        Universe::blow_wind(
            core,
            Wind {
                dx: 126,
                dy: 255,
                pressure: 0,
                density: 0,
            },
            SandApi {
                universe: &mut universe,
                x: 10,
                y: 10,
            },
        );

        let original_position = universe.cells[universe.get_index(10, 10)];
        let downwind_position = universe.cells[universe.get_index(11, 10)];
        assert_eq!(original_position, core);
        assert_eq!(downwind_position, original_downwind);
        assert_eq!(
            (universe.sheep[&1].core_x, universe.sheep[&1].core_y),
            (10, 10)
        );
    }

    #[test]
    fn sheep_eats_one_adjacent_plant_and_grows() {
        let mut universe = Universe::new(30, 30);
        assert!(universe.spawn_sheep(10, 10));
        let plant_index = universe.get_index(12, 10);
        universe.cells[plant_index].species = Species::Plant;
        universe.sheep.get_mut(&1).unwrap().eat_cooldown = 1;

        universe.update_sheep_core(1, 10, 10);

        let eaten = universe.cells[plant_index];
        assert_eq!(
            (eaten.species, eaten.ra, eaten.rb),
            (Species::Sheep, 1, SHEEP_BODY)
        );
        assert_eq!(universe.sheep[&1].size, 6);
        assert!((20..=30).contains(&universe.sheep[&1].eat_cooldown));
    }

    #[test]
    fn sheep_starts_mature_lifespan_at_ten_pixels() {
        let mut universe = Universe::new(30, 30);
        assert!(universe.spawn_sheep(10, 10));
        universe.sheep.get_mut(&1).unwrap().size = 9;
        universe.sheep.get_mut(&1).unwrap().eat_cooldown = 1;
        let plant_index = universe.get_index(12, 10);
        universe.cells[plant_index].species = Species::Plant;

        universe.update_sheep_core(1, 10, 10);

        assert_eq!(universe.sheep[&1].size, 10);
        assert_eq!(universe.sheep[&1].mature_steps, Some(800));
    }

    #[test]
    fn mature_sheep_dies_gradually_into_carcass() {
        let mut universe = Universe::new(30, 30);
        assert!(universe.spawn_sheep(10, 10));
        universe.sheep.get_mut(&1).unwrap().mature_steps = Some(1);

        universe.update_sheep_core(1, 10, 10);
        assert!(universe.sheep.contains_key(&1));
        assert_eq!(
            universe
                .cells
                .iter()
                .filter(|cell| cell.species == Species::Carcass)
                .count(),
            1
        );

        for _ in 0..10 {
            if !universe.sheep.contains_key(&1) {
                break;
            }
            let (x, y) = {
                let state = &universe.sheep[&1];
                (state.core_x, state.core_y)
            };
            universe.update_sheep_core(1, x, y);
        }
        assert!(!universe.sheep.contains_key(&1));
        assert!(!universe
            .cells
            .iter()
            .any(|cell| cell.species == Species::Sheep && cell.ra == 1));
    }

    #[test]
    fn fire_contact_starts_sheep_death() {
        let mut universe = Universe::new(30, 30);
        assert!(universe.spawn_sheep(10, 10));
        let fire_index = universe.get_index(12, 10);
        universe.cells[fire_index].species = Species::Fire;

        universe.update_sheep_core(1, 10, 10);

        assert!(universe
            .cells
            .iter()
            .any(|cell| cell.species == Species::Carcass));
        assert_eq!(universe.sheep[&1].dying_steps, Some(1));
    }
}
