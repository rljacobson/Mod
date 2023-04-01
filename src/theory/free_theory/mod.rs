

pub trait Jim {
    fn get_hank_mut(&mut self) -> bool;

    fn check_burger(& mut self) {
        let hank_mut = self.get_hank_mut();
    }
}

