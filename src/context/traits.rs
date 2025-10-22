pub trait Fork {
    #[must_use]
    fn fork(&self) -> Self;
}

pub trait Update {
    fn update_from(&mut self, other: Self);
}

pub trait Join: Sized {
    fn join(&mut self, others: Box<[Self]>);
}
