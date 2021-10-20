pub use command::{
    autocomplete_phrase, AppCommand, Autocomplete, Command, CommandAlias, ContextAwareParse,
    Runnable,
};
pub use meta::AppMeta;

mod command;
mod meta;

use initiative_macros::motd;

pub struct App {
    meta: AppMeta,
}

impl App {
    pub fn new(meta: AppMeta) -> App {
        App { meta }
    }

    pub async fn init(&mut self) -> &'static str {
        self.meta.repository.init().await;
        let (motd, motd_len) = motd!("! Local storage is not available in your browser. You will be able to use initiative.sh, but saving and loading won't work.");

        if self.meta.repository.data_store_enabled() {
            &motd[..motd_len]
        } else {
            motd
        }
    }

    pub async fn command(&mut self, input: &str) -> Result<String, String> {
        Command::parse_input_irrefutable(input, &self.meta)
            .run(input, &mut self.meta)
            .await
    }

    pub async fn autocomplete(&self, input: &str) -> Vec<(String, String)> {
        let mut suggestions: Vec<_> = Command::autocomplete(input, &self.meta);
        suggestions.sort_by(|(a, _), (b, _)| a.cmp(b));
        suggestions
    }
}
