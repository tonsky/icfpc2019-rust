use lazy_static::lazy_static;
use regex::{Regex, Captures};
use std::cmp;
use std::collections::{HashSet, HashMap, VecDeque};
use crate::{ Point, Line, Cell, Bonus, Drone, Level, UNDECIDED_ZONE };
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg32;

lazy_static! {
    static ref POINT_RE: Regex = Regex::new(r"\((?P<X>-?\d+),(?P<Y>-?\d+)\)").unwrap();
    static ref BONUS_RE: Regex = Regex::new(r"(?P<P>[BFLRC])\((?P<X>-?\d+),(?P<Y>-?\d+)\)").unwrap();
    static ref SPAWN_RE: Regex = Regex::new(r"X\((?P<X>-?\d+),(?P<Y>-?\d+)\)").unwrap();
}

fn grid_idx(x: isize, y: isize, width: isize) -> usize {
    (x + y * width) as usize
}

fn parse_point(s: &str) -> Point {
    let captures = POINT_RE.captures(s).unwrap();
    Point::new(captures["X"].parse::<isize>().unwrap(), captures["Y"].parse::<isize>().unwrap())
}

fn parse_bonus(captures: Captures) -> (Point, Bonus) {
    (Point::new(captures["X"].parse::<isize>().unwrap(), captures["Y"].parse::<isize>().unwrap()),
     match &captures["P"] {
         "B" => { Bonus::HAND }
         "F" => { Bonus::WHEELS }
         "L" => { Bonus::DRILL }
         "R" => { Bonus::TELEPORT }
         "C" => { Bonus::CLONE }
         _   => panic!("Unknown bonus")
     })
}

fn parse_contour(s: &str) -> HashSet<Point> {
    let points: Vec<Point> = POINT_RE.find_iter(s).map(|m| parse_point(m.as_str())).collect();
    let mut walls: HashSet<Point> = HashSet::with_capacity(points.len());
    for (i, &p1) in points.iter().enumerate() {
        let p2 = points[(i+1) % points.len()];
        if p1.x == p2.x { // vercical only
            for y in if p1.y < p2.y { p1.y .. p2.y } else { p2.y .. p1.y } {
                walls.insert(Point::new(p1.x, y));
            }
        }
    }
    walls
}

fn wall_on_left(x: usize, y: usize, walls: &Vec<Line>) -> bool {
    walls.iter().any(|l| l.from.x == x as isize
        && l.from.y <= y as isize
        && l.to.y >= (y + 1) as isize)
}

fn weights(grid: &[Cell], width: isize, height: isize) -> Vec<u8> {
    let mut weights: Vec<u8> = Vec::with_capacity(grid.len());
    for y in 0..height {
        for x in 0..width {
            let mut sum: u8 = 0;
            for (dx, dy) in &[(0,1),(0,-1),(-1,0),(1,0),(1,1),(-1,-1),(-1,1),(1,-1)] {
                let x2 = x + dx;
                let y2 = y + dy;
                if x2 >= 0 && x2 < width && y2 >= 0 && y2 < height && grid[grid_idx(x2, y2, width)] == Cell::BLOCKED {
                    sum += 1;
                }
            }
            weights.push(sum);
        }
    }
    weights
}

fn zones(zones_count: usize, grid: &[Cell], width: isize, height: isize) -> (Vec<u8>, Vec<usize>) {
    let len = (width * height) as usize;

    let mut zones: Vec<u8> = Vec::with_capacity(len);
    for i in 0..len { zones.push(UNDECIDED_ZONE); }

    let mut zones_empty: Vec<usize> = Vec::with_capacity(zones_count);
    for i in 0..zones_count { zones_empty.push(0); }

    let mut queue: VecDeque<(Point, u8)> = VecDeque::with_capacity(len);
    let mut rng = rand_pcg::Pcg32::seed_from_u64(42);
    while queue.len() < zones_count {
        let x = rng.gen_range(0, width);
        let y = rng.gen_range(0, height);
        let idx = grid_idx(x, y, width);
        let point = Point::new(x, y);
        if grid[idx] == Cell::EMPTY && queue.iter().find(|(p, _)| *p == point).is_none() {
            queue.push_back((point, queue.len() as u8));
        }
    }

    while let Some((Point{x, y}, zone)) = queue.pop_front() {
        let idx = grid_idx(x, y, width);
        if zones[idx] == UNDECIDED_ZONE && grid[idx] == Cell::EMPTY {
            zones_empty[zone as usize] += 1;
            zones[idx] = zone;
            if y + 1 < height { queue.push_back((Point::new(x, y + 1), zone)); }
            if y > 0          { queue.push_back((Point::new(x, y - 1), zone)); }
            if x + 1 < width  { queue.push_back((Point::new(x + 1, y), zone)); }
            if x > 0          { queue.push_back((Point::new(x - 1, y), zone)); }
        }
    }

    (zones, zones_empty)
}

fn build_level(walls: &HashSet<Point>, zones_count: usize) -> Level {
    let height = walls.iter().max_by_key(|p| p.y).unwrap().y + 1;
    let width = walls.iter().max_by_key(|p| p.x).unwrap().x;
    let mut grid = Vec::with_capacity((width * height) as usize);
    let mut empty = 0;
    for y in 0..height {
        let mut last_cell = Cell::BLOCKED;
        for x in 0..width {
            if walls.contains(&Point::new(x, y)) {
                last_cell = if last_cell == Cell::EMPTY { Cell::BLOCKED } else { Cell::EMPTY };
            }
            grid.push(last_cell);
            if last_cell == Cell::EMPTY { empty += 1; }
        }
        assert_eq!(walls.contains(&Point::new(width, y)), Cell::EMPTY == last_cell);
    }
    let weights = weights(&grid, width, height);
    let (zones, zones_empty) = zones(zones_count, &grid, width, height);
    Level {
        grid, weights, zones, width, height, empty, zones_empty, 
        spawns:    HashSet::new(),
        beakons:   Vec::new(),
        bonuses:   HashMap::new(),
        collected: HashMap::new()
    }
}

pub fn parse_level(file: &str) -> (Level, Vec<Drone>) {
    let fragments: Vec<&str> = file.split("#").collect();
    match *fragments {
        [walls_str, start_str, obstacles_str, bonuses_str] => {
            let mut walls = parse_contour(walls_str);
            for obstacle_str in obstacles_str.split(";").filter(|s| !s.is_empty()) {
                walls.extend(parse_contour(obstacle_str));
            }
            let clones = Regex::new(r"C\(\d+,\d+\)").unwrap().find_iter(bonuses_str).count();
            let mut level = build_level(&walls, clones + 1);

            for captures in BONUS_RE.captures_iter(bonuses_str) {
                let (pos, bonus) = parse_bonus(captures);
                level.bonuses.insert(pos, bonus);
            }
            for captures in SPAWN_RE.captures_iter(bonuses_str) {
                let pos = Point::new(captures["X"].parse::<isize>().unwrap(), captures["Y"].parse::<isize>().unwrap());
                level.spawns.insert(pos);
            }
            (level, vec![Drone::new(parse_point(start_str))])
        }
        _ => panic!("incomplete file")
    }
}
