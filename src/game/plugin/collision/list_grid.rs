use super::spatial::{SpatialAccelerator, AcceleratorError};
use super::collider::AxisAlignedBoundingBox;
use bevy::ecs::entity::Entity;
use bevy::transform::components::Transform;
use bevy::math::Vec2;
use bevy::utils::HashMap;

#[derive(Eq, PartialEq, Hash, Debug, Copy, Clone)]
struct CellCoord {
    x: usize,
    y: usize
}

struct CellItem {
    entity: Entity
}

struct Iter<'a> {
    query_grid: Option<&'a ListGrid>,
    query_upper_bound: Option<CellCoord>,
    query_lower_bound: Option<CellCoord>,
    current_cell: Option<CellCoord>,
    current_vec_index: usize
}

impl <'a> Iterator for Iter<'a> {
    type Item = Entity;

    fn next(&mut self) -> Option<Self::Item> {
        let grid = match self.query_grid {
            Some(grid) => grid,
            None => return None
        };

        let lower_bound = match self.query_lower_bound {
            Some(bound) => bound,
            None => return None
        };

        let upper_bound = match self.query_upper_bound {
            Some(bound) => bound,
            None => return None
        };

        loop {
            let mut current_cell = match self.current_cell {
                Some(cell) => cell,
                None => break None
            };

            if current_cell.x > upper_bound.x {
                current_cell.x = lower_bound.x;
                current_cell.y += 1;
            }

            if current_cell.y > upper_bound.y {
                break None
            }

            self.current_cell = Some(current_cell);
            
            let index = cell_to_index(grid, current_cell);
            let cell_vec = match grid.grid.get(index) {
                Some(vec) => vec,
                None => break None
            };

            if self.current_vec_index >= cell_vec.len() {
                self.current_vec_index = 0;
                self.current_cell = Some(CellCoord {
                    x: current_cell.x + 1,
                    y: current_cell.y
                });

                continue;
            }

            break match cell_vec.get(self.current_vec_index) {
                Some(item) => {
                    self.current_vec_index += 1;
                    Some(item.entity)
                },
                None => None
            }
        }
    }
}

impl <'a> Iter<'a> {
    fn new(grid: &'a ListGrid, lower_bound: CellCoord, upper_bound: CellCoord) -> Self {
        Iter {
            query_grid: Some(grid),
            query_upper_bound: Some(upper_bound),
            query_lower_bound: Some(lower_bound.clone()),
            current_cell: Some(lower_bound),
            current_vec_index: 0
        }
    }

    fn empty() -> Self {
        Iter {
            query_grid: None,
            query_upper_bound: None,
            query_lower_bound: None,
            current_cell: None,
            current_vec_index: 0
        }
    }
}

pub struct GridSize {
    pub width: usize,
    pub height: usize
}

pub struct ListGrid {
    entity_map: HashMap<Entity, usize>,
    grid: Vec<Vec<CellItem>>,
    grid_size: GridSize,
    grid_root: Vec2,
    cell_size: Vec2
}

impl ListGrid {
    #[inline]
    pub fn new(center: Vec2, size: GridSize, cell_size: Vec2) -> Self {
        let grid_size = Vec2::new(size.width as f32, size.height as f32) * cell_size;
        let mut grid = Vec::with_capacity(size.width * size.height);

        for _ in 0..(size.width * size.height) {
            grid.push(Vec::with_capacity(1));
        }

        ListGrid {
            entity_map: HashMap::new(),
            grid: grid,
            grid_size: size,
            grid_root: center - grid_size / 2.0,
            cell_size: cell_size
        }
    }
}

impl SpatialAccelerator for ListGrid {
    fn insert(&mut self, id: Entity, pos: &Transform, aabb: &AxisAlignedBoundingBox) -> Result<(), AcceleratorError> {
        if self.entity_map.contains_key(&id) {
            return Err(AcceleratorError {
                entity: id,
                aabb: None,
                pos: None
            });
        }

        let cell_index = match pos_to_cell(&self, pos) {
            Some(cell) => cell_to_index(&self, cell),
            None => return Err(AcceleratorError {
                entity: id,
                aabb: Some(aabb.clone()),
                pos: Some(pos.clone())
            })
        };

        self.grid[cell_index].push(CellItem {
            entity: id
        });

        Ok(())
    }

    fn update(&mut self, id: Entity, pos: &Transform, aabb: &AxisAlignedBoundingBox) -> Result<(), AcceleratorError> {
        let cell_index = match self.entity_map.get(&id) {
            Some(index) => index,
            None => return self.insert(id, pos, aabb)
        };

        let new_cell_index = match pos_to_cell(&self, pos) {
            Some(cell) => cell_to_index(&self, cell),
            None => {
                return match self.remove(id) {
                    Ok(_) => Err(AcceleratorError {
                        entity: id,
                        aabb: Some(aabb.clone()),
                        pos: Some(pos.clone())
                    }),
                    Err(e) => Err(e)
                };
            }
        };

        if *cell_index != new_cell_index {
            match self.remove(id) {
                Err(e) => return Err(e),
                _ => {}
            }

            match self.insert(id, pos, aabb) {
                Err(e) => return Err(e),
                _ => {}
            }
        }

        Ok(())
    }

    fn remove(&mut self, id: Entity) -> Result<(), AcceleratorError> {
        let cell_index = match self.entity_map.remove(&id) {
            Some(index) => index,
            None => return Err(AcceleratorError {
                entity: id,
                aabb: None,
                pos: None
            })
        };

        self.grid[cell_index].retain(|item| item.entity != id);

        Ok(())
    }

    fn query<'a>(&'a self, pos: &Transform, aabb: &AxisAlignedBoundingBox) -> Box<dyn Iterator<Item = Entity> + 'a> {
        // TODO: use aabb to generate bounds (here, we assume 9)
        match pos_to_cell(&self, pos) {
            Some(cell) => Box::new(Iter::new(&self, 
                CellCoord {
                    x: cell.x.saturating_sub(1),
                    y: cell.y.saturating_sub(1)
                },
                CellCoord {
                    x: (cell.x + 1).min(self.grid_size.width - 1),
                    y: (cell.y + 1).min(self.grid_size.height - 1)
                }
            )),
            None => Box::new(Iter::empty())
        }
    }
}

#[inline]
fn pos_to_cell(grid: &ListGrid, pos: &Transform) -> Option<CellCoord> {
    let x = ((pos.translation.x - grid.grid_root.x) / grid.cell_size.x).floor();
    let y = ((pos.translation.y - grid.grid_root.y) / grid.cell_size.y).floor();

    if x < 0.0 || y < 0.0 || x >= grid.grid_size.width as f32 || y >= grid.grid_size.height as f32 {
        return None;
    }

    Some(CellCoord {
        x: x as usize,
        y: y as usize
    })
}

#[inline]
fn cell_to_index(grid: &ListGrid, cell: CellCoord) -> usize {
    cell.x + cell.y * grid.grid_size.width
}