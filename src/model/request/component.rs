use crate::concepts::{Condition, Satisfied};

pub enum Component {
    Buttons(Buttons),
    #[allow(dead_code)]
    SelectMenu(Vec<SelectMenuOption>),
}

#[allow(dead_code)]
pub struct SelectMenuOption {
    description: String,
    label: String,
    value: String,
}

pub struct Buttons {
    buttons: Vec<serenity::builder::CreateButton>,
}

impl Buttons {
    pub fn new<const N: usize>(buttons: &[serenity::builder::CreateButton; N]) -> Buttons
    where
        Condition<{ N <= 5 }>: Satisfied,
    {
        Buttons {
            buttons: buttons.to_vec(),
        }
    }
}

impl IntoIterator for Buttons {
    type Item = serenity::builder::CreateButton;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.buttons.into_iter()
    }
}
