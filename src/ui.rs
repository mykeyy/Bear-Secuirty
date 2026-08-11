use twilight_model::channel::message::component::{
    ActionRow, Button, ButtonStyle, Component, Container, Separator, SeparatorSpacingSize,
    TextDisplay,
};

use crate::storage::GuildSettings;

pub fn security_panel(settings: &GuildSettings, backend: &str) -> Vec<Component> {
    let honeypot = settings
        .honeypot_channel_id
        .map_or_else(|| "Off".to_owned(), |id| format!("<#{}>", id));

    vec![Component::Container(Container {
        accent_color: None,
        id: None,
        spoiler: None,
        components: vec![
            text("# Bear Security"),
            text(format!(
                "**Anti spam:** {}\n**Anti scam:** {}\n**Honeypot:** {}\n**Storage:** `{backend}`",
                on_off(settings.anti_spam),
                on_off(settings.anti_scam),
                honeypot,
            )),
            Component::Separator(Separator {
                divider: Some(true),
                id: None,
                spacing: Some(SeparatorSpacingSize::Small),
            }),
            Component::ActionRow(ActionRow {
                id: None,
                components: vec![
                    button(
                        format!("Anti spam: {}", on_off(settings.anti_spam)),
                        "bear:toggle:spam",
                    ),
                    button(
                        format!("Anti scam: {}", on_off(settings.anti_scam)),
                        "bear:toggle:scam",
                    ),
                    button(
                        if settings.honeypot_channel_id.is_some() {
                            "Remove honeypot"
                        } else {
                            "Create honeypot"
                        },
                        "bear:toggle:honeypot",
                    ),
                    button("Refresh", "bear:refresh"),
                ],
            }),
        ],
    })]
}

pub fn about() -> Vec<Component> {
    vec![Component::Container(Container {
        accent_color: None,
        id: None,
        spoiler: None,
        components: vec![
            text("# Bear Security"),
            text(
                "A small Rust Discord security bot with anti spam, anti scam, a honeypot, welcome and leave messages, and optional autoroles.",
            ),
            text("Legacy prefix: `b!`\nVersion: `1.0.0`"),
        ],
    })]
}

pub fn notice(message: impl Into<String>) -> Vec<Component> {
    vec![Component::Container(Container {
        accent_color: None,
        id: None,
        spoiler: None,
        components: vec![text(message)],
    })]
}

pub fn welcome(user_id: u64) -> Vec<Component> {
    notice(format!(
        "# Welcome, <@{user_id}>\nGlad to have you with us! Make yourself at home."
    ))
}

pub fn goodbye(username: &str) -> Vec<Component> {
    notice(format!(
        "**{}** has left the server. We hope to see you again someday.",
        escape_markdown(username)
    ))
}

pub fn honeypot_warning() -> Vec<Component> {
    notice(
        "# Bear Security honeypot\nDo not send messages in this channel. Messages here are treated as automated spam probes. Bots and moderators are exempt.",
    )
}

fn text(content: impl Into<String>) -> Component {
    Component::TextDisplay(TextDisplay {
        content: content.into(),
        id: None,
    })
}

fn button(label: impl Into<String>, custom_id: impl Into<String>) -> Component {
    Component::Button(Button {
        custom_id: Some(custom_id.into()),
        disabled: false,
        emoji: None,
        id: None,
        label: Some(label.into()),
        sku_id: None,
        style: ButtonStyle::Secondary,
        url: None,
    })
}

fn on_off(enabled: bool) -> &'static str {
    if enabled { "On" } else { "Off" }
}

fn escape_markdown(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('*', "\\*")
        .replace('_', "\\_")
        .replace('`', "\\`")
}
