use glam::IVec2;



#[derive(Clone, Copy, PartialEq)]
pub struct Chunk {
    pub should_step: bool,
    pub should_step_next_frame: bool,
    pub topleft: IVec2,
    pub size: usize,
    num_frames_without_step: u8,
}


impl Chunk {
    pub fn new(topleft: IVec2, size: usize) -> Self {
        Chunk {
            should_step: false,
            should_step_next_frame: true,
            topleft,
            size,
            num_frames_without_step: 0,
        }
    }

    pub fn start_step (&mut self) {
        self.should_step = self.should_step_next_frame;
        if !self.should_step {
            self.num_frames_without_step += 1;

            if self.num_frames_without_step >= 200 {
                self.should_step = true;
                self.num_frames_without_step = 0;
            };
        };
        self.should_step_next_frame = false;
    }
}

