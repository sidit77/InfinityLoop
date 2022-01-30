use std::collections::VecDeque;
use lazy_static::lazy_static;
use crate::world::tiles::{TileConfig, TileType};
use enum_iterator::IntoEnumIterator;
use fastrand::Rng;
use priority_queue::PriorityQueue;
use crate::HexPos;
use crate::world::map::HexMap;

type IndexSet = smallbitset::Set64;

pub struct PossibilityMap {
    map: HexMap<IndexSet>,
    propagation_queue: VecDeque<HexPos>,
    minimal_nodes: PriorityQueue<HexPos, usize>,
    rng: Rng
}

impl PossibilityMap {

    pub fn new(radius: i32, seed: u64) -> Self {
        Self {
            map: HexMap::new(radius),
            propagation_queue: VecDeque::new(),
            minimal_nodes: PriorityQueue::new(),
            rng: Rng::with_seed(seed)
        }
    }

    pub fn clear(&mut self) -> Result<(), ()>{
        self.propagation_queue.clear();
        self.minimal_nodes.clear();
        self.map.fill(*COMPLETE_SET);

        for pos in HexPos::ring_iter(self.map.center(), self.map.radius()) {
            let set = pos
                .neighbors()
                .enumerate()
                .map(|(d, n)| match self.map.contains(n) {
                    true => IndexSet::full(),
                    false => ADJACENCY_LISTS[*EMPTY_ELEMENT_INDEX][(d + 3) % 6]
                })
                .fold(IndexSet::full(), |acc, x| acc.inter(x));
            self.intersect(pos, set)?;
        }
        self.propagate()
    }

    fn intersect(&mut self, pos: HexPos, value: IndexSet) -> Result<(), ()> {
        let field = self.map.get_mut(pos).unwrap();
        let intersection = field.inter(value);
        if *field != intersection {
            match intersection.len() {
                0 => return Err(()),
                1 => {
                    self.minimal_nodes.remove(&pos);
                },
                i => {
                    self.minimal_nodes.push(pos, intersection.capacity() - i);
                }
            };
            *field = intersection;
            self.propagation_queue.push_back(pos);
        }
        Ok(())
    }

    pub fn lowest_entropy(&mut self) -> Option<HexPos> {
        self.minimal_nodes.pop().map(|(p, _)|p)
    }

    pub fn collapse(&mut self, pos: HexPos) -> Result<(), ()> {
        let elem = self.map.get(pos).unwrap();
        let selected = elem
            .iter()
            .nth(self.rng.usize(0..elem.len()))
            .unwrap();
        self.intersect(pos, IndexSet::singleton(selected))?;
        self.propagate()
    }

    pub fn propagate(&mut self) -> Result<(), ()>{
        loop {
            match self.propagation_queue.pop_front() {
                None => break Ok(()),
                Some(pos) => {
                    for (index, neighbor) in pos.neighbors().enumerate() {
                        if self.map.contains(neighbor) {
                            let adl = self.map.get(pos).unwrap()
                                .iter()
                                .map(|x| ADJACENCY_LISTS[x as usize][index])
                                .fold(IndexSet::empty(), |acc, x| acc.union(x));
                            self.intersect(neighbor, adl)?;
                        }
                    }
                }
            }
        }
    }

}

impl From<PossibilityMap> for HexMap<TileConfig> {

    fn from(map: PossibilityMap) -> Self {
        assert!(!map.map.values().any(|set| set.len() != 1));
        Self::from(map.map, |set| ELEMENT_TABLE[set.iter().next().unwrap() as usize])
    }

}

lazy_static! {
    static ref ELEMENT_TABLE: Vec<TileConfig> = {
        let mut result = Vec::new();
        for tile_type in TileType::into_enum_iter() {
            for rotation in 0..6 {
                result.push(TileConfig::Tile(tile_type, rotation));
            }
        }
        result.push(TileConfig::Empty);
        result
    };
    static ref EMPTY_ELEMENT_INDEX: usize = {
        ELEMENT_TABLE
            .iter()
            .position(|x| *x == TileConfig::Empty)
            .expect("Cannot find the empty element in table")
    };
    static ref COMPLETE_SET: IndexSet = {
        (0..ELEMENT_TABLE.len())
            .into_iter()
            .map(|x| IndexSet::singleton(x as u8))
            .fold(IndexSet::empty(), |acc, x| acc.union(x))
    };
    static ref ADJACENCY_LISTS: Vec<[IndexSet; 6]> = {
        assert!(ELEMENT_TABLE.len() <= IndexSet::full().len());
        let mut result = Vec::new();
        for i in 0..ELEMENT_TABLE.len() {
            let mut lists = [IndexSet::empty(); 6];
            for j in 0..lists.len() {
                for k in 0..ELEMENT_TABLE.len() {
                    let elem1 = ELEMENT_TABLE[i].endings();
                    let elem2 = ELEMENT_TABLE[k].endings();

                    if elem1[j] == elem2[(j + 3) % elem2.len()] {
                        lists[j] = lists[j].insert(k as u8);
                    }
                }
            }
            result.push(lists);
        }
        result
    };
}
