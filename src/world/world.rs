use fastrand::Rng;
use crate::HexPos;
use crate::world::generator::PossibilityMap;
use crate::world::map::HexMap;
use crate::world::tiles::TileConfig;


#[derive(Debug, Clone)]
pub struct World {
    seed: u64,
    elements: HexMap<TileConfig>
}

impl World {

    pub fn new(seed: u64) -> Self {

        //let now = Instant::now();

        let mut wfc = PossibilityMap::new(4, seed);

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
            elements
        }
    }

    pub fn scramble(&mut self) {
        let rng = Rng::with_seed(self.seed());
        for tile in self.elements.values_mut() {
            *tile = tile.with_rotation(rng.u8(..6));
        }
    }

    pub fn try_rotate(&mut self, pos: HexPos) -> bool {
        match self.elements.get_mut(pos) {
            None => false,
            Some(i) => {
                let rot = i.rotate_by(1);
                let result = *i != rot;
                *i = rot;
                result
            }
        }
    }

    pub fn seed(&self) -> u64 {
        self.seed
    }

    pub fn iter(&self) -> impl Iterator<Item=(HexPos, TileConfig)> + '_ {
        self.elements.keys().map(move |k|(k, self.elements.get(k).unwrap().clone()))
    }

}