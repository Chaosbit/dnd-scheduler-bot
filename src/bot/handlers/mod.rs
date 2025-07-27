pub mod callback;
pub mod message;

use teloxide::{
    dispatching::{dialogue, UpdateHandler},
    prelude::*,
};
use crate::database::connection::DatabaseManager;

pub struct BotHandler {
    pub db: DatabaseManager,
}

impl BotHandler {
    pub fn new(db: DatabaseManager) -> Self {
        Self { db }
    }

    pub fn schema(&self) -> UpdateHandler<Box<dyn std::error::Error + Send + Sync + 'static>> {
        use teloxide::dispatching::UpdateFilterExt;
        
        let db = self.db.clone();
        let db_callback = self.db.clone();
        
        dialogue::enter::<Update, teloxide::dispatching::dialogue::InMemStorage<()>, (), _>()
            .branch(
                Update::filter_message()
                    .filter_command::<crate::bot::commands::Command>()
                    .endpoint(move |bot, msg, cmd| {
                        let db = db.clone();
                        async move { message::command_handler(bot, msg, cmd, db).await }
                    }),
            )
            .branch(Update::filter_callback_query().endpoint(move |bot, q| {
                let db = db_callback.clone();
                async move { callback::callback_handler(bot, q, db).await }
            }))
    }
}
