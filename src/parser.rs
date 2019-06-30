use lazy_static::lazy_static;
use regex::Regex;
use std::cmp;
use std::collections::{HashSet, HashMap};
use crate::{ Point, Line, Cell, Bonus, Drone, Level };

lazy_static! {
    static ref POINT_RE: Regex = Regex::new(r"\((?P<X>-?\d+),(?P<Y>-?\d+)\)").unwrap();
    static ref BONUS_RE: Regex = Regex::new(r"(?P<P>[BFLRCX])\((?P<X>-?\d+),(?P<Y>-?\d+)\)").unwrap();
}

fn parse_point(s: &str) -> Point {
    let captures = POINT_RE.captures(s).unwrap();
    Point::new(captures["X"].parse::<isize>().unwrap(), captures["Y"].parse::<isize>().unwrap())
}

fn parse_bonus(s: &str) -> (Point, Bonus) {
    let captures = BONUS_RE.captures(s).unwrap();
    (Point::new(captures["X"].parse::<isize>().unwrap(), captures["Y"].parse::<isize>().unwrap()),
     match &captures["P"] {
         "B" => { Bonus::HAND }
         "F" => { Bonus::WHEELS }
         "L" => { Bonus::DRILL }
         "R" => { Bonus::TELEPORT }
         "C" => { Bonus::CLONE }
         "X" => { Bonus::SPAWN }
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

fn build_level(walls: &HashSet<Point>) -> Level {
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
    Level {
        grid, width, height, empty,
        bonuses: HashMap::new(),
        picked: HashMap::new()
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
            let mut level = build_level(&walls);
            for (pos, bonus) in BONUS_RE.find_iter(bonuses_str).map(|m| parse_bonus(m.as_str())) {
                level.bonuses.insert(pos, bonus);
            }
            (level, vec![Drone::new(parse_point(start_str))])
        }
        _ => panic!("incomplete file")
    }
}
