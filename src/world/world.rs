use fastrand::Rng;
use instant::Instant;
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

        let now = Instant::now();

        let mut wfc = PossibilityMap::new(40);

        'outer: for i in 0..2000 {
            println!("Attempt {}", i + 1);
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

        println!("Time: {}ms", now.elapsed().as_millis());

        Self {
            seed,
            elements
        }
    }

    pub fn iter(&self) -> impl Iterator<Item=(HexPos, TileConfig)> + '_ {
        self.elements.keys().map(move |k|(k, self.elements.get(k).unwrap().clone()))
    }

}