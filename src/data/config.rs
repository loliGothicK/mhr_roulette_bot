use crate::model::response::Choices;
use serde_derive::{Deserialize, Serialize};
use serenity::model::prelude::User;
use std::collections::HashSet;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub members: HashSet<User>,
    pub settings: Settings,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Settings {
    pub range: Range,
    pub target: Target,
    pub excluded: Excluded,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Range {
    pub lower: usize,
    pub upper: usize,
}

impl Range {
    pub fn as_pretty_string(&self) -> String {
        format!("Range = [★{} ~ ★{}]", self.lower, self.upper)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Target {
    pub quest: HashSet<String>,
    pub monster: HashSet<String>,
    pub weapon: HashSet<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Excluded {
    pub quest: HashSet<String>,
    pub monster: HashSet<String>,
    pub weapon: HashSet<String>,
}

pub trait Pick {
    fn pick(&self, choice: &Choices) -> &HashSet<String>;
    fn pick_mut(&mut self, choice: &Choices) -> &mut HashSet<String>;
}

impl Pick for Target {
    fn pick(&self, choice: &Choices) -> &HashSet<String> {
        match choice {
            Choices::Quest => &self.quest,
            Choices::Monster => &self.monster,
            Choices::Weapon => &self.weapon,
        }
    }
    fn pick_mut(&mut self, choice: &Choices) -> &mut HashSet<String> {
        match choice {
            Choices::Quest => &mut self.quest,
            Choices::Monster => &mut self.monster,
            Choices::Weapon => &mut self.weapon,
        }
    }
}

impl Pick for Excluded {
    fn pick(&self, choice: &Choices) -> &HashSet<String> {
        match choice {
            Choices::Quest => &self.quest,
            Choices::Monster => &self.monster,
            Choices::Weapon => &self.weapon,
        }
    }
    fn pick_mut(&mut self, choice: &Choices) -> &mut HashSet<String> {
        match choice {
            Choices::Quest => &mut self.quest,
            Choices::Monster => &mut self.monster,
            Choices::Weapon => &mut self.weapon,
        }
    }
}
