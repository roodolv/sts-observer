#[derive(Debug, Clone, PartialEq)]
pub enum Mode {
    IsWaiting(Waiting),
    IsWatching(Watching),
    IsFileIO(FileIO),
}
impl Mode {
    pub fn from(current: &Mode, new: &Mode) -> Mode {
        match (current, new) {
            // Waitingからの遷移
            (Mode::IsWaiting(_), Mode::IsWatching(_)) => Mode::IsWatching(Watching::new()),
            (Mode::IsWaiting(_), Mode::IsFileIO(_)) => Mode::IsFileIO(FileIO::new()),
            // Watchingからの遷移
            (Mode::IsWatching(_), Mode::IsWaiting(_)) => Mode::IsWaiting(Waiting::new()),
            (Mode::IsWatching(_), Mode::IsFileIO(_)) => Mode::IsFileIO(FileIO::new()),
            // FileIOからの遷移
            (Mode::IsFileIO(_), Mode::IsWaiting(_)) => Mode::IsWaiting(Waiting::new()),
            (Mode::IsFileIO(_), Mode::IsWatching(_)) => Mode::IsWatching(Watching::new()),
            _ => current.clone(),
        }
    }
}

pub struct ModeSelector {
    current_mode: Mode,
    has_target: bool,
    do_writing: bool,
    times_repeated: u16,
}
#[rustfmt::skip]
impl ModeSelector {
    pub fn new() -> Self {
        Self {
            current_mode: Mode::IsWaiting(Waiting::new()), // Waitingで開始
            has_target: false,
            do_writing: true,
            times_repeated: 0_u16,
        }
    }
    pub fn current_mode(&self) -> Mode { self.current_mode.clone() }
    pub fn switch_mode(&mut self, new_mode: &Mode) {
        self.current_mode = Mode::from(&self.current_mode, new_mode);
    }

    pub fn has_target(&self) -> bool { self.has_target }
    pub fn found_target(&mut self) { self.has_target = true; }
    pub fn reset_target(&mut self) { self.has_target = false; }

    pub fn do_writing(&self) -> bool { self.do_writing }
    pub fn turn_on_do_writing(&mut self) { self.do_writing = true; }
    pub fn turn_off_do_writing(&mut self) { self.do_writing = false; }

    pub fn times_repeated(&self) -> u16 { self.times_repeated }
    pub fn increase_times_repeated(&mut self) { self.times_repeated += 1; }
    pub fn reset_times_repeated(&mut self) { self.times_repeated = 0_u16; }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Waiting {}
impl Waiting {
    pub fn new() -> Self { Self {} }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Watching {}
impl Watching {
    pub fn new() -> Self { Self {} }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FileIO {}
impl FileIO {
    pub fn new() -> Self { Self {} }
}
