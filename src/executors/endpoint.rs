use crate::executors::*;
use crate::model::request::Request;
use crate::model::response::{Commands, Response};
use crate::model::translate::TranslateTo;
use itertools::Itertools;

pub fn interaction_endpoint(items: &[(String, Response)]) -> anyhow::Result<Request> {
    match items {
        [first, options @ ..] => {
            if let Ok(command) = first.1.translate_to::<Commands>() {
                let option_values = options.values().iter().cloned().collect_vec();
                match command {
                    Commands::Settings => settings(&option_values),
                    Commands::Generate => generate(&option_values),
                    Commands::Statistics => statistics(options),
                    Commands::Version => Ok(version().unwrap()),
                }
            } else {
                anyhow::bail!(
                    "FATAL ERROR: Got unknown slash commands or component interactions `{:?}`",
                    first
                );
            }
        }
        [] => anyhow::bail!("FATAL ERROR: no interaction"),
    }
}

pub struct Values_<'a> {
    slice: &'a [(String, Response)],
    count: usize,
}

pub trait Values {
    fn values(&self) -> Values_<'_>;
}

impl<'a> Values for &'a [(String, Response)] {
    fn values(&self) -> Values_<'a> {
        Values_ {
            slice: self,
            count: 0,
        }
    }
}

impl<'a> Values_<'a> {
    pub fn iter(&'a mut self) -> impl Iterator<Item = &'a Response> {
        self
    }
}

impl<'a> std::iter::Iterator for Values_<'a> {
    type Item = &'a Response;

    fn next(&mut self) -> Option<Self::Item> {
        if self.count != self.slice.len() {
            self.count += 1;
            Some(&self.slice[self.count - 1].1)
        } else {
            None
        }
    }
}

pub struct Keys_<'a> {
    slice: &'a [(String, Response)],
    count: usize,
}

pub trait Keys {
    fn keys(&self) -> Keys_<'_>;
}

impl<'a> Keys for &'a [(String, Response)] {
    fn keys(&self) -> Keys_<'a> {
        Keys_ {
            slice: self,
            count: 0,
        }
    }
}

impl<'a> Keys_<'a> {
    #[allow(dead_code)]
    pub fn iter(&'a mut self) -> impl Iterator<Item = &'a String> {
        self
    }
}

impl<'a> std::iter::Iterator for Keys_<'a> {
    type Item = &'a String;

    fn next(&mut self) -> Option<Self::Item> {
        if self.count != self.slice.len() {
            self.count += 1;
            Some(&self.slice[self.count - 1].0)
        } else {
            None
        }
    }
}
