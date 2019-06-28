#![allow(dead_code, unused_imports, unused_variables)]

mod parser;

use std::{env, fs, io};
use std::collections::{HashSet, HashMap};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Point { x: isize, y: isize }

impl Point {
    fn from_isize(x: isize, y: isize) -> Point { Point{ x, y } }
    fn from_usize(x: usize, y: usize) -> Point { Point{ x: x as isize, y: y as isize} }
}

#[derive(Debug)]
struct Line { from: Point, to: Point }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Cell { EMPTY, BLOCKED, WRAPPED }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Bonus { HAND, WHEELS, DRILL, TELEPORT, CLONE, SPAWN, BEACON }

struct Drone {
    pos: Point,
    hands: Vec<Point>,
    active: HashMap<Bonus, usize>
}

impl Drone {
    fn new(pos: Point) -> Drone {
        Drone { pos, 
                hands: vec![Point::from_isize(1,0), Point::from_isize(1,1), Point::from_isize(1,-1)],
                active: HashMap::new() } 
    }
}

struct Level { 
    grid:    Vec<Cell>,
    width:   usize,
    height:  usize,
    bonuses: HashMap<Point, Bonus>,
    picked:  HashMap<Bonus, usize>,
    drones:  Vec<Drone>
}

impl Level {
    fn get_cell(&self, x: usize, y: usize) -> Cell {
        assert!(x < self.width && y < self.height);
        self.grid[x + y * self.width]
    }

    fn print(&self) {
        for y in (0..self.height).rev() {
            for x in 0..self.width {
                let point = Point::from_usize(x, y);
                let char = if let Some(idx) = self.drones.iter().position(|d| d.pos == point) {
                        idx.to_string()
                    } else if let Some(_) = self.drones.iter().find(|d| d.hands.iter().find(|h| d.pos.x + h.x == x as isize && d.pos.y + h.y == y as isize).is_some()) {
                        String::from(
                            match self.get_cell(x, y) {
                                Cell::EMPTY   => { "*" }
                                Cell::BLOCKED => { "█" }
                                Cell::WRAPPED => { "+" }
                            }
                        )
                    } else if let Some(bonus) = self.bonuses.get(&point) {
                        String::from(match bonus {
                            Bonus::HAND     => { "B" }
                            Bonus::WHEELS   => { "F" }
                            Bonus::DRILL    => { "L" }
                            Bonus::SPAWN    => { "X" }
                            Bonus::TELEPORT => { "R" }
                            Bonus::BEACON   => { "T" }
                            Bonus::CLONE    => { "C" }
                        })
                    } else {
                        String::from(match self.get_cell(x, y) {
                            Cell::EMPTY   => { "░" }
                            Cell::BLOCKED => { "█" }
                            Cell::WRAPPED => { "▒" }
                        })
                    };
                print!("{}", char);
            }
            println!()
        }
        println!()
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if let Some(filename) = args.get(1) {
        let contents = fs::read_to_string(filename).unwrap();
        parser::parse_level(&contents);
    } else {
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        parser::parse_level(&input);
    }
}
