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
    pub fn offset(&self) -> (f32, f32) {
        let x_offset = self.start_loc.0 - self.current_loc.0;
        let y_offset = self.start_loc.1 - self.current_loc.1;
        (x_offset, y_offset)
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

#[derive(Clone, PartialEq, Copy)]
pub enum GateDragType {
    Point(PointDragData),
    Gate(GateDragData),
}

impl GateDragType {
    pub fn clone_with_point(self, point: (f32, f32)) -> Self {
        match self {
            GateDragType::Point(point_drag_data) => {
                GateDragType::Point(
                    PointDragData::clone_from_data(point, point_drag_data)
                )
            },
            GateDragType::Gate(gate_drag_data) => {
                GateDragType::Gate(
                    GateDragData::clone_from_data(
                        point,
                    gate_drag_data
                ))
            },
        }
    }
}
