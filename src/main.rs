#![allow(dead_code, unused_imports, unused_variables)]

mod parser;

use std::{env, fs, io, thread, time};
use std::fs::{File};
use std::io::prelude::*;
use std::collections::{HashSet, HashMap, VecDeque};
use std::time::{Instant};
use regex::Regex;

const DELAY: u64 = 50;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Point { x: isize, y: isize }

impl Point {
    fn new(x: isize, y: isize) -> Point { Point{ x, y } }
}

#[derive(Debug)]
struct Line { from: Point, to: Point }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Cell { EMPTY, BLOCKED, WRAPPED }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Action { UP, RIGHT, DOWN, LEFT }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Bonus { HAND, WHEELS, DRILL, TELEPORT, CLONE }

pub struct Drone {
    pos:    Point,
    hands:  Vec<Point>,
    active: HashMap<Bonus, usize>,
    path:   String,
    plan:   VecDeque<Action>
}

impl Drone {
    fn new(pos: Point) -> Drone {
        Drone { pos, 
                hands:  vec![Point::new(1,0), Point::new(1,1), Point::new(1,-1)],
                active: HashMap::new(),
                path:   String::new(),
                plan:   VecDeque::new() }
    }

    fn act(&mut self, action: Action, level: &mut Level) {
        let (dx, dy) = match action {
            Action::LEFT  => { (-1,  0) }
            Action::RIGHT => { ( 1,  0) }
            Action::UP    => { ( 0,  1) }
            Action::DOWN  => { ( 0, -1) }
        };
        let x2 = self.pos.x + dx;
        let y2 = self.pos.y + dy;
        if level.walkable(x2, y2) {
            self.pos = Point::new(x2, y2);
            mark_level(level, self);
            if get_or(&self.active, &Bonus::WHEELS, 0) > 0 && level.walkable(x2 + dx, y2 + dy) {
                self.pos = Point::new(x2 + dx, y2 + dy);
                mark_level(level, self);
            }
            self.path += match action {
                Action::LEFT  => { "A" }
                Action::RIGHT => { "D" }
                Action::UP    => { "W" }
                Action::DOWN  => { "S" }
            };
        } else {
            panic!("Unwalkable ({},{})", x2, y2);
        }
    }

    fn activate(&mut self, level: &mut Level) -> bool {
        if get_or(&level.collected, &Bonus::WHEELS, 0) > 0 && get_or(&self.active, &Bonus::WHEELS, 0) == 0 {
            update(&mut level.collected, Bonus::WHEELS, -1);
            update(&mut self.active, Bonus::WHEELS, 51);
            self.path += "F";
            true
        } else {
            false
        }
    }

    fn wear_off(&mut self) {
        self.active.retain(|_, val| { *val -= 1; *val > 0 });
    }
}

pub struct Level {
    grid:      Vec<Cell>,
    width:     isize,
    height:    isize,
    empty:     usize,
    spawns:    HashSet<Point>,
    beakons:   HashSet<Point>,
    bonuses:   HashMap<Point, Bonus>,
    collected: HashMap<Bonus, usize>
}

impl Level {
    fn coord_to_offset(&self, x: isize, y: isize) -> usize {
        (x + y * self.width) as usize
    }

    fn get_cell(&self, x: isize, y: isize) -> Cell {
        assert!(x >= 0 && x < self.width && y >= 0 && y < self.height);
        self.grid[self.coord_to_offset(x, y)]
    }

    fn mark_cell(&mut self, x: isize, y: isize) {
        assert!(x >= 0 && x < self.width && y >= 0 && y < self.height);
        if self.get_cell(x, y) == Cell::EMPTY {
            let offset = self.coord_to_offset(x, y);
            self.empty -= 1;
            self.grid[offset] = Cell::WRAPPED;
        }
    }

    fn valid(&self, x: isize, y: isize) -> bool {
        x >= 0 && x < self.width as isize && y >= 0 && y < self.height as isize
    }

    fn walkable(&self, x: isize, y: isize) -> bool {
        self.valid(x, y) && self.get_cell(x, y) != Cell::BLOCKED
    }
}

fn print(level: &Level, drones: &Vec<Drone>) {
    for y in (0..level.height).rev() {
        for x in 0..level.width {
            let point = Point::new(x, y);
            let char = if let Some(idx) = drones.iter().position(|d| d.pos == point) {
                    idx.to_string()
                } else if let Some(_) = drones.iter().find(|d| d.hands.iter().find(|h| d.pos.x + h.x == x as isize && d.pos.y + h.y == y as isize).is_some()) {
                    String::from(
                        match level.get_cell(x, y) {
                            Cell::EMPTY   => { "*" }
                            Cell::BLOCKED => { "█" }
                            Cell::WRAPPED => { "+" }
                        }
                    )
                } else if let Some(bonus) = level.bonuses.get(&point) {
                    String::from(match bonus {
                        Bonus::HAND     => { "B" }
                        Bonus::WHEELS   => { "F" }
                        Bonus::DRILL    => { "L" }
                        Bonus::TELEPORT => { "R" }
                        Bonus::CLONE    => { "C" }
                    })
                } else if level.spawns.contains(&point) {
                    String::from("X")
                } else if level.beakons.contains(&point) {
                    String::from("T")
                } else {
                    String::from(match level.get_cell(x, y) {
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

fn get_or<K>(m: &HashMap<K, usize>, k: &K, default: usize) -> usize
    where K: std::hash::Hash + Eq + std::marker::Sized
{
    if let Some(v) = m.get(k) { *v } else { default }
}

fn update<K>(m: &mut HashMap<K, usize>, k: K, delta: isize)
    where K: std::hash::Hash + Eq + std::marker::Sized
{
    let old_v: usize = get_or(m, &k, 0);
    let new_v = old_v as isize + delta;
    if new_v > 0 { m.insert(k, new_v as usize); }
    else { m.remove(&k); }
}

fn mark_level(level: &mut Level, drone: &Drone) {
    let dx = drone.pos.x;
    let dy = drone.pos.y;

    level.mark_cell(dx, dy);

    for Point{x, y} in &drone.hands {
        let hx = dx + *x;
        let hy = dy + *y;
        // TODO visible
        if level.valid(hx, hy) {
            level.mark_cell(hx, hy);
        }
    }
}

struct Plan {
    plan:    VecDeque<Action>,
    pos:     Point,
    wheels:  usize,
    drill:   usize,
    drilled: HashSet<Point>
}

fn max_wrapping(level: &Level, p: &Point) -> f64 {
    if level.bonuses.contains_key(p) { 100. }
    else if Cell::EMPTY == level.get_cell(p.x, p.y) { 1. }
    else { 0. }
}

fn explore<F>(level: &Level, drone: &Drone, rate: F) -> Option<VecDeque<Action>>
    where F: Fn(&Level, &Point) -> f64
{
    let mut seen: HashSet<Point> = HashSet::new();
    let mut queue: VecDeque<Plan> = VecDeque::with_capacity(100);
    let mut best: Option<(VecDeque<Action>, f64)> = None;
    let mut max_len = 5;
    queue.push_back(Plan{plan:    VecDeque::new(),
                         pos:     drone.pos,
                         wheels:  get_or(&drone.active, &Bonus::WHEELS, 0),
                         drill:   get_or(&drone.active, &Bonus::DRILL, 0),
                         drilled: HashSet::new() });
    loop {
        if let Some(Plan{plan, pos, wheels, drill, drilled}) = queue.pop_front() {
            if plan.len() >= max_len {
                if best.is_some() {
                    break Some(best.unwrap().0)
                } else {
                    max_len += 5;
                }
            }

            let score = if plan.is_empty() { 0. } else { rate(level, &pos) / plan.len() as f64 };

            if best.is_some() {
                if score > best.as_ref().unwrap().1 { best = Some((plan.clone(), score)); }
            } else {
                if score > 0. { best = Some((plan.clone(), score)); }
            }
            
            for (action, dx, dy) in &[(Action::LEFT,  -1,  0),
                                      (Action::RIGHT,  1,  0),
                                      (Action::UP,     0,  1),
                                      (Action::DOWN,   0, -1)] {
                let (x2, y2) = (pos.x + dx, pos.y + dy);
                if !level.walkable(x2, y2) { continue; }
                let (x3, y3) = if wheels > 0 && level.walkable(x2 + dx, y2 + dy) {
                    (x2 + dx, y2 + dy)
                } else { (x2, y2) };
                let pos3 = Point::new(x3, y3);
                if seen.contains(&pos3) { continue; }
                seen.insert(pos3);
                let mut plan2 = plan.clone();
                plan2.push_back(*action);
                queue.push_back(Plan{
                    plan:    plan2,
                    pos:     pos3,
                    wheels:  if wheels > 1 { wheels - 1 } else { 0 },
                    drill:   if drill > 1  { drill - 1 }  else { 0 },
                    drilled: drilled.clone()
                });
            }
        } else {
            break None
        }
    }
}

fn collect(level: &mut Level, drone: &Drone) {
    if let Some(bonus) = level.bonuses.get(&drone.pos) {
        if let Some(collected) = level.collected.get_mut(bonus) {
            *collected += 1;
        } else {
            level.collected.insert(*bonus, 1);
        }
        level.bonuses.remove(&drone.pos);
    }
}

fn print_debug(level: &Level, drones: &Vec<Drone>) {
    println!("\x1B[1J");
    print(level, &drones);
    println!("Collected {:?}", level.collected);
    let active: Vec<&HashMap<Bonus, usize>> = drones.iter().map(|d| &d.active).collect();
    println!("Active {:?}", active);
    println!("Empty {}", level.empty);
    thread::sleep(time::Duration::from_millis(DELAY));
}

fn solve(level: &mut Level, drones: &mut Vec<Drone>, debug: bool) -> String {
    if debug { println!("\x1B[?1049h"); }
    mark_level(level, &drones[0]);
    while level.empty > 0 {
        for drone_idx in 0..drones.len() {
            if debug { print_debug(level, drones); }

            if level.empty <= 0 { break; }

            let mut drone = &mut drones[drone_idx];
            collect(level, &drone);
            drone.wear_off();
            
            if drone.plan.is_empty() {
                if drone.activate(level) { continue; }

                if let Some(plan) = explore(level, drone, max_wrapping) {
                    drone.plan = plan;
                } else { break; }
            }
            
            if let Some(action)= drone.plan.pop_front() {
                drone.act(action, level);
            } else { break; }
        }
    }
    
    if debug {
        print_debug(level, drones);
        println!("\x1B[?1049l");
    }
    
    let paths: Vec<&str> = drones.iter().map(|d| d.path.as_str()).collect();
    paths.join("#")
}

fn main() {
    let t_start = Instant::now();
    let args: Vec<String> = env::args().collect();
    let mut debug = false;
    let mut filename: Option<&str> = None;
    
    for arg in args[1..].iter() {
        if arg == "--debug" { debug = true; }
        else if arg.ends_with(".desc") { filename = Some(arg); }
        else { panic!("cargo run [--debug] <path/to/problem.desc>"); }
    }
    
    let contents = fs::read_to_string(filename.unwrap()).unwrap();
    // let mut input = String::new();
    // io::stdin().read_line(&mut input).unwrap();
    let (mut level, mut drones) = parser::parse_level(&contents);
    let solution = solve(&mut level, &mut drones, debug);
    let score = Regex::new(r"[A-Z]").unwrap().find_iter(&solution).count();
    eprintln!("Score: {}\nElapsed: {} ms", score, t_start.elapsed().as_millis());
    
    let filename_sol = Regex::new(r"\.desc$").unwrap().replace(filename.unwrap(), ".sol");
    let mut file = File::create(filename_sol.into_owned()).unwrap();
    file.write_all(solution.as_bytes()).unwrap();

    println!("{}", solution);
}
