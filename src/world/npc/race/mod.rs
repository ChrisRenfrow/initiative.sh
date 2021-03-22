use std::convert::TryFrom;
use std::fmt;

use rand::Rng;

use super::{Age, Gender, Npc, Size};
use crate::command::Noun;

mod human;
mod warforged;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum Race {
    Human,
    Warforged,
}

trait RaceGenerate {
    fn regenerate(rng: &mut impl Rng, npc: &mut Npc) {
        npc.gender.replace_with(|_| Self::gen_gender(rng));
        npc.age.replace_with(|_| Self::gen_age(rng));

        if let (Some(gender), Some(age)) = (&npc.gender.value, &npc.age.value) {
            npc.name.replace_with(|_| Self::gen_name(rng, age, gender));
            npc.size.replace_with(|_| Self::gen_size(rng, age, gender));
        }
    }

    fn gen_gender(rng: &mut impl Rng) -> Gender;

    fn gen_age(rng: &mut impl Rng) -> Age;

    fn gen_name(rng: &mut impl Rng, age: &Age, gender: &Gender) -> String;

    fn gen_size(rng: &mut impl Rng, age: &Age, gender: &Gender) -> Size;
}

pub fn regenerate(rng: &mut impl Rng, npc: &mut Npc) {
    if let Some(race) = npc.race.value {
        match race {
            Race::Human => human::Race::regenerate(rng, npc),
            Race::Warforged => warforged::Race::regenerate(rng, npc),
        }
    }
}

impl TryFrom<Noun> for Race {
    type Error = ();

    fn try_from(noun: Noun) -> Result<Self, Self::Error> {
        match noun {
            Noun::Human => Ok(Race::Human),
            Noun::Warforged => Ok(Race::Warforged),
            _ => Err(()),
        }
    }
}

impl fmt::Display for Race {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Human => write!(f, "human"),
            Self::Warforged => write!(f, "warforged"),
        }
    }
}
