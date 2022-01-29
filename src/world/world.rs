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
        let rng = Rng::new();

        let mut wfc = PossibilityMap::new(6);

        'outer: for _ in 0..20 {
            wfc.clear();
            //console_log!("{:?}", wfc.elements.iter().map(|x|x.len()).collect::<Vec<_>>());
            //assert!(wfc.valid());
            if !wfc.valid() {
                continue 'outer;
            }

            loop {
                match wfc.lowest_entropy(&rng) {
                    None => break,
                    Some(index) => {
                        wfc.collapse(index, &rng);
                        if !wfc.valid() {
                            continue 'outer;
                        }
                    }
                }
            }
        }

        Self {
            seed,
            elements: wfc.into()
        }
    }

    pub fn iter(&self) -> impl Iterator<Item=(HexPos, TileConfig)> + '_ {
        self.elements.keys().map(move |k|(k, self.elements.get(k).unwrap().clone()))
    }

}