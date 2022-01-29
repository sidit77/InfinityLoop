use std::collections::VecDeque;
use lazy_static::lazy_static;
use crate::world::tiles::{TileConfig, TileType};
use enum_iterator::IntoEnumIterator;
use fastrand::Rng;
use crate::HexPos;
use crate::world::map::HexMap;

type IndexSet = smallbitset::Set64;

pub struct PossibilityMap {
    map: HexMap<IndexSet>,
    propagation_queue: VecDeque<HexPos>
}

impl PossibilityMap {

    pub fn new(radius: i32) -> Self {
        Self {
            map: HexMap::new(radius),
            propagation_queue: VecDeque::new()
        }
    }

    pub fn clear(&mut self) {
        for pos in self.map.keys() {
            let set = pos
                .neighbors()
                .enumerate()
                .map(|(d, n)| {
                    if !self.map.contains(n) {
                        ADJACENCY_LISTS[*EMPTY_ELEMENT_INDEX][(d + 3) % 6]
                    } else {
                        *COMPLETE_SET
                    }
                })
                .fold(IndexSet::full(), |acc, x| acc.inter(x));
            if set != *COMPLETE_SET {
                self.propagation_queue.push_back(pos);
            }
            self.map.set(pos, set);
        }
        self.propagate();
    }

    pub fn valid(&self) -> bool {
        !self.map.values().any(|x| x.is_empty())
    }

    pub fn lowest_entropy(&self, rng: &Rng) -> Option<HexPos> {
        let mut set = Vec::new();
        let mut min = usize::MAX;
        for pos in self.map.keys() {
            let l = self.map.get(pos).unwrap().len();
            if l > 1 {
                if l < min {
                    set.clear();
                    min = l;
                }
                if l == min {
                    set.push(pos);
                }
            }
        }
        if set.len() == 0 {
            None
        } else {
            Some(set[rng.usize(0..set.len())])
        }
    }

    pub fn collapse(&mut self, pos: HexPos, rng: &Rng) {
        let elem = self.map.get_mut(pos).unwrap();
        let selected = elem
            .iter()
            .nth(rng.usize(0..elem.len()))
            .unwrap();
        *elem =  IndexSet::singleton(selected);
        self.propagation_queue.push_back(pos);
        self.propagate();
    }

    pub fn propagate(&mut self) {
        loop {
            match self.propagation_queue.pop_front() {
                None => break,
                Some(pos) => {
                    //let id = self.elements[index].iter().nth(0).unwrap();
                    for (index, neighbor) in pos.neighbors().enumerate() {
                        if self.map.contains(neighbor) {
                            let adl = self.map.get(pos).unwrap()
                                .iter()
                                .map(|x| ADJACENCY_LISTS[x as usize][index])
                                .fold(IndexSet::empty(), |acc, x| acc.union(x));
                            let prl = self.map.get(neighbor).unwrap().inter(adl);
                            if prl != *self.map.get(neighbor).unwrap() {
                                self.map.set(neighbor, prl);
                                self.propagation_queue.push_back(neighbor);
                            }
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
        let mut result = HexMap::new(map.map.radius());
        for pos in result.keys() {
            let index = map.map.get(pos).unwrap().iter().next().unwrap() as usize;
            result.set(pos, ELEMENT_TABLE[index]);
        }
        result
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
