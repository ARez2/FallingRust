use glam::IVec2;



#[derive(Clone, Copy, PartialEq)]
pub struct Chunk {
    pub should_step: bool,
    pub should_step_next_frame: bool,
    pub topleft: IVec2,
    pub size: usize,
}


impl Chunk {
    pub fn start_step (&mut self) {
        self.should_step = self.should_step_next_frame;
        self.should_step_next_frame = false;
    }
}

