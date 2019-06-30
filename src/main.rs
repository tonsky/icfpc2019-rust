#![allow(dead_code, unused_imports, unused_variables)]

mod parser;

use std::{env, fs, io, thread, time};
use std::collections::{HashSet, HashMap, VecDeque};

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
enum Bonus { HAND, WHEELS, DRILL, TELEPORT, CLONE, SPAWN, BEACON }

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

    fn act(&mut self, action: Action, level: &Level) {
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
            self.path += match action {
                Action::LEFT  => { "A" }
                Action::RIGHT => { "D" }
                Action::UP    => { "W" }
                Action::DOWN  => { "S" }
            };
        }
    }
}

pub struct Level {
    grid:    Vec<Cell>,
    width:   isize,
    height:  isize,
    empty:   usize,
    bonuses: HashMap<Point, Bonus>,
    picked:  HashMap<Bonus, usize>
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
                        Bonus::SPAWN    => { "X" }
                        Bonus::TELEPORT => { "R" }
                        Bonus::BEACON   => { "T" }
                        Bonus::CLONE    => { "C" }
                    })
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

fn rate(level: &Level, p: &Point) -> f64 {
    match level.get_cell(p.x, p.y) {
        Cell::EMPTY => { 1. }
        _ => { 0. }
    }
}

fn explore(level: &Level, drone: &Drone) -> Option<VecDeque<Action>> {
    let mut seen: HashSet<Point> = HashSet::new();
    let mut queue: VecDeque<Plan> = VecDeque::with_capacity(100);
    queue.push_back(Plan { plan: VecDeque::new(), pos: drone.pos, wheels: 0, drill: 0, drilled: HashSet::new() });
//    println!("AT {:?} has {:?}", drone.pos, level.get_cell(drone.pos.x, drone.pos.y));
    loop {
        if let Some(Plan{plan, pos, wheels, drill, drilled}) = queue.pop_front() {
            if !level.walkable(pos.x, pos.y) || seen.contains(&pos) { continue }
            
//            println!("RATE {:?} pos {:?} seen {} has {:?}", rate(level, &pos), pos, seen.contains(&pos), level.get_cell(pos.x, pos.y));
            
            seen.insert(pos);
            let rate = rate(level, &pos);
             
            if rate > 0. { break Some(plan) }
            for (action, dx, dy) in &[(Action::LEFT,  -1,  0),
                                      (Action::RIGHT,  1,  0),
                                      (Action::UP,     0,  1),
                                      (Action::DOWN,   0, -1)] {
                let mut plan2 = plan.clone();
                plan2.push_back(*action);
                queue.push_back(Plan {
                    plan: plan2,
                    pos: Point::new(pos.x + dx, pos.y + dy),
                    wheels,
                    drill,
                    drilled: drilled.clone()
                });
            }
        } else {
            break None
        }
    }
}

fn solve(level: &mut Level, drones: &mut Vec<Drone>, debug: bool) -> String {
    
    if debug {
        print!("\x1B[?1049h\x1B[1J");
        print(&level, &drones);
        thread::sleep(time::Duration::from_millis(DELAY));
    }
    
    let mut step = 0;
    while level.empty > 0 {
        for drone_idx in 0..drones.len() {
            if level.empty <= 0 { break; }

            let mut drone = &mut drones[drone_idx];
            mark_level(level, &drone);
            
            if drone.plan.is_empty() {
                if let Some(plan) = explore(level, drone) {
                    drone.plan = plan;
                } else { break; }
            }
            
            if let Some(action)= drone.plan.pop_front() {
                drone.act(action, level);
            } else { break; }
            
            step += 1;
            
            if debug {
                print!("\x1B[1J");
                print(level, &drones);
                println!("Step {}, empty {}", step, level.empty);
                thread::sleep(time::Duration::from_millis(DELAY));
            }
        }
    }
    
    if debug {
        print!("\x1B[?1049l");
    }
    
    let paths: Vec<&str> = drones.iter().map(|d| d.path.as_str()).collect();
    paths.join("#")
}

fn main() {
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
    println!("{}", solution);
}
