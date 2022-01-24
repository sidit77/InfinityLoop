use crate::meshes::{MODEL1, MODEL2, MODEL3, MODEL4, MODEL5, MODEL6, MODEL7};
use enum_iterator::IntoEnumIterator;
use glam::Vec2;
use lazy_static::lazy_static;
use miniserde::{Serialize, Deserialize};
use std::collections::VecDeque;
use std::ops::Range;
use crate::types::Angle;

const SIN_FRAC_PI_6: f32 = 0.5;
const COS_FRAC_PI_6: f32 = 0.86602540378;

#[derive(Debug, Copy, Clone)]
pub struct BoundingBox {
    pub left: f32,
    pub right: f32,
    pub top: f32,
    pub bottom: f32,
}

impl BoundingBox {
    pub fn center(&self) -> Vec2 {
        Vec2::new((self.right + self.left) / 2.0, (self.top + self.bottom) / 2.0)
    }
    pub fn width(&self) -> f32 {
        self.right - self.left
    }
    pub fn height(&self) -> f32 {
        self.top - self.bottom
    }
}

#[derive(Debug, IntoEnumIterator, Copy, Clone, Eq, PartialEq)]
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
            TileType::Tile0134 => MODEL5,
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
pub enum WorldElement {
    Empty,
    Tile(usize, Angle),
}

impl From<TileConfig> for WorldElement {
    fn from(tc: TileConfig) -> Self {
        match tc {
            TileConfig::Empty => Self::Empty,
            TileConfig::Tile(_, _) => Self::Tile(tc.index(), tc.angle()),
        }
    }
}

impl WorldElement {
    fn tile_config_index(&self) -> usize {
        match self {
            WorldElement::Empty => *EMPTY_ELEMENT_INDEX,
            WorldElement::Tile(i, _) => *i,
        }
    }

    fn as_tile_config(&self) -> TileConfig {
        TileConfig::from(self.tile_config_index())
    }
}

#[derive(Debug)]
pub struct World {
    seed: u64,
    rows: u32,
    width: u32,
    elements: Vec<WorldElement>,
}

impl World {
    pub fn from_seed(seed: u64) -> Self {
        let mut wfc = WaveCollapseWorld::new(5, 8, seed);

        'outer: for _ in 0..20 {
            wfc.prepare();
            //console_log!("{:?}", wfc.elements.iter().map(|x|x.len()).collect::<Vec<_>>());
            //assert!(wfc.valid());
            if !wfc.valid() {
                continue 'outer;
            }

            loop {
                match wfc.lowest_entropy() {
                    None => break,
                    Some(index) => {
                        wfc.collapse(index);
                        if !wfc.valid() {
                            continue 'outer;
                        }
                    }
                }
            }
            return wfc.into();
        }

        unreachable!()
    }

    pub fn indices(&self) -> Range<usize> {
        0..self.elements.len()
    }

    pub fn get_position(&self, index: usize) -> Vec2 {
        let (x, y) = self.get_xy(index);
        let offset = -((y % 2) as f32) * COS_FRAC_PI_6;
        Vec2::new(
            (-0.5 * self.width as f32 - 1.0) + (2.0 * COS_FRAC_PI_6) * x as f32 + offset,
            (-0.5 * self.rows as f32 - 1.0) + (1.0 + SIN_FRAC_PI_6) * y as f32,
        )
    }
    pub fn get_bounding_box(&self) -> BoundingBox {
        let mut bb = BoundingBox {
            left: f32::INFINITY,
            right: f32::NEG_INFINITY,
            top: f32::NEG_INFINITY,
            bottom: f32::INFINITY
        };
        for i in self.indices() {
            let position = self.get_position(i);
            if let WorldElement::Tile(_, _) = self.elements[i] {
                bb.left = f32::min(bb.left, position.x);
                bb.right = f32::max(bb.right, position.x);
                bb.bottom = f32::min(bb.bottom, position.y);
                bb.top = f32::max(bb.top, position.y);
            }
        }
        bb.left -= COS_FRAC_PI_6;
        bb.right += COS_FRAC_PI_6;
        bb.bottom -= 1.0;
        bb.top += 1.0;
        bb
    }
    pub fn get_element(&mut self, index: usize) -> &mut WorldElement {
        &mut self.elements[index]
    }
    pub fn scramble(&mut self, rng: &fastrand::Rng) {
        for e in &mut self.elements {
            *e = WorldElement::from(e.as_tile_config().rotate_by(rng.u8(0..6)));
        }
    }
    pub fn seed(&self) -> u64 {
        self.seed
    }
    pub fn is_completed(&self) -> bool {
        for i in self.indices() {
            let local = self.elements[i].tile_config_index();
            for (j, n) in self.get_neighbors(i).iter().enumerate() {
                let n = n
                    .map(|k| self.elements[k].tile_config_index())
                    .unwrap_or(*EMPTY_ELEMENT_INDEX);
                if !ADJACENCY_LISTS[local][j].contains(n as u8) {
                    return false;
                }
            }
        }
        true
    }
}

#[derive(Debug)]
struct WaveCollapseWorld {
    seed: u64,
    rng: fastrand::Rng,
    propagation_stack: VecDeque<usize>,
    rows: u32,
    width: u32,
    elements: Vec<IndexSet>,
}

impl WaveCollapseWorld {
    fn new(rows: u32, width: u32, seed: u64) -> Self {
        let rng = fastrand::Rng::with_seed(seed);
        Self {
            seed,
            rng,
            propagation_stack: VecDeque::new(),
            rows,
            width,
            elements: Vec::new(),
        }
    }

    fn prepare(&mut self) {
        let empty_element = *EMPTY_ELEMENT_INDEX;
        self.elements.clear();
        let complete_set = (0..ELEMENT_TABLE.len())
            .into_iter()
            .map(|x| IndexSet::singleton(x as u8))
            .fold(IndexSet::empty(), |acc, x| acc.union(x));
        for i in 0..(self.rows * self.width + (self.rows / 2)) {
            let set = self
                .get_neighbors(i as usize)
                .iter()
                .enumerate()
                .map(|(d, n)| {
                    if n.is_none() {
                        ADJACENCY_LISTS[empty_element][(d + 3) % 6]
                    } else {
                        complete_set
                    }
                })
                .fold(IndexSet::full(), |acc, x| acc.inter(x));
            if set != complete_set {
                self.propagation_stack.push_back(i as usize);
            }
            self.elements.push(set);
        }
        self.propagate();
    }

    fn valid(&self) -> bool {
        !self.elements.iter().any(|x| x.is_empty())
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
        let selected = self.elements[index]
            .iter()
            .nth(self.rng.usize(0..self.elements[index].len()))
            .unwrap();
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
                            let adl = self.elements[index]
                                .iter()
                                .map(|x| ADJACENCY_LISTS[x as usize][i])
                                .fold(IndexSet::empty(), |acc, x| acc.union(x));
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
            seed: self.seed,
            rows: self.rows,
            width: self.width,
            elements: self
                .elements
                .iter()
                .map(|x| x.iter().nth(0).unwrap() as usize)
                .map(|x| TileConfig::from(x))
                .map(|x| x.into())
                .collect(),
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
        let (x, y) = (x as u32, y as u32);
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
                self.get_index(x, y + 1),
            ]
        } else {
            [
                self.get_index(x, y + 1),
                self.get_index(x + 1, y),
                self.get_index(x, y - 1),
                self.get_index(x - 1, y - 1),
                self.get_index(x - 1, y),
                self.get_index(x - 1, y + 1),
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

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct WorldSave {
    pub seed: u64,
    pub rotations: Vec<u8>,
}

impl From<&World> for WorldSave {
    fn from(world: &World) -> Self {
        Self {
            seed: world.seed,
            rotations: world
                .elements
                .iter()
                .map(|x| x.as_tile_config().rotation())
                .collect(),
        }
    }
}

impl From<WorldSave> for World {
    fn from(save: WorldSave) -> Self {
        let mut world = World::from_seed(save.seed);
        world.scramble(&fastrand::Rng::with_seed(save.seed));
        for (elem, rot) in world.elements.iter_mut().zip(save.rotations) {
            *elem = elem.as_tile_config().with_rotation(rot).into();
        }
        world
    }
}

//impl Display for WorldSave {
//    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//        write!(f, "{};{:?}", self.seed, self.rotations)
//    }
//}
//
//impl FromStr for WorldSave {
//    type Err = Box<dyn Error>;
//
//    fn from_str(s: &str) -> Result<Self, Self::Err> {
//        let (seed, rotations) = s.split_once(';').ok_or("expected ;")?;
//        let seed = seed.parse()?;
//        let rotations = rotations
//            .strip_prefix('[')
//            .and_then(|s| s.strip_suffix(']'));
//        let rotations = rotations.ok_or("expected [...]")?;
//        let rotations = rotations
//            .split(',')
//            .map(|s| s.trim().parse::<u8>())
//            .collect::<Result<Vec<_>, _>>()?;
//        Ok(Self { seed, rotations })
//    }
//}

type IndexSet = smallbitset::Set64;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum TileConfig {
    Empty,
    Tile(TileType, u8),
}

impl TileConfig {
    pub fn from(index: usize) -> Self {
        match ELEMENT_TABLE.get(index) {
            None => unreachable!(),
            Some(elem) => *elem,
        }
    }

    pub fn endings(self) -> [bool; 6] {
        match self {
            TileConfig::Empty => [false; 6],
            TileConfig::Tile(tile_type, rotation) => {
                let mut result = [false; 6];
                let endings = tile_type.endings();
                for i in 0..6 {
                    result[(i + rotation as usize) % result.len()] = endings[i];
                }
                result
            }
        }
    }

    pub fn rotation(self) -> u8 {
        match self {
            TileConfig::Empty => 0,
            TileConfig::Tile(_, r) => r,
        }
    }

    pub fn angle(self) -> Angle {
        Angle::radians(-std::f32::consts::FRAC_PI_3 * self.rotation() as f32)
    }

    pub fn normalized(self) -> Self {
        match self {
            TileConfig::Empty => TileConfig::Empty,
            TileConfig::Tile(t, r) => TileConfig::Tile(t, r % 6),
        }
    }

    pub fn with_rotation(self, r: u8) -> Self {
        match self {
            TileConfig::Empty => TileConfig::Empty,
            TileConfig::Tile(t, _) => TileConfig::Tile(t, r)
        }
    }

    pub fn rotate_by(self, d: u8) -> Self {
        match self {
            TileConfig::Empty => TileConfig::Empty,
            TileConfig::Tile(t, r) => TileConfig::Tile(t, r + d).normalized(),
        }
    }

    pub fn index(&self) -> usize {
        match ELEMENT_TABLE.iter().position(|e| *e == self.normalized()) {
            None => unreachable!(),
            Some(i) => i,
        }
    }

    pub fn model(self) -> Range<i32> {
        match self {
            TileConfig::Empty => panic!("TileConfig::Empty has no model"),
            TileConfig::Tile(t, _) => t.model(),
        }
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
