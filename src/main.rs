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

    fn is_active(&self, bonus: &Bonus) -> bool {
        get_or(&self.active, bonus, 0) > 0
    }

    fn wrap(&self, level: &mut Level) {
        let mut to_wrap: HashSet<Point> = HashSet::new();
        would_wrap(level, self, &self.pos, &mut to_wrap);
        for p in to_wrap { level.wrap_cell(p.x, p.y); }
    }

    fn collect(&self, level: &mut Level) {
        if let Some(bonus) = level.bonuses.get(&self.pos) {
            if let Some(collected) = level.collected.get_mut(bonus) {
                *collected += 1;
            } else {
                level.collected.insert(*bonus, 1);
            }
            level.bonuses.remove(&self.pos);
        }
    }

    fn wear_off(&mut self) {
        self.active.retain(|_, val| { *val -= 1; *val > 0 });
    }

    fn activate_wheels(&mut self, level: &mut Level) -> bool {
        if get_or(&level.collected, &Bonus::WHEELS, 0) > 0 && !self.is_active(&Bonus::WHEELS) {
            update(&mut level.collected, Bonus::WHEELS, -1);
            update(&mut self.active, Bonus::WHEELS, 51);
            self.path += "F";
            true
        } else { false }
    }

    fn act(&mut self, action: &Action, level: &mut Level) {
        let wheels = self.is_active(&Bonus::WHEELS);
        let drill = self.is_active(&Bonus::DRILL);
        if let Some((pos, new_wrapped, new_drilled)) = step(level, self, &self.pos, action, wheels, drill, &HashSet::new()) {
            self.pos = pos;
            self.path += match action { Action::UP => "W", Action::DOWN => "S", Action::LEFT => "A", Action::RIGHT => "D" };
            for p in new_wrapped {
                level.wrap_cell(p.x, p.y);
            }
        } else {
            panic!("Unwalkable from ({},{}) {:?}", self.pos.x, self.pos.y, action);
        }
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

    fn wrap_cell(&mut self, x: isize, y: isize) {
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

fn would_wrap(level: &Level, drone: &Drone, pos: &Point, wrapped: &mut HashSet<Point>) { // TODO make an iterator?
    wrapped.insert(*pos);
    for Point{x, y} in &drone.hands {
        let hx = pos.x + *x;
        let hy = pos.y + *y;
        // TODO check hand visibility
        if level.valid(hx, hy) && level.get_cell(hx, hy) == Cell::EMPTY {
            wrapped.insert(Point::new(hx, hy));
        }
    }
}

fn step(level: &Level, drone: &Drone, from: &Point, action: &Action, wheels: bool, drill: bool, drilled: &HashSet<Point>) -> Option<(Point, HashSet<Point>, HashSet<Point>)> {
    let (dx, dy) = match action {
        Action::LEFT  => (-1,  0),
        Action::RIGHT => ( 1,  0),
        Action::UP    => ( 0,  1),
        Action::DOWN  => ( 0, -1)
    };
    let mut to = Point::new(from.x + dx, from.y + dy);
    let mut new_wrapped = HashSet::new();
    let mut new_drilled = HashSet::new();
    if drilled.contains(&to) || (drill && level.valid(to.x, to.y)) || level.walkable(to.x, to.y) {
        would_wrap(level, drone, &to, &mut new_wrapped);
        if drill && !drilled.contains(&to) && !level.walkable(to.x, to.y) {
            new_drilled.insert(to);
        }
        if wheels {
            let to2 = Point::new(to.x + dx, to.y + dy);
            if drilled.contains(&to2) || (drill && level.valid(to2.x, to2.y)) || level.walkable(to2.x, to2.y) {
                would_wrap(level, drone, &to2, &mut new_wrapped);
                if drill && !drilled.contains(&to2) && level.valid(to2.x, to2.y) && !level.walkable(to2.x, to2.y) {
                    new_drilled.insert(to2);
                }
                to = to2;
            }
        }
        Some((to, new_wrapped, new_drilled))
    } else {
        None
    }
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
            
            for action in &[Action::LEFT, Action::RIGHT, Action::UP, Action::DOWN] {
                if let Some((pos2, new_wrapped, new_drilled)) = step(level, drone, &pos, action, wheels > 0, drill > 0, &drilled) {
                    if seen.contains(&pos2) { continue; }
                    seen.insert(pos2);
                    let mut plan2 = plan.clone();
                    plan2.push_back(*action);
                    let mut drilled2 = drilled.clone();
                    for p in new_drilled { drilled2.insert(p); }
                    queue.push_back(Plan{
                        plan:    plan2,
                        pos:     pos2,
                        wheels:  if wheels > 1 { wheels - 1 } else { 0 },
                        drill:   if drill > 1  { drill - 1 }  else { 0 },
                        drilled: drilled2
                    });    
                }
            }
        } else {
            break None
        }
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
    drones[0].wrap(level);
    while level.empty > 0 {
        for drone_idx in 0..drones.len() {
            if debug { print_debug(level, drones); }

            if level.empty <= 0 { break; }

            let mut drone = &mut drones[drone_idx];
            drone.collect(level);
            drone.wear_off();
            
            if drone.plan.is_empty() {
                if drone.activate_wheels(level) { continue; }
                if let Some(plan) = explore(level, drone, max_wrapping) {
                    drone.plan = plan;
                } else { break; }
            }
            
            if let Some(action) = drone.plan.pop_front() {
                drone.act(&action, level);
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
