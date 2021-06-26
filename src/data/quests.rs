use crate::concepts::{Condition, Satisfied};

pub type Quest = [&'static str; 2];

pub trait QuestInfo {
    fn title(&self) -> &'static str;
    fn objective(&self) -> &'static str;
}

impl<const LENGTH: usize> QuestInfo for [&'static str; LENGTH]
where
    Condition<{ LENGTH >= 2 }>: Satisfied,
{
    fn title(&self) -> &'static str {
        self[0]
    }
    fn objective(&self) -> &'static str {
        self[1]
    }
}
