mod map;
mod tiles;
mod generator;

use std::collections::HashSet;
use fastrand::Rng;
use map::HexMap;
use generator::PossibilityMap;
use crate::HexPos;
use crate::util::Update;

pub use tiles::*;


#[derive(Debug, Clone)]
pub struct World {
    seed: u64,
    elements: HexMap<TileConfig>,
    incomplete: HashSet<HexPos>
}

impl World {

    pub fn new(seed: u64) -> Self {

        //let now = Instant::now();

        let mut wfc = PossibilityMap::new(5, seed);

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

    pub fn scramble(&mut self) {
        let rng = Rng::with_seed(self.seed());
        for tile in self.elements.values_mut() {
            *tile = tile.with_rotation(rng.u8(..6));
        }
        self.incomplete.clear();
        for pos in self.elements.keys() {
            if self.is_tile_complete(pos) {
                self.incomplete.insert(pos);
            }
        }
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
        self.elements.keys().map(move |k|(k, *self.elements.get(k).unwrap()))
    }

}