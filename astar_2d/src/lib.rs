#![feature(test)]
extern crate test;

use smallvec::{smallvec, SmallVec};
use std::cmp::Ordering;
use std::collections::BinaryHeap;

#[derive(Copy, Clone, Eq, PartialEq)]
struct FrontierItem {
    pub position: u32,
    pub cost: u32,
}

impl Ord for FrontierItem {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .cost
            .cmp(&self.cost)
            .then_with(|| self.position.cmp(&other.position))
    }
}

impl PartialOrd for FrontierItem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[inline(always)]
fn get_neighbor_coords(current: u32, grid: &Vec<u32>, width: u32) -> SmallVec<[u32; 8]> {
    let is_top = current < width;
    let is_bottom = current >= grid.len() as u32 - width;
    let x = current % width;
    let is_left = x == 0;
    let is_right = x == width - 1;
    let mut neighbors: SmallVec<[u32; 8]> = smallvec![];
    if !is_top {
        let top_index = current - width;
        if grid[top_index as usize] > 0 {
            neighbors.push(top_index)
        }
        if !is_left && grid[top_index as usize - 1] > 0 {
            neighbors.push(top_index - 1)
        }
        if !is_right && grid[top_index as usize + 1] > 0 {
            neighbors.push(top_index + 1)
        }
    }
    if !is_left && grid[current as usize - 1] > 0 {
        neighbors.push(current - 1)
    }
    if !is_right && grid[current as usize + 1] > 0 {
        neighbors.push(current + 1)
    }
    if !is_bottom {
        let bottom_index = current + width;
        if grid[bottom_index as usize] > 0 {
            neighbors.push(bottom_index)
        }
        if !is_left && grid[bottom_index as usize - 1] > 0 {
            neighbors.push(bottom_index - 1)
        }
        if !is_right && grid[bottom_index as usize + 1] > 0 {
            neighbors.push(bottom_index + 1)
        }
    }
    neighbors
}

#[inline(always)]
fn get_cardinal_neighbor_coords(current: u32, grid: &Vec<u32>, width: u32) -> SmallVec<[u32; 8]> {
    let is_top = current < width;
    let is_bottom = current >= grid.len() as u32 - width;
    let x = current % width;
    let is_left = x == 0;
    let is_right = x == width - 1;
    let mut neighbors: SmallVec<[u32; 8]> = smallvec![];
    if !is_top {
        let top_index = current - width;
        if grid[top_index as usize] > 0 {
            neighbors.push(top_index)
        }
    }
    if !is_left && grid[current as usize - 1] > 0 {
        neighbors.push(current - 1)
    }
    if !is_right && grid[current as usize + 1] > 0 {
        neighbors.push(current + 1)
    }
    if !is_bottom {
        let bottom_index = current + width;
        if grid[bottom_index as usize] > 0 {
            neighbors.push(bottom_index)
        }
    }
    neighbors
}

#[inline(always)]
fn manhattan(x1: i32, y1: i32, x2: i32, y2: i32) -> u32 {
    ((x1 - x2).abs() + (y1 - y2).abs()) as u32
}

pub fn find_path(
    start: u32,
    end: u32,
    grid: &Vec<u32>,
    width: u32,
    cardinal_directions: bool,
) -> Vec<u32> {
    let end_x = end % width;
    let end_y = end / width;
    let mut frontier = BinaryHeap::with_capacity(grid.len());
    let mut cost_so_far = vec![0; grid.len()];
    let mut came_from = vec![start; grid.len()];
    cost_so_far[start as usize] = 1;
    frontier.push(FrontierItem {
        cost: 0,
        position: start,
    });
    let neighbor_coord_getter = match cardinal_directions {
        true => get_cardinal_neighbor_coords,
        _ => get_neighbor_coords,
    };
    while !frontier.is_empty() {
        let current_position = frontier.pop().unwrap().position;
        if current_position == end {
            break;
        }
        let current_x = current_position % width;
        let current_y = current_position / width;
        let neighbor_coords = neighbor_coord_getter(current_position, grid, width);
        for idx in 0..neighbor_coords.len() {
            let neighbor = neighbor_coords[idx];
            let neighbor_cost = grid[neighbor as usize];
            let neighbor_x = neighbor % width;
            let neighbor_y = neighbor / width;
            let cost = cost_so_far[current_position as usize]
                + neighbor_cost
                + manhattan(
                    current_x as i32,
                    current_y as i32,
                    neighbor_x as i32,
                    neighbor_y as i32,
                );
            let neighbor_cost_so_far = cost_so_far[neighbor as usize];
            if neighbor_cost_so_far == 0 || cost < neighbor_cost_so_far {
                cost_so_far[neighbor as usize] = cost;
                let priority = cost
                    + manhattan(
                        end_x as i32,
                        end_y as i32,
                        neighbor_x as i32,
                        neighbor_y as i32,
                    );
                frontier.push(FrontierItem {
                    cost: priority,
                    position: neighbor,
                });
                came_from[neighbor as usize] = current_position;
            }
        }
    }
    let mut last = end;
    let mut path: Vec<u32> = Vec::new();
    loop {
        path.push(last);
        last = came_from[last as usize];
        if last == start {
            break;
        }
    }
    path.reverse();
    path
}

#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;

    fn xy_to_idx(x: u32, y: u32, width: u32) -> u32 {
        (y * width) + x
    }

    #[test]
    fn xy_to_idx_works() {
        assert_eq!(xy_to_idx(1, 1, 7), 8);
        assert_eq!(xy_to_idx(1, 2, 7), 15);
    }

    #[test]
    fn it_runs_in_a_straigh_line() {
        #[rustfmt::skip]
        let grid = vec![
            1, 1, 1, 1, 1,
            1, 1, 1, 1, 1,
            1, 1, 1, 1, 1,
            1, 1, 1, 1, 1,
            1, 1, 1, 1, 1,
        ];
        let path = find_path(0, 24, &grid, 5, false);
        assert_eq!(path, vec![6, 12, 18, 24]);
    }

    #[test]
    fn it_avoids_walls() {
        #[rustfmt::skip]
        let grid = vec![
            1, 1, 1, 1, 1, 1, 1,
            1, 1, 0, 1, 1, 0, 1,
            1, 1, 0, 0, 1, 0, 1,
            1, 1, 0, 1, 1, 0, 1,
            1, 1, 0, 0, 0, 0, 1,
            1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1, 1,
        ];
        let path = find_path(0, 48, &grid, 7, false);
        assert_eq!(path, vec![8, 15, 22, 29, 37, 45, 46, 47, 48]);
    }

    #[test]
    fn it_cuts_corners() {
        let width: u32 = 4;
        #[rustfmt::skip]
        let grid = vec![
            1, 0, 1, 1,
            1, 0, 1, 1,
            1, 0, 1, 1,
            1, 1, 1, 1,
        ];
        let path = find_path(0, 15, &grid, width, false);
        assert_eq!(
            path,
            vec![
                xy_to_idx(0, 1, width),
                xy_to_idx(0, 2, width),
                xy_to_idx(1, 3, width),
                xy_to_idx(2, 3, width),
                xy_to_idx(3, 3, width),
            ]
        );
    }

    #[test]
    fn it_doesnt_cut_corners_using_cardinal_directions() {
        let width: u32 = 4;
        #[rustfmt::skip]
        let grid = vec![
            1, 0, 1, 1,
            1, 0, 1, 1,
            1, 0, 1, 1,
            1, 1, 1, 1,
        ];
        let path = find_path(0, 15, &grid, width, true);
        assert_eq!(
            path,
            vec![
                xy_to_idx(0, 1, width),
                xy_to_idx(0, 2, width),
                xy_to_idx(0, 3, width),
                xy_to_idx(1, 3, width),
                xy_to_idx(2, 3, width),
                xy_to_idx(3, 3, width),
            ]
        );
    }

    #[bench]
    fn bench_it_avoids_walls(b: &mut Bencher) {
        #[rustfmt::skip]
        let grid = vec![
            1, 1, 1, 1, 1, 1, 1,
            1, 1, 0, 1, 1, 0, 1,
            1, 1, 0, 0, 1, 0, 1,
            1, 1, 0, 1, 1, 0, 1,
            1, 1, 0, 0, 0, 0, 1,
            1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1, 1,
        ];
        b.iter(|| find_path(0, 48, &grid, 7, false));
    }
    #[bench]
    fn bench_it_avoids_walls_cardinal(b: &mut Bencher) {
        #[rustfmt::skip]
        let grid = vec![
            1, 1, 1, 1, 1, 1, 1,
            1, 1, 0, 1, 1, 0, 1,
            1, 1, 0, 0, 1, 0, 1,
            1, 1, 0, 1, 1, 0, 1,
            1, 1, 0, 0, 0, 0, 1,
            1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1, 1,
        ];
        b.iter(|| find_path(0, 48, &grid, 7, true));
    }
}
