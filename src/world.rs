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
    elements: Box<[Option<WorldElement>]>
}

fn build_symmetries(tile_type: TileType) -> Vec<u8> {
    let endings = tile_type.endings();
    let mut result = Vec::new();
    for i in 0..endings.len() {
        if !result.iter().any(|j| {
            for x in 0..endings.len() {
                if endings[(x + i as usize) % endings.len()] != endings[(x + *j as usize) % endings.len()]{
                    return false;
                }
            }
            true
        }) {
            result.push(i as u8);
        }
    }
    result
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

fn build_adjacency_lists(table: &Vec<Option<WorldElement>>) -> Vec<[Vec<usize>; 6]>{

    let mut result = Vec::new();

    for i in 0..table.len() {
        let mut lists: [Vec<_>; 6] = Default::default();

        for j in 0..lists.len() {

            for k in 0..table.len() {

                let elem1 = get_endings(table[i]);
                let elem2 = get_endings(table[k]);

                if elem1[j] == elem2[(j + 3) % elem2.len()]{
                    lists[j].push(k);
                }
            }

        }

        result.push(lists);
    }

    result
}

impl World {

    fn new(rows: u32, width: u32) -> Self {
        Self {
            rows,
            width,
            elements: vec![None; (rows * width + (rows / 2)) as usize].into_boxed_slice()
        }
    }

    pub fn from_seed(seed: u64) -> Self{
        let rng = fastrand::Rng::with_seed(seed);
        let mut world = World::new(9, 5);

        let table = {
            let mut v = build_table();
            rng.shuffle(v.as_mut_slice());
            v
        };
        let adjacency_lists = build_adjacency_lists(&table);

        //for i in 0..table.len() {
        //    console_log!("{}: {:?}", i, table[i]);
        //}

        let elem= world.elements.len() / 2;
        let item = rng.usize(0..table.len());
        //console_log!("current element: {}", item);
        *world.get_element(elem) = table[item];
        //console_log!("endings: {:?}", get_endings(table[item]));
        for (i, n) in world.get_neighbors(elem).iter().enumerate() {
            if let Some(n) = *n {
                let al = &adjacency_lists[item][i];
                //console_log!("al {}: {:?}", i, al);
                let select = al[rng.usize(0..al.len())];
                //console_log!("picked {}", select);
                *world.get_element(n) = table[select];
            }
        }

        let mut possibilities = {
            let mut vec = Vec::new();
            for _ in world.indices() {
                let mut v = Vec::new();
                for i in 0..table.len() {
                    v.push(i);
                }
                vec.push(v);
            }
            vec
        };

        let lowest_entropy = |vec: &Vec<Vec<usize>>| {
            let mut set = Vec::new();
            let mut min = usize::MAX;
            for i in 0..vec.len() {
                let l = vec[i].len();
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
                Some(set[rng.usize(0..set.len())])
            }
        };

        let mut stack = VecDeque::new();
        loop
        {
            match lowest_entropy(&possibilities){
                None => break,
                Some(field) => {
                    let selected = possibilities[field][rng.usize(0..possibilities[field].len())];
                    possibilities[field].clear();
                    possibilities[field].push(selected);

                    stack.clear();
                    stack.push_back(field);

                    console_log!("Collapsed {}", field);

                    loop {
                        match stack.pop_front() {
                            None => break,
                            Some(index) => {
                                console_log!("Propagating {}", index);
                                let id = *possibilities[index].first().unwrap();
                                for (i, n) in world.get_neighbors(index).iter().enumerate() {
                                    if let Some(n) = *n {
                                        let prl = possibilities[n].len();
                                        possibilities[n].retain(|g| adjacency_lists[id][i].contains(g));
                                        if prl != possibilities[n].len() {
                                            stack.push_back(n);
                                        }
                                    }
                                }

                            }
                        }
                    }

                }
            }

        }

        console_log!("{:?}", possibilities.iter().map(|v|v.len()).collect::<Vec<usize>>());

        for (i, e) in world.elements.iter_mut().enumerate(){
            *e = table[*possibilities[i].first().unwrap()];
        }

        //for i in world.indices() {
        //    *world.get_element(i) = Some(WorldElement {
        //        tile_type: match rng.u8(0..7) {
        //            0 => TileType::Tile0,
        //            1 => TileType::Tile01,
        //            2 => TileType::Tile02,
        //            3 => TileType::Tile03,
        //            4 => TileType::Tile012,
        //            5 => TileType::Tile024,
        //            6 => TileType::Tile0134,
        //            _ => unreachable!()
        //        },
        //        rotation: 0
        //    });
        //}

        /*
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        queue.push_back(world.elements.len() / 2);

        loop {
            match queue.pop_front() {
                None => break,
                Some(elem) => {
                    if !visited.contains(&elem) {
                        *world.get_element(elem) = Some(WorldElement {
                            tile_type: match rng.u8(0..7) {
                                0 => TileType::Tile0,
                                1 => TileType::Tile01,
                                2 => TileType::Tile02,
                                3 => TileType::Tile03,
                                4 => TileType::Tile012,
                                5 => TileType::Tile024,
                                6 => TileType::Tile0134,
                                _ => unreachable!()
                            },
                            rotation: 0
                        });
                        visited.insert(elem);
                        let (x,y) = world.get_xy(elem);
                        let (x, y) = (x as i32, y as i32);
                        let dx = -2 * (y & 1) as i32 + 1;
                        for (nx, ny) in &[(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1), (x + dx, y + 1), (x + dx, y - 1)] {
                            if let Some(i) = world.get_index(*nx, *ny) {
                                queue.push_back(i);
                            }
                        }
                    }
                }
            }
        }
        */
        world
    }

    pub fn get_neighbors(&self, index: usize) -> [Option<usize>; 6] {
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
    pub fn indices(&self) -> Range<usize> {
        0..self.elements.len()
    }
    pub fn get_index(&self, x: i32, y: i32) -> Option<usize> {
        if x < 0 || y < 0 {
            return None;
        }
        let (x,y) = (x as u32, y as u32);
        if y < self.rows && x < (self.width + (y % 2)) {
            Some((y * self.width + (y / 2) + x) as usize)
        } else {
            None
        }
    }
    pub fn get_xy(&self, index: usize) -> (i32, i32) {
        let div = index as i32 / (2 * self.width as i32 + 1);
        let rem = index as i32 % (2 * self.width as i32 + 1);

        if rem < self.width as i32 {
            (rem, 2 * div)
        } else {
            (rem - self.width as i32, 2 * div + 1)
        }
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
