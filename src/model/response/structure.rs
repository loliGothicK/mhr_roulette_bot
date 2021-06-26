type OptionValue = serenity::model::interactions::ApplicationCommandInteractionDataOptionValue;

#[derive(Debug, Clone)]
pub enum SlashCommand {
    Command(String),
    SubCommand(String),
    Option(Box<OptionValue>),
}

#[derive(Debug, Clone)]
pub enum Component {
    Button(String),
    SelectMenu(Vec<String>),
}

#[derive(Debug, Clone)]
pub enum Response {
    SlashCommand(SlashCommand),
    Component(Component),
}
