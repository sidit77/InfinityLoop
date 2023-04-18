mod map;
mod tiles;
mod generator;

use fastrand::Rng;
use hashbrown::HashSet;
use serde::{Serialize, Deserialize};
use generator::PossibilityMap;
use crate::HexPos;
use crate::util::Update;

pub use tiles::*;
pub use map::HexMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(into = "WorldSave")]
#[serde(from = "WorldSave")]
pub struct World {
    seed: u64,
    elements: HexMap<TileConfig>,
    incomplete: HashSet<HexPos>
}

impl World {

    pub fn new(seed: u64) -> Self {

        //let now = Instant::now();

        let mut wfc = PossibilityMap::new(2, seed);

        'outer: loop {
            //println!("Attempt {}", i + 1);
            assert!(wfc.clear().is_ok());

            loop {
                match wfc.lowest_entropy() {
                    None => break 'outer,
                    Some(index) => {
                        if wfc.collapse(index).is_err() {
                            continue 'outer;
                        }
                    }
                }
            }
        }

        let elements = wfc.into();

        //println!("Time: {}ms", now.elapsed().as_millis());

        Self {
            seed,
            elements,
            incomplete: HashSet::new()
        }
    }

    pub fn tiles(&self) -> &HexMap<TileConfig> {
        &self.elements
    }

    pub fn scramble(&mut self, force_rotation: bool) {
        let rng = Rng::with_seed(self.seed());
        while {
            for tile in self.elements.values_mut() {
                *tile = if force_rotation {
                    tile.rotate_by(rng.u8(1..6))
                } else {
                    tile.with_rotation(rng.u8(..6))
                };
            }
            self.incomplete.clear();
            for pos in self.elements.keys() {
                if !self.is_tile_complete(pos) {
                    self.incomplete.insert(pos);
                }
            }
            self.incomplete.is_empty()
        } {}
    }

    fn is_tile_complete(&self, pos: HexPos) -> bool {
        match self.elements.get(pos) {
            None => true,
            Some(tile) => !pos
                .neighbors()
                .map(|npos| self.elements.get(npos).unwrap_or(&TileConfig::Empty))
                .enumerate()
                .any(|(i, n)| tile.endings()[i] != n.endings()[(i + 3) % 6])
        }
    }

    pub fn try_rotate(&mut self, pos: HexPos) -> bool {
        let updated = self.elements
            .get_mut(pos)
            .map(|t|t.update(t.rotate_by(1)))
            .unwrap_or(false);

        if updated {
            for pos in HexPos::spiral_iter(pos, 1) {
                match self.is_tile_complete(pos) {
                    true => self.incomplete.remove(&pos),
                    false => self.incomplete.insert(pos)
                };
            }
        }
        updated
    }

    pub fn is_completed(&self) -> bool {
        self.incomplete.is_empty()
    }

    pub fn seed(&self) -> u64 {
        self.seed
    }

    pub fn iter(&self) -> impl Iterator<Item=(HexPos, TileConfig)> + '_ {
        self.elements.keys().map(move |k|(k, self.elements[k]))
    }

}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WorldSave {
    seed: u64,
    rotations: Vec<u8>
}

impl From<World> for WorldSave {
    fn from(world: World) -> Self {
        Self {
            seed: world.seed,
            rotations: world.elements.values().map(|v| match *v {
                TileConfig::Empty => 0,
                TileConfig::Tile(_, r) => r
            }).collect()
        }
    }
}

impl From<WorldSave> for World {
    fn from(save: WorldSave) -> Self {
        let mut world = World::new(save.seed);
        if save.rotations.len() == world.elements.len() {
            for (tc, r) in world.elements.values_mut().zip(save.rotations.iter()) {
                *tc = tc.with_rotation(*r);
            }
            world.incomplete.clear();
            for pos in world.elements.keys() {
                if !world.is_tile_complete(pos) {
                    world.incomplete.insert(pos);
                }
            }
        } else {
            log::warn!("Number of rotations in save doesn't match the number of tiles in this level");
        }
        world
    }
}