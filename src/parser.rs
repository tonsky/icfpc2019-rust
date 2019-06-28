use lazy_static::lazy_static;
use regex::Regex;
use std::cmp;
use std::collections::{HashMap};
use crate::{ Point, Line, Cell, Bonus, Drone, Level };

lazy_static! {
    static ref POINT_RE: Regex = Regex::new(r"\((?P<X>-?\d+),(?P<Y>-?\d+)\)").unwrap();
}

fn parse_point(s: &str) -> Point {
    let captures = POINT_RE.captures(s).unwrap();
    Point::from_isize(captures["X"].parse::<isize>().unwrap(), captures["Y"].parse::<isize>().unwrap())
}

fn parse_bonus(s: &str) -> (Point, Bonus) {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"(?P<P>[BFLRCX])\((?P<X>-?\d+),(?P<Y>-?\d+)\)").unwrap();
    }
    let captures = RE.captures(s).unwrap();
    (Point::from_isize(captures["X"].parse::<isize>().unwrap(), captures["Y"].parse::<isize>().unwrap()),
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

fn parse_contour(s: &str) -> Vec<Line> {
    let points: Vec<Point> = POINT_RE.find_iter(s).map(|m| parse_point(m.as_str())).collect();
    let mut lines: Vec<Line> = Vec::with_capacity(points.len() / 2);
    for (i, &p1) in points.iter().enumerate() {
        let p2 = points[(i+1) % points.len()];
        if p1.y != p2.y { // vercical only
            if p1.y < p2.y { // arrange bottom up
                lines.push(Line{from: p1, to: p2});
            } else {
                lines.push(Line{from: p2, to: p1});
            }
        }
    }
    lines
}

fn wall_on_left(x: usize, y: usize, walls: &Vec<Line>) -> bool {
    walls.iter().any(|l| l.from.x == x as isize
        && l.from.y <= y as isize
        && l.to.y >= (y + 1) as isize)
}

fn build_level(walls: &Vec<Line>) -> Level {
    let height = walls.iter().max_by_key(|l| l.to.y).unwrap().to.y as usize;
    let line = walls.iter().max_by_key(|l| cmp::max(l.from.x, l.to.x)).unwrap();
    let width = cmp::max(line.from.x, line.to.x) as usize;
    let mut grid = Vec::with_capacity(width * height);
    for y in 0..height {
        let mut last_cell = Cell::BLOCKED;
        for x in 0..width {
            if wall_on_left(x, y, walls) {
                last_cell = if last_cell == Cell::EMPTY { Cell::BLOCKED } else { Cell::EMPTY };
            }
            grid.push(last_cell);
        }
        assert_eq!(wall_on_left(width, y, walls), Cell::EMPTY == last_cell);
    }
    Level { grid, width, height,
        bonuses: HashMap::new(),
        picked: HashMap::new(),
        drones: Vec::with_capacity(1) }
}


pub fn parse_level(file: &str) {
    let fragments: Vec<&str> = file.split("#").collect();
    match *fragments {
        [walls_str, start_str, obstacles_str, bonuses_str] => {
            let mut walls: Vec<Line> = parse_contour(walls_str);
            walls.extend(obstacles_str.split(";").flat_map(|s| parse_contour(s)));
            let mut level = build_level(&walls);
            for (pos, bonus) in bonuses_str.split(";").map(|s| parse_bonus(s)) {
                level.bonuses.insert(pos, bonus);
            }
            level.drones.push(Drone::new(parse_point(start_str)));
            level.print();
        }
        _ => panic!("incomplete file")
    }
}
