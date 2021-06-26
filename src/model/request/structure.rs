use super::component::Component;

pub enum Message {
    String(String),
    Embed(serenity::builder::CreateEmbed),
}

pub enum Request {
    Message(Message),
    Components(Component),
    Update {
        content: String,
        component: Option<Component>,
    },
}
