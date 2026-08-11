mod security;
mod storage;
mod ui;

use std::{env, time::Instant};

use anyhow::{Context, Result};
use security::{SpamTracker, looks_like_scam};
use storage::{GuildSettings, Storage};
use twilight_gateway::{Event, EventTypeFlags, Intents, Shard, ShardId, StreamExt as _};
use twilight_http::{Client as HttpClient, request::AuditLogReason};
use twilight_model::{
    application::{
        command::CommandType,
        interaction::{
            Interaction, InteractionData,
            application_command::{CommandData, CommandOptionValue},
        },
    },
    channel::{ChannelType, message::MessageFlags},
    gateway::payload::incoming::{MemberAdd, MemberRemove, MessageCreate},
    guild::{PartialMember, Permissions},
    http::interaction::{InteractionResponse, InteractionResponseData, InteractionResponseType},
    id::{
        Id,
        marker::{ApplicationMarker, ChannelMarker, GuildMarker, RoleMarker, UserMarker},
    },
};
use twilight_util::builder::command::{ChannelBuilder, CommandBuilder, RoleBuilder};

struct App {
    http: HttpClient,
    storage: Storage,
    spam: SpamTracker,
    started_at: Instant,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("bear_security=info")),
        )
        .init();

    let _ = rustls::crypto::ring::default_provider().install_default();

    let token = env::var("DISCORD_TOKEN").context("DISCORD_TOKEN is required")?;
    let intents = Intents::GUILDS
        | Intents::GUILD_MEMBERS
        | Intents::GUILD_MESSAGES
        | Intents::MESSAGE_CONTENT;
    let mut shard = Shard::new(ShardId::ONE, token.clone(), intents);
    let http = HttpClient::new(token);

    let application = http.current_user_application().await?.model().await?;
    register_commands(&http, application.id).await?;

    let storage = Storage::from_env().await?;
    tracing::info!(backend = storage.name(), "storage ready");

    let mut app = App {
        http,
        storage,
        spam: SpamTracker::default(),
        started_at: Instant::now(),
    };

    while let Some(event) = shard.next_event(EventTypeFlags::all()).await {
        match event {
            Ok(event) => {
                if let Err(error) = app.handle_event(event).await {
                    tracing::error!(%error, "event handler failed");
                }
            }
            Err(error) => tracing::warn!(%error, "gateway event failed"),
        }
    }

    Ok(())
}

impl App {
    async fn handle_event(&mut self, event: Event) -> Result<()> {
        match event {
            Event::Ready(ready) => {
                tracing::info!(user = %ready.user.name, "Bear Security connected");
            }
            Event::MessageCreate(message) => self.on_message(&message).await?,
            Event::MemberAdd(member) => self.on_member_add(&member).await?,
            Event::MemberRemove(member) => self.on_member_remove(&member).await?,
            Event::InteractionCreate(interaction) => self.on_interaction(&interaction.0).await?,
            _ => {}
        }

        Ok(())
    }

    async fn on_message(&mut self, message: &MessageCreate) -> Result<()> {
        if message.author.bot {
            return Ok(());
        }

        let Some(guild_id) = message.guild_id else {
            return Ok(());
        };

        if message.content.starts_with("b!") {
            self.handle_legacy_command(message, guild_id).await?;
            return Ok(());
        }

        let settings = self.storage.get(guild_id.get()).await?;
        let is_honeypot = settings.honeypot_channel_id == Some(message.channel_id.get());
        let is_scam = settings.anti_scam && looks_like_scam(&message.content);
        let is_spam = settings.anti_spam
            && self
                .spam
                .is_spam(guild_id.get(), message.author.id.get(), &message.content);

        if !(is_honeypot || is_scam || is_spam) {
            return Ok(());
        }

        if self
            .is_moderator(guild_id, message.author.id, message.member.as_ref())
            .await?
        {
            return Ok(());
        }

        self.http
            .delete_message(message.channel_id, message.id)
            .await?;

        if is_honeypot {
            self.http
                .remove_guild_member(guild_id, message.author.id)
                .reason("Bear Security honeypot triggered")
                .await?;
            tracing::warn!(
                guild = guild_id.get(),
                user = message.author.id.get(),
                "honeypot kicked member"
            );
        } else if is_scam {
            tracing::warn!(
                guild = guild_id.get(),
                user = message.author.id.get(),
                "removed suspicious scam message"
            );
        } else {
            tracing::warn!(
                guild = guild_id.get(),
                user = message.author.id.get(),
                "removed spam message"
            );
        }

        Ok(())
    }

    async fn handle_legacy_command(
        &mut self,
        message: &MessageCreate,
        guild_id: Id<GuildMarker>,
    ) -> Result<()> {
        match message
            .content
            .split_whitespace()
            .next()
            .unwrap_or_default()
        {
            "b!ping" => {
                let uptime = self.started_at.elapsed().as_secs();
                let components = ui::notice(format!(
                    "# Pong\nBear Security is online.\nUptime: `{}`",
                    format_uptime(uptime)
                ));
                send_v2(&self.http, message.channel_id, &components).await?;
            }
            "b!about" => {
                let components = ui::about();
                send_v2(&self.http, message.channel_id, &components).await?;
            }
            "b!security" | "b!settings" => {
                let settings = self.storage.get(guild_id.get()).await?;
                let components = ui::security_panel(&settings, self.storage.name());
                send_v2(&self.http, message.channel_id, &components).await?;
            }
            _ => {}
        }

        Ok(())
    }

    async fn on_member_add(&mut self, event: &MemberAdd) -> Result<()> {
        let settings = self.storage.get(event.guild_id.get()).await?;

        if let Some(role_id) = settings.autorole_id
            && let Err(error) = self
                .http
                .add_guild_member_role(
                    event.guild_id,
                    event.user.id,
                    Id::<RoleMarker>::new(role_id),
                )
                .reason("Bear Security autorole")
                .await
        {
            tracing::warn!(%error, "failed to add autorole");
        }

        if let Some(channel_id) = settings.welcome_channel_id {
            let components = ui::welcome(event.user.id.get());
            if let Err(error) = send_v2(
                &self.http,
                Id::<ChannelMarker>::new(channel_id),
                &components,
            )
            .await
            {
                tracing::warn!(%error, "failed to send welcome message");
            }
        }

        Ok(())
    }

    async fn on_member_remove(&mut self, event: &MemberRemove) -> Result<()> {
        let settings = self.storage.get(event.guild_id.get()).await?;
        let Some(channel_id) = settings.leave_channel_id else {
            return Ok(());
        };

        let components = ui::goodbye(&event.user.name);
        send_v2(
            &self.http,
            Id::<ChannelMarker>::new(channel_id),
            &components,
        )
        .await?;

        Ok(())
    }

    async fn on_interaction(&mut self, interaction: &Interaction) -> Result<()> {
        let Some(data) = interaction.data.as_ref() else {
            return Ok(());
        };

        match data {
            InteractionData::ApplicationCommand(command) => {
                self.handle_application_command(interaction, command)
                    .await?;
            }
            InteractionData::MessageComponent(component) => {
                self.handle_component(interaction, &component.custom_id)
                    .await?;
            }
            _ => {}
        }

        Ok(())
    }

    async fn handle_application_command(
        &mut self,
        interaction: &Interaction,
        command: &CommandData,
    ) -> Result<()> {
        match command.name.as_str() {
            "about" => {
                self.respond_v2(interaction, ui::about(), true).await?;
            }
            "security" => {
                let Some(guild_id) = interaction.guild_id else {
                    self.respond_v2(
                        interaction,
                        ui::notice("This command only works in a server."),
                        true,
                    )
                    .await?;
                    return Ok(());
                };
                if !can_manage_guild(interaction) {
                    self.respond_v2(
                        interaction,
                        ui::notice("You need **Manage Server** to use this panel."),
                        true,
                    )
                    .await?;
                    return Ok(());
                }
                let settings = self.storage.get(guild_id.get()).await?;
                let panel = ui::security_panel(&settings, self.storage.name());
                self.respond_v2(interaction, panel, true).await?;
            }
            "welcome" => {
                self.set_channel_setting(interaction, command, ChannelSetting::Welcome)
                    .await?;
            }
            "leave" => {
                self.set_channel_setting(interaction, command, ChannelSetting::Leave)
                    .await?;
            }
            "autorole" => self.set_autorole(interaction, command).await?,
            _ => {}
        }

        Ok(())
    }

    async fn handle_component(&mut self, interaction: &Interaction, custom_id: &str) -> Result<()> {
        if !custom_id.starts_with("bear:") {
            return Ok(());
        }

        let Some(guild_id) = interaction.guild_id else {
            return Ok(());
        };
        if !can_manage_guild(interaction) {
            self.respond_v2(
                interaction,
                ui::notice("You need **Manage Server** to change Bear Security settings."),
                true,
            )
            .await?;
            return Ok(());
        }

        let mut settings = self.storage.get(guild_id.get()).await?;
        match custom_id {
            "bear:toggle:spam" => settings.anti_spam = !settings.anti_spam,
            "bear:toggle:scam" => settings.anti_scam = !settings.anti_scam,
            "bear:toggle:honeypot" => {
                self.toggle_honeypot(guild_id, &mut settings).await?;
            }
            "bear:refresh" => {}
            _ => return Ok(()),
        }
        self.storage.save(guild_id.get(), &settings).await?;

        let panel = ui::security_panel(&settings, self.storage.name());
        self.update_v2(interaction, panel).await?;
        Ok(())
    }

    async fn set_channel_setting(
        &mut self,
        interaction: &Interaction,
        command: &CommandData,
        target: ChannelSetting,
    ) -> Result<()> {
        let Some(guild_id) = interaction.guild_id else {
            return Ok(());
        };
        if !can_manage_guild(interaction) {
            self.respond_v2(
                interaction,
                ui::notice("You need **Manage Server** to change this setting."),
                true,
            )
            .await?;
            return Ok(());
        }

        let channel = command
            .options
            .first()
            .and_then(|option| match &option.value {
                CommandOptionValue::Channel(id) => Some(id.get()),
                _ => None,
            });
        let mut settings = self.storage.get(guild_id.get()).await?;
        match target {
            ChannelSetting::Welcome => settings.welcome_channel_id = channel,
            ChannelSetting::Leave => settings.leave_channel_id = channel,
        }
        self.storage.save(guild_id.get(), &settings).await?;

        let label = match target {
            ChannelSetting::Welcome => "Welcome channel",
            ChannelSetting::Leave => "Leave channel",
        };
        let value = channel.map_or_else(|| "disabled".to_owned(), |id| format!("set to <#{id}>"));
        self.respond_v2(interaction, ui::notice(format!("{label} {value}.")), true)
            .await?;
        Ok(())
    }

    async fn set_autorole(
        &mut self,
        interaction: &Interaction,
        command: &CommandData,
    ) -> Result<()> {
        let Some(guild_id) = interaction.guild_id else {
            return Ok(());
        };
        if !can_manage_guild(interaction) {
            self.respond_v2(
                interaction,
                ui::notice("You need **Manage Server** to change this setting."),
                true,
            )
            .await?;
            return Ok(());
        }

        let role = command
            .options
            .first()
            .and_then(|option| match &option.value {
                CommandOptionValue::Role(id) => Some(id.get()),
                _ => None,
            });
        let mut settings = self.storage.get(guild_id.get()).await?;
        settings.autorole_id = role;
        self.storage.save(guild_id.get(), &settings).await?;

        let value = role.map_or_else(|| "disabled".to_owned(), |id| format!("set to <@&{id}>"));
        self.respond_v2(interaction, ui::notice(format!("Autorole {value}.")), true)
            .await?;
        Ok(())
    }

    async fn toggle_honeypot(
        &self,
        guild_id: Id<GuildMarker>,
        settings: &mut GuildSettings,
    ) -> Result<()> {
        if let Some(channel_id) = settings.honeypot_channel_id.take() {
            self.http
                .delete_channel(Id::<ChannelMarker>::new(channel_id))
                .await?;
            return Ok(());
        }

        let channel = self
            .http
            .create_guild_channel(guild_id, "bear-honeypot")
            .await?
            .model()
            .await?;
        settings.honeypot_channel_id = Some(channel.id.get());

        let components = ui::honeypot_warning();
        send_v2(&self.http, channel.id, &components).await?;
        Ok(())
    }

    async fn is_moderator(
        &self,
        guild_id: Id<GuildMarker>,
        user_id: Id<UserMarker>,
        member: Option<&PartialMember>,
    ) -> Result<bool> {
        let guild = self.http.guild(guild_id).await?.model().await?;
        if guild.owner_id == user_id {
            return Ok(true);
        }

        let Some(member) = member else {
            return Ok(false);
        };
        let roles = self.http.roles(guild_id).await?.models().await?;
        let moderator_permissions =
            Permissions::ADMINISTRATOR | Permissions::MANAGE_GUILD | Permissions::MANAGE_MESSAGES;

        Ok(roles.iter().any(|role| {
            (role.id.get() == guild_id.get() || member.roles.contains(&role.id))
                && role.permissions.intersects(moderator_permissions)
        }))
    }

    async fn respond_v2(
        &self,
        interaction: &Interaction,
        components: Vec<twilight_model::channel::message::component::Component>,
        ephemeral: bool,
    ) -> Result<()> {
        let mut flags = MessageFlags::IS_COMPONENTS_V2;
        if ephemeral {
            flags |= MessageFlags::EPHEMERAL;
        }
        let response = InteractionResponse {
            kind: InteractionResponseType::ChannelMessageWithSource,
            data: Some(InteractionResponseData {
                components: Some(components),
                flags: Some(flags),
                ..InteractionResponseData::default()
            }),
        };
        self.http
            .interaction(interaction.application_id)
            .create_response(interaction.id, &interaction.token, &response)
            .await?;
        Ok(())
    }

    async fn update_v2(
        &self,
        interaction: &Interaction,
        components: Vec<twilight_model::channel::message::component::Component>,
    ) -> Result<()> {
        let response = InteractionResponse {
            kind: InteractionResponseType::UpdateMessage,
            data: Some(InteractionResponseData {
                components: Some(components),
                ..InteractionResponseData::default()
            }),
        };
        self.http
            .interaction(interaction.application_id)
            .create_response(interaction.id, &interaction.token, &response)
            .await?;
        Ok(())
    }
}

#[derive(Clone, Copy)]
enum ChannelSetting {
    Welcome,
    Leave,
}

async fn register_commands(http: &HttpClient, application_id: Id<ApplicationMarker>) -> Result<()> {
    let manage = Permissions::MANAGE_GUILD;
    let commands = vec![
        CommandBuilder::new("about", "About Bear Security", CommandType::ChatInput).build(),
        CommandBuilder::new(
            "security",
            "Open the Bear Security control panel",
            CommandType::ChatInput,
        )
        .default_member_permissions(manage)
        .build(),
        CommandBuilder::new(
            "welcome",
            "Set the welcome channel, or leave empty to disable it",
            CommandType::ChatInput,
        )
        .default_member_permissions(manage)
        .option(
            ChannelBuilder::new("channel", "Channel for welcome messages")
                .channel_types([ChannelType::GuildText, ChannelType::GuildAnnouncement]),
        )
        .build(),
        CommandBuilder::new(
            "leave",
            "Set the leave channel, or leave empty to disable it",
            CommandType::ChatInput,
        )
        .default_member_permissions(manage)
        .option(
            ChannelBuilder::new("channel", "Channel for leave messages")
                .channel_types([ChannelType::GuildText, ChannelType::GuildAnnouncement]),
        )
        .build(),
        CommandBuilder::new(
            "autorole",
            "Set the role given to new members, or leave empty to disable it",
            CommandType::ChatInput,
        )
        .default_member_permissions(manage)
        .option(RoleBuilder::new("role", "Role to give new members"))
        .build(),
    ];

    http.interaction(application_id)
        .set_global_commands(&commands)
        .await?;
    Ok(())
}

async fn send_v2(
    http: &HttpClient,
    channel_id: Id<ChannelMarker>,
    components: &[twilight_model::channel::message::component::Component],
) -> Result<()> {
    http.create_message(channel_id)
        .flags(MessageFlags::IS_COMPONENTS_V2)
        .components(components)
        .await?;
    Ok(())
}

fn can_manage_guild(interaction: &Interaction) -> bool {
    interaction
        .member
        .as_ref()
        .and_then(|member| member.permissions)
        .is_some_and(|permissions| {
            permissions.contains(Permissions::ADMINISTRATOR)
                || permissions.contains(Permissions::MANAGE_GUILD)
        })
}

fn format_uptime(seconds: u64) -> String {
    let days = seconds / 86_400;
    let hours = (seconds % 86_400) / 3_600;
    let minutes = (seconds % 3_600) / 60;
    let seconds = seconds % 60;

    if days > 0 {
        format!("{days}d {hours}h {minutes}m {seconds}s")
    } else if hours > 0 {
        format!("{hours}h {minutes}m {seconds}s")
    } else {
        format!("{minutes}m {seconds}s")
    }
}
