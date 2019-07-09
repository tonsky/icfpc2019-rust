#![allow(dead_code, unused_imports, unused_variables)]

mod parser;

use std::{env, fs, io, thread, time};
use std::cmp::{min, max};
use std::fs::{File};
use std::io::prelude::*;
use std::collections::{HashSet, HashMap, VecDeque};
use std::time::{Instant};
use std::sync::{Mutex, Arc};
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
enum Action { UP, RIGHT, DOWN, LEFT, JUMP0, JUMP1, JUMP2 }

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

fn hand_blocker(p: &Point) -> Vec<Point> {
    match p {
        Point{x: 0, y: 0}  => vec![Point::new(0, 0)],
        Point{x: 1, y: -1} => vec![Point::new(1, -1)],
        Point{x: 1, y: 0}  => vec![Point::new(1, 0)],
        Point{x: 1, y: 1}  => vec![Point::new(1, 1)],
        Point{x: 1, y: 2} => vec![Point{x: 0, y: 1}, Point{x: 0, y: 2}, Point{x: 1, y: 1}, Point{x: 1, y: 2}, Point{x: 1, y: 3}],
        Point{x: 1, y: 3} => vec![Point{x: 0, y: 1}, Point{x: 0, y: 2}, Point{x: 1, y: 2}, Point{x: 1, y: 3}, Point{x: 1, y: 4}],
        Point{x: 1, y: 4} => vec![Point{x: 0, y: 1}, Point{x: 0, y: 2}, Point{x: 0, y: 3}, Point{x: 1, y: 2}, Point{x: 1, y: 3}, Point{x: 1, y: 4}, Point{x: 1, y: 5}],
        Point{x: 1, y: 5} => vec![Point{x: 0, y: 1}, Point{x: 0, y: 2}, Point{x: 0, y: 3}, Point{x: 1, y: 3}, Point{x: 1, y: 4}, Point{x: 1, y: 5}, Point{x: 1, y: 6}],
        Point{x: 1, y: 6} => vec![Point{x: 0, y: 1}, Point{x: 0, y: 2}, Point{x: 0, y: 3}, Point{x: 0, y: 4}, Point{x: 1, y: 3}, Point{x: 1, y: 4}, Point{x: 1, y: 5}, Point{x: 1, y: 6}, Point{x: 1, y: 7}],
        Point{x: 1, y: 7} => vec![Point{x: 0, y: 1}, Point{x: 0, y: 2}, Point{x: 0, y: 3}, Point{x: 0, y: 4}, Point{x: 1, y: 4}, Point{x: 1, y: 5}, Point{x: 1, y: 6}, Point{x: 1, y: 7}, Point{x: 1, y: 8}],
        Point{x: 1, y: 8} => vec![Point{x: 0, y: 1}, Point{x: 0, y: 2}, Point{x: 0, y: 3}, Point{x: 0, y: 4}, Point{x: 0, y: 5}, Point{x: 1, y: 4}, Point{x: 1, y: 5}, Point{x: 1, y: 6}, Point{x: 1, y: 7}, Point{x: 1, y: 8}, Point{x: 1, y: 9}],
        Point{x: 1, y: 9} => vec![Point{x: 0, y: 1}, Point{x: 0, y: 2}, Point{x: 0, y: 3}, Point{x: 0, y: 4}, Point{x: 0, y: 5}, Point{x: 1, y: 5}, Point{x: 1, y: 6}, Point{x: 1, y: 7}, Point{x: 1, y: 8}, Point{x: 1, y: 9}, Point{x: 1, y: 10}],
        Point{x: 1, y: 10} => vec![Point{x: 0, y: 1}, Point{x: 0, y: 2}, Point{x: 0, y: 3}, Point{x: 0, y: 4}, Point{x: 0, y: 5}, Point{x: 0, y: 6}, Point{x: 1, y: 5}, Point{x: 1, y: 6}, Point{x: 1, y: 7}, Point{x: 1, y: 8}, Point{x: 1, y: 9}, Point{x: 1, y: 10}, Point{x: 1, y: 11}],
        Point{x: 1, y: 11} => vec![Point{x: 0, y: 1}, Point{x: 0, y: 2}, Point{x: 0, y: 3}, Point{x: 0, y: 4}, Point{x: 0, y: 5}, Point{x: 0, y: 6}, Point{x: 1, y: 6}, Point{x: 1, y: 7}, Point{x: 1, y: 8}, Point{x: 1, y: 9}, Point{x: 1, y: 10}, Point{x: 1, y: 11}, Point{x: 1, y: 12}],
        Point{x: 1, y: 12} => vec![Point{x: 0, y: 1}, Point{x: 0, y: 2}, Point{x: 0, y: 3}, Point{x: 0, y: 4}, Point{x: 0, y: 5}, Point{x: 0, y: 6}, Point{x: 0, y: 7}, Point{x: 1, y: 6}, Point{x: 1, y: 7}, Point{x: 1, y: 8}, Point{x: 1, y: 9}, Point{x: 1, y: 10}, Point{x: 1, y: 11}, Point{x: 1, y: 12}, Point{x: 1, y: 13}],
        Point{x: 1, y: 13} => vec![Point{x: 0, y: 1}, Point{x: 0, y: 2}, Point{x: 0, y: 3}, Point{x: 0, y: 4}, Point{x: 0, y: 5}, Point{x: 0, y: 6}, Point{x: 0, y: 7}, Point{x: 1, y: 7}, Point{x: 1, y: 8}, Point{x: 1, y: 9}, Point{x: 1, y: 10}, Point{x: 1, y: 11}, Point{x: 1, y: 12}, Point{x: 1, y: 13}, Point{x: 1, y: 14}],
        Point{x: 1, y: 14} => vec![Point{x: 0, y: 1}, Point{x: 0, y: 2}, Point{x: 0, y: 3}, Point{x: 0, y: 4}, Point{x: 0, y: 5}, Point{x: 0, y: 6}, Point{x: 0, y: 7}, Point{x: 0, y: 8}, Point{x: 1, y: 7}, Point{x: 1, y: 8}, Point{x: 1, y: 9}, Point{x: 1, y: 10}, Point{x: 1, y: 11}, Point{x: 1, y: 12}, Point{x: 1, y: 13}, Point{x: 1, y: 14}, Point{x: 1, y: 15}],
        Point{x: 1, y: 15} => vec![Point{x: 0, y: 1}, Point{x: 0, y: 2}, Point{x: 0, y: 3}, Point{x: 0, y: 4}, Point{x: 0, y: 5}, Point{x: 0, y: 6}, Point{x: 0, y: 7}, Point{x: 0, y: 8}, Point{x: 1, y: 8}, Point{x: 1, y: 9}, Point{x: 1, y: 10}, Point{x: 1, y: 11}, Point{x: 1, y: 12}, Point{x: 1, y: 13}, Point{x: 1, y: 14}, Point{x: 1, y: 15}, Point{x: 1, y: 16}],
        Point{x: 1, y: 16} => vec![Point{x: 0, y: 1}, Point{x: 0, y: 2}, Point{x: 0, y: 3}, Point{x: 0, y: 4}, Point{x: 0, y: 5}, Point{x: 0, y: 6}, Point{x: 0, y: 7}, Point{x: 0, y: 8}, Point{x: 0, y: 9}, Point{x: 1, y: 8}, Point{x: 1, y: 9}, Point{x: 1, y: 10}, Point{x: 1, y: 11}, Point{x: 1, y: 12}, Point{x: 1, y: 13}, Point{x: 1, y: 14}, Point{x: 1, y: 15}, Point{x: 1, y: 16}, Point{x: 1, y: 17}],
        Point{x: 1, y: 17} => vec![Point{x: 0, y: 1}, Point{x: 0, y: 2}, Point{x: 0, y: 3}, Point{x: 0, y: 4}, Point{x: 0, y: 5}, Point{x: 0, y: 6}, Point{x: 0, y: 7}, Point{x: 0, y: 8}, Point{x: 0, y: 9}, Point{x: 1, y: 9}, Point{x: 1, y: 10}, Point{x: 1, y: 11}, Point{x: 1, y: 12}, Point{x: 1, y: 13}, Point{x: 1, y: 14}, Point{x: 1, y: 15}, Point{x: 1, y: 16}, Point{x: 1, y: 17}, Point{x: 1, y: 18}],
        Point{x: 1, y: 18} => vec![Point{x: 0, y: 1}, Point{x: 0, y: 2}, Point{x: 0, y: 3}, Point{x: 0, y: 4}, Point{x: 0, y: 5}, Point{x: 0, y: 6}, Point{x: 0, y: 7}, Point{x: 0, y: 8}, Point{x: 0, y: 9}, Point{x: 0, y: 10}, Point{x: 1, y: 9}, Point{x: 1, y: 10}, Point{x: 1, y: 11}, Point{x: 1, y: 12}, Point{x: 1, y: 13}, Point{x: 1, y: 14}, Point{x: 1, y: 15}, Point{x: 1, y: 16}, Point{x: 1, y: 17}, Point{x: 1, y: 18}, Point{x: 1, y: 19}],
        Point{x: 1, y: 19} => vec![Point{x: 0, y: 1}, Point{x: 0, y: 2}, Point{x: 0, y: 3}, Point{x: 0, y: 4}, Point{x: 0, y: 5}, Point{x: 0, y: 6}, Point{x: 0, y: 7}, Point{x: 0, y: 8}, Point{x: 0, y: 9}, Point{x: 0, y: 10}, Point{x: 1, y: 10}, Point{x: 1, y: 11}, Point{x: 1, y: 12}, Point{x: 1, y: 13}, Point{x: 1, y: 14}, Point{x: 1, y: 15}, Point{x: 1, y: 16}, Point{x: 1, y: 17}, Point{x: 1, y: 18}, Point{x: 1, y: 19}, Point{x: 1, y: 20}],
        _ => unimplemented!()
    }
}

type Zone = u8;
const UNDECIDED_ZONE: Zone = !0;
fn zone_char(zone: Zone) -> char {
    if zone < UNDECIDED_ZONE { (65 + zone) as char }
    else { '-' }
}

pub struct Drone {
    pos:    Point,
    hands:  Vec<Point>,
    wheels: usize,
    drill:  usize,
    path:   String,
    plan:   VecDeque<Action>,
    zone:   Zone
}

impl Drone {
    fn new(pos: Point) -> Drone {
        Drone { pos,
                hands:  vec![Point::new(0,0), Point::new(1,-1), Point::new(1,0), Point::new(1,1)],
                wheels: 0,
                drill:  0,
                path:   String::new(),
                plan:   VecDeque::new(),
                zone:   UNDECIDED_ZONE}
    }

    fn wrap_bot(&self, level: &mut Level) {
        let mut to_wrap: HashSet<Point> = HashSet::new();
        would_wrap(level, self, &self.pos, &mut to_wrap);
        for p in to_wrap {
            level.wrap_cell(p.x, p.y);
        }
    }

    fn choose_zone(&mut self, taken: &[u8], level: &Level) -> bool {
        if self.zone == UNDECIDED_ZONE || level.zones_empty[self.zone as usize] == 0 {
            let not_empty:  Vec<u8> = (0..level.zones_empty.len() as u8).filter(|&z| level.zones_empty[z as usize] > 0).collect();
            let not_taken:  Vec<u8> = not_empty.iter().cloned().filter(|&z| taken.iter().all(|&t| t != z)).collect();
            let looking_in: Vec<u8> = if not_taken.len() > 0 { not_taken } else { not_empty };
            let rate = |level: &Level, drone: &Drone, pos: &Point| {
                if level.get_cell(pos.x, pos.y) == Cell::EMPTY && looking_in.contains(&level.get_zone(pos.x, pos.y)) { 1. }
                else { 0. }
            };

            if let Some((plan, pos, _)) = explore_impl(level, self, rate) {
                self.zone = level.get_zone(pos.x, pos.y);
                self.plan = plan;
            } else {
                panic!("No zone left to choose")
            }
            true
        } else {
            false
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
        if self.wheels > 0 { self.wheels -= 1; }
        if self.drill > 0 { self.drill -= 1; }
    }

    fn has_space(&self, level: &Level) -> bool {
        (1..5).all(   |i| level.valid(self.pos.x, self.pos.y+i) && level.get_cell(self.pos.x, self.pos.y+i) != Cell::BLOCKED)
        || (1..5).all(|i| level.valid(self.pos.x, self.pos.y-i) && level.get_cell(self.pos.x, self.pos.y-i) != Cell::BLOCKED)
        || (1..5).all(|i| level.valid(self.pos.x+i, self.pos.y) && level.get_cell(self.pos.x+i, self.pos.y) != Cell::BLOCKED)
        || (1..5).all(|i| level.valid(self.pos.x-i, self.pos.y) && level.get_cell(self.pos.x-i, self.pos.y) != Cell::BLOCKED)
    }

    fn activate_wheels(&mut self, level: &mut Level) -> bool {
        if get_or(&level.collected, &Bonus::WHEELS, 0) > 0
           && self.wheels == 0
           && self.has_space(level) {
            update(&mut level.collected, Bonus::WHEELS, -1);
            self.wheels = 51;
            self.path += "F";
            true
        } else { false }
    }

    fn activate_drill(&mut self, level: &mut Level) -> bool {
        if get_or(&level.collected, &Bonus::DRILL, 0) > 0
           && self.drill == 0 {
            update(&mut level.collected, Bonus::DRILL, -1);
            self.drill = 31;
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

    fn set_beakon(&mut self, level: &mut Level) -> bool {
        if get_or(&level.collected, &Bonus::TELEPORT, 0) > 0
           && level.beakons.iter().all(|b| (b.x - self.pos.x).abs() + (b.y - self.pos.y).abs() >= 50)
        {
            update(&mut level.collected, Bonus::TELEPORT, -1);
            self.path += "R";
            level.beakons.push(self.pos);
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
        let wheels = self.wheels > 0;
        let drill = self.drill > 0;
        if let Some((pos, new_wrapped, new_drilled)) = step(level, self, &self.pos, action, wheels, drill, &HashSet::new()) {
            self.pos = pos;
            match action {
                Action::UP    => self.path += "W",
                Action::DOWN  => self.path += "S",
                Action::LEFT  => self.path += "A",
                Action::RIGHT => self.path += "D",
                Action::JUMP0 => self.path += &format!("T({},{})", level.beakons[0].x, level.beakons[0].y),
                Action::JUMP1 => self.path += &format!("T({},{})", level.beakons[1].x, level.beakons[1].y),
                Action::JUMP2 => self.path += &format!("T({},{})", level.beakons[2].x, level.beakons[2].y)
            };
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
    grid:        Vec<Cell>,
    weights:     Vec<u8>,
    zones:       Vec<Zone>,
    width:       isize,
    height:      isize,
    empty:       usize,
    zones_empty: Vec<usize>,
    spawns:      HashSet<Point>,
    beakons:     Vec<Point>,
    bonuses:     HashMap<Point, Bonus>,
    collected:   HashMap<Bonus, usize>
}

impl Level {
    fn grid_idx(&self, x: isize, y: isize) -> usize {
        (x + y * self.width) as usize
    }

    fn get_cell(&self, x: isize, y: isize) -> Cell {
        assert!(x >= 0 && x < self.width && y >= 0 && y < self.height);
        self.grid[self.grid_idx(x, y)]
    }

    fn get_zone(&self, x: isize, y: isize) -> u8 {
        assert!(x >= 0 && x < self.width && y >= 0 && y < self.height);
        self.zones[self.grid_idx(x, y)]
    }

    fn wrap_cell(&mut self, x: isize, y: isize) {
        assert!(x >= 0 && x < self.width && y >= 0 && y < self.height);
        assert!(self.get_cell(x, y) == Cell::EMPTY);
        let idx = self.grid_idx(x, y);
        self.empty -= 1;
        let zone = self.zones[idx];
        if zone < 255 {
            self.zones_empty[zone as usize] -= 1;
        }
        self.grid[idx] = Cell::WRAPPED;
    }

    fn drill_cell(&mut self, x: isize, y: isize) {
        assert!(x >= 0 && x < self.width && y >= 0 && y < self.height);
        assert!(self.get_cell(x, y) == Cell::BLOCKED);
        let idx = self.grid_idx(x, y);
        self.grid[idx] = Cell::WRAPPED;
    }

    fn valid(&self, x: isize, y: isize) -> bool {
        x >= 0 && x < self.width as isize && y >= 0 && y < self.height as isize
    }

    fn walkable(&self, x: isize, y: isize) -> bool {
        self.valid(x, y) && self.get_cell(x, y) != Cell::BLOCKED
    }
}

fn print_level(level: &Level, drones: &[Drone]) {
    let ymin = max(0, min(drones[0].pos.y - 25, level.height - 50));
    let ymax = min(max(drones[0].pos.y + 25, 50), level.height);
    let xmin = max(0, min(drones[0].pos.x - 50, level.width - 100));
    let xmax = min(max(drones[0].pos.x + 50, 100), level.width);

    for y in (ymin..ymax).rev() {
        for x in xmin..xmax {
            let point = Point::new(x, y);

            let bg = if drones.iter().find(|d| d.hands.iter().find(|h| {
                       d.pos.x + h.x == x as isize && d.pos.y + h.y == y as isize && is_reaching(level, &d.pos, &h)
                     }).is_some()).is_some() { "\x1B[48;5;202m" }
            else if level.bonuses.get(&point).is_some() { "\x1B[48;5;33m\x1B[38;5;15m" }
            else if level.spawns.contains(&point)       { "\x1B[48;5;33m\x1B[38;5;15m" }
            else if level.beakons.contains(&point)      { "\x1B[48;5;33m\x1B[38;5;15m" }
            else {
                match level.get_cell(x, y) {
                    Cell::EMPTY   => { "\x1B[48;5;252m" }
                    Cell::BLOCKED => { "\x1B[48;5;240m" }
                    Cell::WRAPPED => { "\x1B[48;5;227m" }
                }
            };

            let char = if let Some((idx, _)) = drones.iter().enumerate().find(|(idx, d)| d.hands.iter().find(|h| {
                       d.pos.x + h.x == x as isize && d.pos.y + h.y == y as isize && is_reaching(level, &d.pos, &h)
                     }).is_some()) { idx.to_string() }
            else if let Some(bonus) = level.bonuses.get(&point) {
                    String::from(match bonus {
                        Bonus::HAND     => { "B" }
                        Bonus::WHEELS   => { "F" }
                        Bonus::DRILL    => { "L" }
                        Bonus::TELEPORT => { "R" }
                        Bonus::CLONE    => { "C" }
                    })
                } else if level.spawns.contains(&point) {
                    String::from("X")
                } else if let Some(beakon_idx) = level.beakons.iter().position(|&x| x == point) {
                    beakon_idx.to_string()
                } else {
                    zone_char(level.get_zone(x, y)).to_string()
                };
            print!("{}{}\x1B[0m", bg, char);
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
    if level.get_zone(pos.x, pos.y) != drone.zone { 0. }
    else if level.bonuses.contains_key(pos) { 100. }
    else {
        let mut wrapped: HashSet<Point> = HashSet::new();
        would_wrap(level, drone, pos, &mut wrapped);
        wrapped.iter().map(|p| 1.0_f64.max(level.weights[level.grid_idx(p.x, p.y)] as f64)).sum()
    }
}

fn is_reaching(level: &Level, from: &Point, hand: &Point) -> bool {
    hand_blocker(hand).iter().all(|p| level.walkable(from.x+p.x, from.y+p.y))
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

fn step_move(level: &Level, drone: &Drone, from: &Point, dx: isize, dy: isize, wheels: bool, drill: bool, drilled: &HashSet<Point>) -> Option<(Point, HashSet<Point>, HashSet<Point>)>
{
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

fn step_jump(level: &Level, drone: &Drone, beakon_idx: usize) -> Option<(Point, HashSet<Point>, HashSet<Point>)>
{
    if beakon_idx < level.beakons.len() {
        let to = level.beakons[beakon_idx];
        let mut new_wrapped = HashSet::new();
        would_wrap(level, drone, &to, &mut new_wrapped);
        Some((to, new_wrapped, HashSet::new()))
    } else {
        None
    }
}

fn step(level: &Level, drone: &Drone, from: &Point, action: &Action, wheels: bool, drill: bool, drilled: &HashSet<Point>) -> Option<(Point, HashSet<Point>, HashSet<Point>)> {
    match action {
        Action::LEFT  => step_move(level, drone, from, -1,  0, wheels, drill, drilled),
        Action::RIGHT => step_move(level, drone, from,  1,  0, wheels, drill, drilled),
        Action::UP    => step_move(level, drone, from,  0,  1, wheels, drill, drilled),
        Action::DOWN  => step_move(level, drone, from,  0, -1, wheels, drill, drilled),
        Action::JUMP0 => step_jump(level, drone, 0),
        Action::JUMP1 => step_jump(level, drone, 1),
        Action::JUMP2 => step_jump(level, drone, 2)
    }
}

fn explore<F>(level: &Level, drone: &Drone, rate: F) -> Option<VecDeque<Action>>
    where F: Fn(&Level, &Drone, &Point) -> f64
{
    explore_impl(level, drone, rate).and_then(|(path, _, _)| Some(path))
}

fn explore_impl<F>(level: &Level, drone: &Drone, rate: F) -> Option<(VecDeque<Action>, Point, f64)>
    where F: Fn(&Level, &Drone, &Point) -> f64
{
    let mut seen: HashSet<Point> = HashSet::new();
    let mut queue: VecDeque<Plan> = VecDeque::with_capacity(100);
    let mut best: Option<(VecDeque<Action>, Point, f64)> = None;
    let mut max_len = 5;
    queue.push_back(Plan{plan:    VecDeque::new(),
                         pos:     drone.pos,
                         wheels:  drone.wheels,
                         drill:   drone.drill,
                         drilled: HashSet::new() });
    loop {
        if let Some(Plan{plan, pos, wheels, drill, drilled}) = queue.pop_front() {
            if plan.len() >= max_len {
                if best.is_some() {
                    break best
                } else {
                    max_len += 5;
                }
            }

            let score = if plan.is_empty() { 0. } else { rate(level, drone, &pos) / plan.len() as f64 };

            if best.is_some() {
                if score > best.as_ref().unwrap().2 { best = Some((plan.clone(), pos, score)); }
            } else {
                if score > 0. { best = Some((plan.clone(), pos, score)); }
            }

            for action in &[Action::LEFT, Action::RIGHT, Action::UP, Action::DOWN, Action::JUMP0, Action::JUMP1, Action::JUMP2] {
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
        } else { break best }
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
    println!("Empty {:?} Collected {:?}", level.zones_empty, level.collected);
    for (i, drone) in drones.iter().enumerate() {
        let plan: Vec<_> = drone.plan.iter().map(|action| match action { Action::UP => "↑", Action::DOWN => "↓", Action::LEFT => "←", Action::RIGHT => "→", Action::JUMP0 => "T0", Action::JUMP1 => "T1", Action::JUMP2 => "T2", }).collect();
        println!("{}: zone {} wheels {} drill {} at ({},{}) plan {}", i, zone_char(drone.zone), drone.wheels, drone.drill, drone.pos.x, drone.pos.y, plan.join(""));
    }
    thread::sleep(time::Duration::from_millis(DELAY));
}

fn solve_impl(level: &mut Level, drones: &mut Vec<Drone>, interactive: bool) -> String {
    if interactive { println!("\x1B[?1049h"); }
    drones[0].wrap_bot(level);
    while level.empty > 0 {
        if interactive { print_state(level, drones); }
        for drone_idx in 0..drones.len() {
            if level.empty <= 0 { break; }

            let taken: Vec<_> = drones.iter().map(|d| d.zone).collect();
            let mut drone = &mut drones[drone_idx];
            drone.collect(level);
            drone.wear_off();
            drone.choose_zone(&taken, level);

            if drone.plan.is_empty() {
                if let Some(clone) = drone.reduplicate(level) {
                    drones.push(clone);
                    continue;
                }

                if drone.activate_wheels(level)
                   || drone.activate_drill(level)
                   || drone.activate_hand(level)
                   || drone.set_beakon(level)
                { continue; }

                if let Some(plan) = explore_clone(level, drone, drone_idx)
                                    .or_else(|| explore_spawn(level, drone, drone_idx))
                                    .or_else(|| explore(level, drone, max_wrapping)) {
                    drone.plan = plan;
                }
            }

            if let Some(action) = drone.plan.pop_front() {
                drone.act(&action, level);
            } else if drone.wheels > 0 {
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

fn solve(filename: &str, interactive: bool) {
    if let Ok(contents) = fs::read_to_string(filename) {
        let t_start = Instant::now();
        let (mut level, mut drones) = parser::parse_level(&contents);
        let solution = solve_impl(&mut level, &mut drones, interactive);
        let score = solution.split("#").map(|s| Regex::new(r"[A-Z]").unwrap().find_iter(s).count()).max().unwrap();
        println!("{} \tscore {} \ttime {} ms", filename, score, t_start.elapsed().as_millis());

        let filename_sol = Regex::new(r"\.desc$").unwrap().replace(filename, ".sol");
        let mut file = File::create(filename_sol.into_owned()).unwrap();
        file.write_all(solution.as_bytes()).unwrap();
    } else {
        println!("Failed to read {}", filename);
    }
}

fn doall<T, F>(tasks: VecDeque<T>, threads: usize, f: F)
    where F: Fn(T),
          F: Copy + Send + 'static,
          T: Send + 'static
{
    let m_queue = Arc::new(Mutex::new(tasks));
    let mut handles = vec![];

    for i in 0..threads {
        let m_queue = Arc::clone(&m_queue);
        let handle = thread::spawn(move || loop {
            let o_task = {
                let mut queue = m_queue.lock().unwrap();
                queue.pop_front()
            };
            if let Some(task) = o_task {
                f(task);
            } else {
                break;
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
}

fn main() {
    let t_start = Instant::now();
    let args: Vec<String> = env::args().collect();
    let threads_re = Regex::new(r"--threads=([1-9][0-9]*)").unwrap();
    let mut interactive = false;
    let mut threads = 1;
    let mut filenames: VecDeque<String> = VecDeque::new();

    for arg in args[1..].iter() {
        if arg == "--interactive" {
            interactive = true;
        } else if let Some(caps) = threads_re.captures(arg) {
            threads = caps.get(1).unwrap().as_str().parse::<isize>().unwrap() as usize;
        } else if arg.ends_with(".desc") {
            filenames.push_back(arg.clone());
        } else {
            panic!("cargo run --release [--interactive] [--threads=N] <path/to/problem.desc>");
        }
    }

    let tasks = filenames.len();
    doall(filenames, threads, move |f| solve(&f, interactive));
    if tasks > 1 {
        println!("Finished {} tasks in {} ms", tasks, t_start.elapsed().as_millis());
    }
}