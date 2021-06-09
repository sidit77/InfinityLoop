use std::ops::Range;
use glam::Vec2;
use crate::meshes::{MODEL1, MODEL2, MODEL6, MODEL3, MODEL4, MODEL7, MODEL5};
use std::collections::VecDeque;

const SIN_FRAC_PI_6: f32 = 0.5;
const COS_FRAC_PI_6: f32 = 0.86602540378;

#[derive(Debug, Copy, Clone)]
pub enum TileType {
    Tile0,
    Tile01,
    Tile02,
    Tile03,
    Tile012,
    Tile024,
    Tile0134,
}

impl TileType {
    pub fn model(&self) -> Range<i32> {
        match self {
            TileType::Tile0 => MODEL1,
            TileType::Tile01 => MODEL2,
            TileType::Tile02 => MODEL6,
            TileType::Tile03 => MODEL3,
            TileType::Tile012 => MODEL4,
            TileType::Tile024 => MODEL7,
            TileType::Tile0134 => MODEL5
        }
    }
    pub fn endings(&self) -> [bool; 6] {
        match self {
            TileType::Tile0 => [true, false, false, false, false, false],
            TileType::Tile01 => [true, true, false, false, false, false],
            TileType::Tile02 => [true, false, true, false, false, false],
            TileType::Tile03 => [true, false, false, true, false, false],
            TileType::Tile012 => [true, true, true, false, false, false],
            TileType::Tile024 => [true, false, true, false, true, false],
            TileType::Tile0134 => [true, true, false, true, true, false],
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct WorldElement {
    pub tile_type: TileType,
    pub rotation: u8
}

#[derive(Debug)]
pub struct World {
    rows: u32,
    width: u32,
    elements: Vec<Option<WorldElement>>
}


impl World {

    pub fn from_seed(seed: u64) -> Self{
        let mut wfc = WaveCollapseWorld::new(9, 5, seed);
        wfc.prepare();
        loop
        {
            match wfc.lowest_entropy(){
                None => break,
                Some(index) => {
                    wfc.collapse(index);
                    assert!(wfc.valid())
                }
            }

        }

        wfc.into()

    }

    pub fn indices(&self) -> Range<usize> {
        0..self.elements.len()
    }

    pub fn get_position(&self, index: usize) -> Vec2{
        let (x, y) = self.get_xy(index);
        let offset = -((y % 2) as f32) * COS_FRAC_PI_6;
        Vec2::new(
            (-0.5 * self.width as f32 - 1.0) + (2.0 * COS_FRAC_PI_6) * x as f32 + offset,
            (-0.5 * self.rows  as f32 - 1.0) + (1.0 + SIN_FRAC_PI_6) * y as f32
        )
    }
    pub fn get_size(&self) -> (f32, f32) {
        ((2.0 * COS_FRAC_PI_6) * self.width as f32, (1.0 + SIN_FRAC_PI_6) * self.rows as f32)
    }
    pub fn get_element(&mut self, index: usize) -> &mut Option<WorldElement> {
        &mut self.elements[index]
    }
}


#[derive(Debug)]
struct WaveCollapseWorld {
    table: Vec<Option<WorldElement>>,
    adjacency_lists: Vec<[IndexSet; 6]>,
    rng: fastrand::Rng,
    propagation_stack: VecDeque<usize>,
    rows: u32,
    width: u32,
    elements: Vec<IndexSet>
}

impl WaveCollapseWorld {

    fn new(rows: u32, width: u32, seed: u64) -> Self {
        let rng = fastrand::Rng::with_seed(seed);
        let table =  build_table();
        let adjacency_lists = build_adjacency_lists(&table);
        Self {
            table,
            adjacency_lists,
            rng,
            propagation_stack: VecDeque::new(),
            rows,
            width,
            elements: Vec::new()
        }
    }

    fn prepare(&mut self) {
        self.elements.clear();
        for _ in 0..(self.rows * self.width + (self.rows / 2)) {
            let mut v = IndexSet::empty();
            for i in 0..self.table.len() {
                v = v.insert(i as u8);
            }
            self.elements.push(v);
        }
    }

    fn valid(&self) -> bool {
        !self.elements.iter().any(|x|x.is_empty())
    }

    fn lowest_entropy(&self) -> Option<usize> {
        let mut set = Vec::new();
        let mut min = usize::MAX;
        for i in 0..self.elements.len() {
            let l = self.elements[i].len();
            if l > 1 {
                if l < min {
                    set.clear();
                    min = l;
                }
                if l == min {
                    set.push(i);
                }
            }
        }
        if set.len() == 0 {
            None
        } else {
            Some(set[self.rng.usize(0..set.len())])
        }
    }

    fn collapse(&mut self, index: usize) {
        let selected = self.elements[index].iter().nth(self.rng.usize(0..self.elements[index].len())).unwrap();
        self.elements[index] = IndexSet::singleton(selected);
        self.propagation_stack.push_back(index);
        self.propagate();
    }

    fn propagate(&mut self) {
        loop {
            match self.propagation_stack.pop_front() {
                None => break,
                Some(index) => {
                    //let id = self.elements[index].iter().nth(0).unwrap();
                    for (i, n) in self.get_neighbors(index).iter().enumerate() {
                        if let Some(n) = *n {
                            let adl = self
                                .elements[index]
                                .iter()
                                .map(|x|self.adjacency_lists[x as usize][i])
                                .fold(IndexSet::empty(), | acc, x| acc.union(x));
                            let prl = self.elements[n].inter(adl);
                            if prl != self.elements[n] {
                                self.elements[n] = prl;
                                self.propagation_stack.push_back(n);
                            }
                        }
                    }

                }
            }
        }
    }

}

impl Into<World> for WaveCollapseWorld {
    fn into(self) -> World {
        World {
            rows: self.rows,
            width: self.width,
            elements: self.elements
                .iter()
                .map(|x|x.iter().nth(0).unwrap())
                .map(|x|self.table[x as usize])
                .collect()
        }
    }
}

trait HexWorld {
    fn width(&self) -> u32;
    fn rows(&self) -> u32;
    fn get_index(&self, x: i32, y: i32) -> Option<usize> {
        if x < 0 || y < 0 {
            return None;
        }
        let (x,y) = (x as u32, y as u32);
        if y < self.rows() && x < (self.width() + (y % 2)) {
            Some((y * self.width() + (y / 2) + x) as usize)
        } else {
            None
        }
    }
    fn get_xy(&self, index: usize) -> (i32, i32) {
        let div = index as i32 / (2 * self.width() as i32 + 1);
        let rem = index as i32 % (2 * self.width() as i32 + 1);

        if rem < self.width() as i32 {
            (rem, 2 * div)
        } else {
            (rem - self.width() as i32, 2 * div + 1)
        }
    }
    fn get_neighbors(&self, index: usize) -> [Option<usize>; 6] {
        let (x, y) = self.get_xy(index);
        if y % 2 == 0 {
            [
                self.get_index(x + 1, y + 1),
                self.get_index(x + 1, y),
                self.get_index(x + 1, y - 1),
                self.get_index(x, y - 1),
                self.get_index(x - 1, y),
                self.get_index(x, y + 1)
            ]
        } else {
            [
                self.get_index(x, y + 1),
                self.get_index(x + 1, y),
                self.get_index(x, y - 1),
                self.get_index(x - 1, y - 1),
                self.get_index(x - 1, y),
                self.get_index(x - 1, y + 1)
            ]
        }
    }
}

impl HexWorld for WaveCollapseWorld {
    fn width(&self) -> u32 {
        self.width
    }
    fn rows(&self) -> u32 {
        self.rows
    }
}

impl HexWorld for World {
    fn width(&self) -> u32 {
        self.width
    }
    fn rows(&self) -> u32 {
        self.rows
    }
}

type IndexSet = smallbitset::Set64;

fn build_symmetries(_tile_type: TileType) -> Vec<u8> {
    //let endings = tile_type.endings();
    //let mut result = Vec::new();
    //for i in 0..endings.len() {
    //    if !result.iter().any(|j| {
    //        for x in 0..endings.len() {
    //            if endings[(x + i as usize) % endings.len()] != endings[(x + *j as usize) % endings.len()]{
    //                return false;
    //            }
    //        }
    //        true
    //    }) {
    //        result.push(i as u8);
    //    }
    //}
    //result
    (0..6).into_iter().collect()
}

fn build_table() -> Vec<Option<WorldElement>>{
    let tile_types = [
        TileType::Tile0,
        TileType::Tile01,
        TileType::Tile02,
        TileType::Tile03,
        TileType::Tile012,
        TileType::Tile024,
        TileType::Tile0134
    ];

    let mut result = Vec::new();

    for tile_type in &tile_types {
        let tile_type = *tile_type;
        for rotation in build_symmetries(tile_type) {
            result.push(Some(WorldElement {
                tile_type,
                rotation
            }))
        }
    }
    result.push(None);

    result
}

fn get_endings(elem: Option<WorldElement>) -> [bool; 6] {
    match elem {
        None => [false; 6],
        Some(elem) => {
            let mut result = [false; 6];
            let endings = elem.tile_type.endings();
            for i in 0..6 {
                result[(i + elem.rotation as usize) % result.len()] = endings[i];
            }
            result
        }
    }
}

fn build_adjacency_lists(table: &Vec<Option<WorldElement>>) -> Vec<[IndexSet; 6]>{

    assert!(table.len() <= IndexSet::full().len());

    let mut result = Vec::new();

    for i in 0..table.len() {
        let mut lists = [IndexSet::empty(); 6];

        for j in 0..lists.len() {

            for k in 0..table.len() {

                let elem1 = get_endings(table[i]);
                let elem2 = get_endings(table[k]);

                if elem1[j] == elem2[(j + 3) % elem2.len()]{
                    lists[j] = lists[j].insert(k as u8);
                }
            }

        }

        result.push(lists);
    }

    result
}

