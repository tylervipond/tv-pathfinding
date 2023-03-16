#![feature(test)]
extern crate test;

use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashSet};

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
pub fn get_neighbor_idxs(
    current: u32,
    grid: &Vec<u32>,
    dimensions: (u32, u32),
    up_stairs_idxs: &HashSet<u32>,
    down_stairs_idxs: &HashSet<u32>,
) -> Vec<u32> {
    let (width, height) = dimensions;
    let tile_count = width * height;
    let idx_in_level = current % tile_count;
    let is_top = idx_in_level < width;
    let is_bottom = idx_in_level >= tile_count - width;
    let x = current % width;
    let is_left = x == 0;
    let is_right = x == width - 1;
    let mut neighbors: Vec<u32> = vec![];
    if !is_top {
        let top_index = current - width;
        if grid[top_index as usize] > 0 {
            neighbors.push(top_index);
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
    let mut vertical_neighbors = vec![];
    for neighbor in &neighbors {
        if up_stairs_idxs.contains(&neighbor) {
            vertical_neighbors.push(neighbor + tile_count)
        }
        if down_stairs_idxs.contains(&neighbor) {
            vertical_neighbors.push(neighbor - tile_count)
        }
    }
    neighbors.append(&mut vertical_neighbors);
    neighbors
}

pub fn create_neighbor_idx_cache(
    grid: &Vec<u32>,
    dimensions: (u32, u32),
    up_stairs_idxs: &HashSet<u32>,
    down_stairs_idxs: &HashSet<u32>,
) -> Vec<Vec<u32>> {
    let mut neighbor_idx_cache = vec![];
    for idx in 0..grid.len() {
        let neighbors = if grid[idx] == 0 {
            vec![]
        } else {
            get_neighbor_idxs(
                idx as u32,
                grid,
                dimensions,
                up_stairs_idxs,
                down_stairs_idxs,
            )
        };
        neighbor_idx_cache.push(neighbors);
    }
    neighbor_idx_cache
}

#[inline(always)]
fn manhattan(x1: i32, y1: i32, depth1: i32, x2: i32, y2: i32, depth2: i32) -> u32 {
    ((x1 - x2).abs() + (y1 - y2).abs() + (depth1 - depth2).abs()) as u32
}

pub fn find_path(
    start: u32,
    end: u32,
    grid: &Vec<u32>,
    dimensions: (u32, u32),
    neighbors: &Vec<Vec<u32>>,
) -> Vec<u32> {
    let (width, height) = dimensions;
    let tile_count_per_floor = width * height;
    let end_x = end % width;
    let end_y = end % tile_count_per_floor / width;
    let end_depth = end / tile_count_per_floor;
    let mut frontier = BinaryHeap::with_capacity(grid.len());
    let mut cost_so_far = vec![0; grid.len()];
    let mut came_from = vec![start; grid.len()];
    cost_so_far[start as usize] = 1;
    frontier.push(FrontierItem {
        cost: 0,
        position: start,
    });
    while !frontier.is_empty() {
        let current_idx = frontier.pop().unwrap().position;
        if current_idx == end {
            break;
        }
        let current_x = current_idx % width;
        let current_y = current_idx % tile_count_per_floor / width;
        let current_depth = current_idx / tile_count_per_floor;
        let neighbor_idxs = &neighbors[current_idx as usize];
        for idx in 0..neighbor_idxs.len() {
            let neighbor = neighbor_idxs[idx];
            let neighbor_cost = grid[neighbor as usize];
            let neighbor_x = neighbor % width;
            let neighbor_y = neighbor % tile_count_per_floor / width;
            let neighbor_depth = neighbor / tile_count_per_floor;
            let cost = cost_so_far[current_idx as usize]
                + neighbor_cost
                + manhattan(
                    current_x as i32,
                    current_y as i32,
                    current_depth as i32,
                    neighbor_x as i32,
                    neighbor_y as i32,
                    neighbor_depth as i32,
                );
            let neighbor_cost_so_far = cost_so_far[neighbor as usize];
            if neighbor_cost_so_far == 0 || cost < neighbor_cost_so_far {
                cost_so_far[neighbor as usize] = cost;
                let priority = cost
                    + manhattan(
                        end_x as i32,
                        end_y as i32,
                        end_depth as i32,
                        neighbor_x as i32,
                        neighbor_y as i32,
                        neighbor_depth as i32,
                    );
                frontier.push(FrontierItem {
                    cost: priority,
                    position: neighbor,
                });
                came_from[neighbor as usize] = current_idx;
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
    fn it_runs_in_a_straight_line() {
        #[rustfmt::skip]
        let grid = vec![
            1, 1, 1, 1, 1,
            1, 1, 1, 1, 1,
            1, 1, 1, 1, 1,
            1, 1, 1, 1, 1,
            1, 1, 1, 1, 1,
        ];
        let up_stairs_idxs = HashSet::new();
        let down_stairs_idxs = HashSet::new();
        let dimensions = (5, 5);
        let neighbors =
            create_neighbor_idx_cache(&grid, dimensions, &up_stairs_idxs, &down_stairs_idxs);
        let path = find_path(0, 24, &grid, dimensions, &neighbors);
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
        let up_stairs_idxs = HashSet::new();
        let down_stairs_idxs = HashSet::new();
        let dimensions = (7, 7);
        let neighbors =
            create_neighbor_idx_cache(&grid, dimensions, &up_stairs_idxs, &down_stairs_idxs);
        let path = find_path(0, 48, &grid, dimensions, &neighbors);
        assert_eq!(path, vec![8, 15, 22, 29, 37, 45, 46, 47, 48]);
    }

    #[test]
    fn it_finds_the_path_across_floors() {
        #[rustfmt::skip]
        let grid = vec![
            1, 1, 1, 1, 1, 1, 1,
            1, 1, 0, 1, 1, 0, 1,
            1, 1, 0, 0, 1, 0, 1,
            1, 1, 0, 1, 1, 0, 1,
            1, 1, 0, 0, 0, 0, 1,
            1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1, 1,

            1, 1, 1, 1, 1, 1, 1,
            1, 1, 0, 1, 1, 0, 1,
            1, 1, 0, 0, 1, 0, 1,
            1, 1, 0, 1, 1, 0, 1,
            1, 1, 0, 0, 0, 0, 1,
            1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1, 1,

            1, 1, 1, 1, 1, 1, 1,
            1, 1, 0, 1, 1, 0, 1,
            1, 1, 0, 0, 1, 0, 1,
            1, 1, 0, 1, 1, 0, 1,
            1, 1, 0, 0, 0, 0, 1,
            1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1, 1,
        ];
        let up_stairs_idxs = HashSet::from([24, 87]);
        let down_stairs_idxs = HashSet::new();
        let dimensions = (7, 7);
        let neighbors =
            create_neighbor_idx_cache(&grid, dimensions, &up_stairs_idxs, &down_stairs_idxs);
        let path = find_path(0, 146, &grid, dimensions, &neighbors);
        assert_eq!(
            path,
            vec![1, 2, 10, 18, 73, 67, 59, 51, 57, 64, 71, 78, 86, 136, 144, 145, 146]
        );
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
        let up_stairs_idxs = HashSet::new();
        let down_stairs_idxs = HashSet::new();
        let dimensions = (4, 4);
        let neighbors =
            create_neighbor_idx_cache(&grid, dimensions, &up_stairs_idxs, &down_stairs_idxs);
        let path = find_path(0, 15, &grid, dimensions, &neighbors);

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
        let up_stairs_idxs = HashSet::new();
        let down_stairs_idxs = HashSet::new();
        let dimensions = (7, 7);
        let neighbors =
            create_neighbor_idx_cache(&grid, dimensions, &up_stairs_idxs, &down_stairs_idxs);
        b.iter(|| find_path(0, 48, &grid, dimensions, &neighbors));
    }

    #[bench]
    fn bench_it_paths_between_levels(b: &mut Bencher) {
        #[rustfmt::skip]
        let grid = vec![
            1, 1, 1, 1, 1, 1, 1,
            1, 1, 0, 1, 1, 0, 1,
            1, 1, 0, 0, 1, 0, 1,
            1, 1, 0, 1, 1, 0, 1,
            1, 1, 0, 0, 0, 0, 1,
            1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1, 1,

            1, 1, 1, 1, 1, 1, 1,
            1, 1, 0, 1, 1, 0, 1,
            1, 1, 0, 0, 1, 0, 1,
            1, 1, 0, 1, 1, 0, 1,
            1, 1, 0, 0, 0, 0, 1,
            1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1, 1,

            1, 1, 1, 1, 1, 1, 1,
            1, 1, 0, 1, 1, 0, 1,
            1, 1, 0, 0, 1, 0, 1,
            1, 1, 0, 1, 1, 0, 1,
            1, 1, 0, 0, 0, 0, 1,
            1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1, 1,
        ];
        let up_stairs_idxs = HashSet::from([24, 87]);
        let down_stairs_idxs = HashSet::new();
        let dimensions = (7, 7);
        let neighbors =
            create_neighbor_idx_cache(&grid, dimensions, &up_stairs_idxs, &down_stairs_idxs);
        b.iter(|| find_path(0, 146, &grid, dimensions, &neighbors));
    }
}
