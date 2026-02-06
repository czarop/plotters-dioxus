#[derive(Clone, PartialEq, Copy)]
pub struct GateDragData {
    start_loc: (f32, f32),
    current_loc: (f32, f32),
}

impl GateDragData {
    pub fn new(start_loc: (f32, f32), current_loc: (f32, f32)) -> Self {
        Self {
            start_loc,
            current_loc,
        }
    }

    pub fn clone_from_data(new_loc: (f32, f32), old_data: Self) -> Self {
        Self {
            start_loc: old_data.start_loc,
            current_loc: new_loc,
        }
    }

    pub fn start_loc(&self) -> (f32, f32) {
        self.start_loc
    }
    pub fn current_loc(&self) -> (f32, f32) {
        self.current_loc
    }
}

#[derive(Clone, PartialEq, Copy)]
pub struct PointDragData {
    point_index: usize,
    loc: (f32, f32),
}

impl PointDragData {
    pub fn new(point_index: usize, loc: (f32, f32)) -> Self {
        Self { point_index, loc }
    }
    pub fn clone_from_data(new_loc: (f32, f32), old_data: Self) -> Self {
        Self {
            point_index: old_data.point_index,
            loc: new_loc,
        }
    }
    pub fn point_index(&self) -> usize {
        self.point_index
    }
    pub fn loc(&self) -> (f32, f32) {
        self.loc
    }
}
