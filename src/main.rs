#![allow(dead_code, unused_imports, unused_variables)]

mod parser;

use std::{env, fs, io, thread, time};
use std::fs::{File};
use std::io::prelude::*;
use std::collections::{HashSet, HashMap, VecDeque};
use std::time::{Instant};
use regex::Regex;
use lazy_static::lazy_static;

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

fn hand_blockers() -> HashMap<Point, Vec<Point>> {
    let mut res = HashMap::new();
    res.insert(Point::new(0,  0), vec![Point::new(0,  0)]);
    res.insert(Point::new(1, -1), vec![Point::new(1, -1)]);
    res.insert(Point::new(1,  0), vec![Point::new(1,  0)]);
    res.insert(Point::new(1,  1), vec![Point::new(1,  1)]);
    for maxy in 2..19 {
        let mut val = Vec::with_capacity(maxy);
        for y in 1..(maxy/2+1) { val.push(Point::new(0, y as isize)) }
        for y in (maxy+1)/2..(maxy+1) { val.push(Point::new(1, y as isize)) }
        res.insert(Point::new(1, maxy as isize), val);
    }
    res
}

lazy_static! {
    static ref HAND_BLOCKERS: HashMap<Point, Vec<Point>> = hand_blockers();
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
                hands:  vec![Point::new(0,0), Point::new(1,-1), Point::new(1,0), Point::new(1,1)],
                active: HashMap::new(),
                path:   String::new(),
                plan:   VecDeque::new() }
    }

    fn is_active(&self, bonus: &Bonus) -> bool {
        get_or(&self.active, bonus, 0) > 0
    }

    fn wrap_bot(&self, level: &mut Level) {
        let mut to_wrap: HashSet<Point> = HashSet::new();
        would_wrap(level, self, &self.pos, &mut to_wrap);
        for p in to_wrap {
            level.wrap_cell(p.x, p.y);
        }
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

    fn has_space(&self, level: &Level) -> bool {
        (1..5).all(   |i| level.valid(self.pos.x, self.pos.y+i) && level.get_cell(self.pos.x, self.pos.y+i) != Cell::BLOCKED)
        || (1..5).all(|i| level.valid(self.pos.x, self.pos.y-i) && level.get_cell(self.pos.x, self.pos.y-i) != Cell::BLOCKED)
        || (1..5).all(|i| level.valid(self.pos.x+i, self.pos.y) && level.get_cell(self.pos.x+i, self.pos.y) != Cell::BLOCKED)
        || (1..5).all(|i| level.valid(self.pos.x-i, self.pos.y) && level.get_cell(self.pos.x-i, self.pos.y) != Cell::BLOCKED)
    }

    fn activate_wheels(&mut self, level: &mut Level) -> bool {
        if get_or(&level.collected, &Bonus::WHEELS, 0) > 0
           && !self.is_active(&Bonus::WHEELS)
           && self.has_space(level) {
            update(&mut level.collected, Bonus::WHEELS, -1);
            update(&mut self.active, Bonus::WHEELS, 51);
            self.path += "F";
            true
        } else { false }
    }

    fn activate_drill(&mut self, level: &mut Level) -> bool {
        if get_or(&level.collected, &Bonus::DRILL, 0) > 0 && !self.is_active(&Bonus::DRILL) {
            update(&mut level.collected, Bonus::DRILL, -1);
            update(&mut self.active, Bonus::DRILL, 31);
            self.path += "L";
            true
        } else { false }
    }

    fn activate_hand(&mut self, level: &mut Level) -> bool {
        if get_or(&level.collected, &Bonus::HAND, 0) > 0 {
            update(&mut level.collected, Bonus::HAND, -1);
            let new_hand = Point::new(1, self.hands.last().unwrap().y + 1);
            self.path += &format!("B({},{})", new_hand.x, new_hand.y);
            self.hands.push(new_hand);
            true
        } else { false }
    }

    fn reduplicate(&mut self, level: &mut Level) -> Option<Drone> {
        if get_or(&level.collected, &Bonus::CLONE, 0) > 0 && level.spawns.contains(&self.pos) {
            update(&mut level.collected, Bonus::CLONE, -1);
            self.path += "C";
            Some(Drone::new(self.pos))
        } else { None }
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
            for p in new_drilled {
                level.drill_cell(p.x, p.y);
            }
        } else {
            panic!("Unwalkable from ({},{}) {:?}", self.pos.x, self.pos.y, action);
        }
    }
}

pub struct Level {
    grid:      Vec<Cell>,
    weights:   Vec<u8>,
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
        assert!(self.get_cell(x, y) == Cell::EMPTY);
        let offset = self.coord_to_offset(x, y);
        self.empty -= 1;
        self.grid[offset] = Cell::WRAPPED;
    }

    fn drill_cell(&mut self, x: isize, y: isize) {
        assert!(x >= 0 && x < self.width && y >= 0 && y < self.height);
        assert!(self.get_cell(x, y) == Cell::BLOCKED);
        let offset = self.coord_to_offset(x, y);
        self.grid[offset] = Cell::WRAPPED;
    }

    fn valid(&self, x: isize, y: isize) -> bool {
        x >= 0 && x < self.width as isize && y >= 0 && y < self.height as isize
    }

    fn walkable(&self, x: isize, y: isize) -> bool {
        self.valid(x, y) && self.get_cell(x, y) != Cell::BLOCKED
    }
}

fn print_level(level: &Level, drones: &[Drone]) {
    for y in (0..level.height).rev() {
        for x in 0..level.width {
            let point = Point::new(x, y);
            let char = if let Some(idx) = drones.iter().position(|d| d.pos == point) {
                    idx.to_string()
                } else if drones.iter().find(|d| d.hands.iter().find(|h| {
                    d.pos.x + h.x == x as isize && d.pos.y + h.y == y as isize && is_reaching(level, &d.pos, &h)
                  }).is_some()).is_some() {
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

fn max_wrapping(level: &Level, drone: &Drone, pos: &Point) -> f64 {
    if level.bonuses.contains_key(pos) { 
        100.
    } else {
        let mut wrapped: HashSet<Point> = HashSet::new();
        would_wrap(level, drone, pos, &mut wrapped);
        wrapped.iter().map(|p| 1.0_f64.max(level.weights[level.coord_to_offset(p.x, p.y)] as f64)).sum()
    }
}

fn is_reaching(level: &Level, from: &Point, hand: &Point) -> bool {
    HAND_BLOCKERS.get(hand).unwrap().iter().all(|p| level.walkable(from.x+p.x, from.y+p.y))
}

fn would_wrap(level: &Level, drone: &Drone, pos: &Point, wrapped: &mut HashSet<Point>) {
    for hand in &drone.hands {
        if is_reaching(level, pos, &hand) {
            let hand_pos = Point::new(pos.x + hand.x, pos.y + hand.y);
            if level.get_cell(hand_pos.x, hand_pos.y) == Cell::EMPTY {
                wrapped.insert(hand_pos);
            }
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
    where F: Fn(&Level, &Drone, &Point) -> f64
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

            let score = if plan.is_empty() { 0. } else { rate(level, drone, &pos) / plan.len() as f64 };

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
            if best.is_some() { break Some(best.unwrap().0) }
            else { break None }
        }
    }
}

fn find_clone_score(level: &Level, drone: &Drone, pos: &Point) -> f64 {
    if level.bonuses.get(pos) == Some(&Bonus::CLONE) { 1. } else { 0. }
}

fn explore_clone(level: &Level, drone: &Drone, drone_idx: usize) -> Option<VecDeque<Action>> {
    if drone_idx == 0
       && level.bonuses.values().any(|&b| b == Bonus::CLONE)
       && get_or(&level.collected, &Bonus::CLONE, 0) == 0 {
        explore(level, drone, find_clone_score)
    } else {
        None
    }
}

fn find_spawn_score(level: &Level, drone: &Drone, pos: &Point) -> f64 {
    if level.spawns.contains(pos) { 1.} else { 0. }
}

fn explore_spawn(level: &Level, drone: &Drone, drone_idx: usize) -> Option<VecDeque<Action>> {
    if drone_idx == 0 && get_or(&level.collected, &Bonus::CLONE, 0) > 0 {
        explore(level, drone, find_spawn_score)
    } else {
        None
    }
}

fn print_state(level: &Level, drones: &[Drone]) {
    println!("\x1B[2J");
    print_level(level, &drones);
    println!("Empty {} Collected {:?}", level.empty, level.collected);
    for (i, drone) in drones.iter().enumerate() {
        println!("{}: active {:?} at ({},{}) plan {:?}", i, drone.active, drone.pos.x, drone.pos.y, drone.plan);
    }
    thread::sleep(time::Duration::from_millis(DELAY));
}

fn solve(level: &mut Level, drones: &mut Vec<Drone>, interactive: bool) -> String {
    if interactive { println!("\x1B[?1049h"); }
    drones[0].wrap_bot(level);
    while level.empty > 0 {
        for drone_idx in 0..drones.len() {
            if interactive { print_state(level, drones); }

            if level.empty <= 0 { break; }

            let mut drone = &mut drones[drone_idx];
            drone.collect(level);
            drone.wear_off();
            
            if drone.plan.is_empty() {
                if let Some(clone) = drone.reduplicate(level) {
                    drones.push(clone);
                    continue;
                }

                if drone.activate_wheels(level)
                   || drone.activate_drill(level)
                   || drone.activate_hand(level)
                { continue; }

                if let Some(plan) = explore_clone(level, drone, drone_idx)
                                    .or_else(|| explore_spawn(level, drone, drone_idx))
                                    .or_else(|| explore(level, drone, max_wrapping)) {
                    drone.plan = plan;
                }
            }
            
            if let Some(action) = drone.plan.pop_front() {
                drone.act(&action, level);
            } else if get_or(&drone.active, &Bonus::WHEELS, 0) > 0 {
                drone.path += "Z";
            } else {
                panic!("Nothing to do");
            }
        }
    }
    
    if interactive {
        print_state(level, drones);
        println!("\x1B[?1049l");
    }
    
    let paths: Vec<&str> = drones.iter().map(|d| d.path.as_str()).collect();
    paths.join("#")
}

fn main() {
    let t_start = Instant::now();
    let args: Vec<String> = env::args().collect();
    let mut interactive = false;
    let mut filename: Option<&str> = None;
    
    for arg in args[1..].iter() {
        if arg == "--interactive" { interactive = true; }
        else if arg.ends_with(".desc") { filename = Some(arg); }
        else { panic!("cargo run [--interactive] <path/to/problem.desc>"); }
    }
    
    let contents = fs::read_to_string(filename.unwrap()).unwrap();
    // let mut input = String::new();
    // io::stdin().read_line(&mut input).unwrap();
    let (mut level, mut drones) = parser::parse_level(&contents);
    let solution = solve(&mut level, &mut drones, interactive);
    // let score = Regex::new(r"[A-Z]").unwrap().find_iter(&solution).count();
    let score = solution.split("#").map(|s| Regex::new(r"[A-Z]").unwrap().find_iter(s).count()).max().unwrap();

    eprintln!("{} \tscore {} \ttime {} ms", filename.unwrap(), score, t_start.elapsed().as_millis());
    
    let filename_sol = Regex::new(r"\.desc$").unwrap().replace(filename.unwrap(), ".sol");
    let mut file = File::create(filename_sol.into_owned()).unwrap();
    file.write_all(solution.as_bytes()).unwrap();

    // println!("{}", solution);
}