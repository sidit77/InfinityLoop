use fastrand::Rng;
use crate::HexPos;
use crate::world::hex_map::HexMap;
use crate::world::tiles::TileConfig;


#[derive(Debug, Clone)]
pub struct World {
    seed: u64,
    elements: HexMap<TileConfig>
}

impl World {

    pub fn new(seed: u64) -> Self {
        let rng = Rng::with_seed(seed);
        let mut elements = HexMap::new(9);
        for key in elements.keys() {
            elements.set(key, TileConfig::random(&rng));
        }
        Self {
            seed,
            elements
        }
    }

    pub fn iter(&self) -> impl Iterator<Item=(HexPos, TileConfig)> + '_ {
        self.elements.keys().map(move |k|(k, self.elements.get(k).unwrap().clone()))
    }

}