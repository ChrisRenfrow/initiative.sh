use super::{Item, ItemCategory, MagicItem, Spell};
use crate::app::{AppMeta, Autocomplete, ContextAwareParse, Runnable};
use crate::utils::CaseInsensitiveStr;
use async_trait::async_trait;
use caith::Roller;
use std::fmt;
use std::iter::repeat;

#[derive(Clone, Debug, PartialEq)]
pub enum ReferenceCommand {
    Spell(Spell),
    Spells,
    Item(Item),
    ItemCategory(ItemCategory),
    MagicItem(MagicItem),
    OpenGameLicense,
}

#[async_trait(?Send)]
impl Runnable for ReferenceCommand {
    async fn run(self, _input: &str, _app_meta: &mut AppMeta) -> Result<String, String> {
        let (output, name) = match self {
            Self::Spell(spell) => (format!("{}", spell), spell.get_name()),
            Self::Spells => (Spell::get_list().to_string(), "This listing"),
            Self::Item(item) => (format!("{}", item), item.get_name()),
            Self::ItemCategory(category) => (format!("{}", category), "This listing"),
            Self::MagicItem(magic_item) => (format!("{}", magic_item), magic_item.get_name()),
            Self::OpenGameLicense => {
                return Ok(include_str!("../../../data/ogl-1.0a.md")
                    .trim_end()
                    .to_string());
            }
        };

        Ok(format!(
            "{}\n\n*{} is Open Game Content subject to the `Open Game License`.*",
            linkify_dice(&output),
            name,
        ))
    }
}

#[async_trait(?Send)]
impl ContextAwareParse for ReferenceCommand {
    async fn parse_input(input: &str, _app_meta: &AppMeta) -> (Option<Self>, Vec<Self>) {
        let exact_match = if input.eq_ci("Open Game License") {
            Some(Self::OpenGameLicense)
        } else if input.eq_ci("srd spells") {
            Some(Self::Spells)
        } else {
            None.or_else(|| {
                input
                    .strip_prefix_ci("srd spell ")
                    .and_then(|s| s.parse().ok())
                    .map(Self::Spell)
            })
            .or_else(|| {
                input
                    .strip_prefix_ci("srd item category ")
                    .and_then(|s| s.parse().ok())
                    .map(Self::ItemCategory)
            })
            .or_else(|| {
                input
                    .strip_prefix_ci("srd item ")
                    .and_then(|s| s.parse().ok())
                    .map(Self::Item)
            })
            .or_else(|| {
                input
                    .strip_prefix_ci("srd magic item ")
                    .and_then(|s| s.parse().ok())
                    .map(Self::MagicItem)
            })
        };

        let mut fuzzy_matches = Vec::new();

        if let Ok(spell) = input.parse() {
            fuzzy_matches.push(Self::Spell(spell));
        }
        if let Ok(item) = input.parse() {
            fuzzy_matches.push(Self::Item(item));
        }
        if let Ok(category) = input.parse() {
            fuzzy_matches.push(Self::ItemCategory(category));
        }
        if let Ok(magic_item) = input.parse() {
            fuzzy_matches.push(Self::MagicItem(magic_item));
        }
        if input == "spells" {
            fuzzy_matches.push(Self::Spells);
        }

        (exact_match, fuzzy_matches)
    }
}

#[async_trait(?Send)]
impl Autocomplete for ReferenceCommand {
    async fn autocomplete(input: &str, _app_meta: &AppMeta) -> Vec<(String, String)> {
        [
            ("Open Game License", "SRD license"),
            ("spells", "SRD index"),
        ]
        .into_iter()
        .chain(Spell::get_words().zip(repeat("SRD spell")))
        .chain(Item::get_words().zip(repeat("SRD item")))
        .chain(ItemCategory::get_words().zip(repeat("SRD item category")))
        .chain(MagicItem::get_words().zip(repeat("SRD magic item")))
        .filter(|(s, _)| s.starts_with_ci(input))
        .take(10)
        .map(|(a, b)| (a.to_string(), b.to_string()))
        .collect()
    }
}

impl fmt::Display for ReferenceCommand {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            Self::Spell(spell) => write!(f, "srd spell {}", spell.get_name()),
            Self::Spells => write!(f, "srd spells"),
            Self::Item(item) => write!(f, "srd item {}", item.get_name()),
            Self::ItemCategory(category) => write!(f, "srd item category {}", category.get_name()),
            Self::MagicItem(item) => write!(f, "srd magic item {}", item.get_name()),
            Self::OpenGameLicense => write!(f, "Open Game License"),
        }
    }
}

fn linkify_dice(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut input_offset = 0;

    let mut hold = String::new();
    let mut hold_offset = 0;
    let mut hold_active = false;

    for part in input.split_inclusive(|c: char| c.is_whitespace() || c.is_ascii_punctuation()) {
        if !hold_active
            && part.contains(|c: char| c.is_ascii_digit())
            && part.contains(&['d', 'D'][..])
        {
            hold_active = true;
            hold_offset = input_offset;
        } else if hold_active && part.contains(char::is_alphabetic) {
            hold_active = false;
        }

        if hold_active {
            hold.push_str(part);
        } else {
            while !hold.is_empty() {
                let hold_trimmed = hold.trim();
                if hold_trimmed.contains(&['d', 'D'][..])
                    && Roller::new(hold_trimmed).map_or(false, |r| r.roll().is_ok())
                {
                    result.push('`');
                    result.push_str(hold_trimmed);
                    result.push('`');
                    result.push_str(&input[hold_offset + hold_trimmed.len()..input_offset]);
                    hold.clear();
                    break;
                }

                if let Some(pos) =
                    hold.rfind(|c: char| c.is_whitespace() || c.is_ascii_punctuation())
                {
                    hold.truncate(pos);

                    if hold.is_empty() {
                        result.push_str(&input[hold_offset..input_offset]);
                    }
                } else {
                    result.push_str(&input[hold_offset..input_offset]);
                    hold.clear();
                }
            }

            result.push_str(part);
        }

        input_offset += part.len();
    }

    result.push_str(&hold);
    result
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::storage::NullDataStore;
    use tokio_test::block_on;

    #[test]
    fn display_test() {
        let app_meta = AppMeta::new(NullDataStore::default());

        vec![
            ReferenceCommand::Spell(Spell::Shield),
            ReferenceCommand::Spells,
            ReferenceCommand::Item(Item::Shield),
            ReferenceCommand::ItemCategory(ItemCategory::Shields),
            ReferenceCommand::MagicItem(MagicItem::DeckOfManyThings),
            ReferenceCommand::OpenGameLicense,
        ]
        .drain(..)
        .for_each(|command| {
            let command_string = command.to_string();
            assert_ne!("", command_string);

            assert_eq!(
                (Some(command.clone()), Vec::new()),
                block_on(ReferenceCommand::parse_input(&command_string, &app_meta)),
                "{}",
                command_string,
            );

            assert_eq!(
                (Some(command), Vec::new()),
                block_on(ReferenceCommand::parse_input(
                    &command_string.to_uppercase(),
                    &app_meta,
                )),
                "{}",
                command_string.to_uppercase(),
            );
        });
    }
}
