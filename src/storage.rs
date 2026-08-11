use std::{collections::HashMap, env};

use anyhow::{Context, Result, bail};
use libsql::{Builder, Connection, params};
use rustls::RootCertStore;
use tokio_postgres::Client;
use tokio_postgres_rustls::MakeRustlsConnect;

#[derive(Clone, Debug, Default)]
pub struct GuildSettings {
    pub anti_spam: bool,
    pub anti_scam: bool,
    pub welcome_channel_id: Option<u64>,
    pub leave_channel_id: Option<u64>,
    pub autorole_id: Option<u64>,
    pub honeypot_channel_id: Option<u64>,
}

pub enum Storage {
    Memory(HashMap<u64, GuildSettings>),
    Turso(Connection),
    Postgres(Client),
}

impl Storage {
    pub async fn from_env() -> Result<Self> {
        match env::var("STORAGE_BACKEND")
            .unwrap_or_else(|_| "memory".to_owned())
            .to_ascii_lowercase()
            .as_str()
        {
            "memory" => Ok(Self::Memory(HashMap::new())),
            "turso" => {
                let url =
                    env::var("TURSO_DATABASE_URL").context("TURSO_DATABASE_URL is required")?;
                let token = env::var("TURSO_AUTH_TOKEN").context("TURSO_AUTH_TOKEN is required")?;
                let db = Builder::new_remote(url, token).build().await?;
                let connection = db.connect()?;
                let storage = Self::Turso(connection);
                storage.migrate().await?;
                Ok(storage)
            }
            "postgres" | "postgresql" => {
                let url = env::var("DATABASE_URL").context("DATABASE_URL is required")?;
                let roots =
                    RootCertStore::from_iter(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
                let config = rustls::ClientConfig::builder()
                    .with_root_certificates(roots)
                    .with_no_client_auth();
                let tls = MakeRustlsConnect::new(config);
                let (client, connection) = tokio_postgres::connect(&url, tls).await?;

                tokio::spawn(async move {
                    if let Err(error) = connection.await {
                        tracing::error!(%error, "PostgreSQL connection ended");
                    }
                });

                let storage = Self::Postgres(client);
                storage.migrate().await?;
                Ok(storage)
            }
            other => bail!("unsupported STORAGE_BACKEND: {other}"),
        }
    }

    pub const fn name(&self) -> &'static str {
        match self {
            Self::Memory(_) => "memory",
            Self::Turso(_) => "turso",
            Self::Postgres(_) => "postgres",
        }
    }

    async fn migrate(&self) -> Result<()> {
        match self {
            Self::Memory(_) => {}
            Self::Turso(connection) => {
                connection
                    .execute(
                        "CREATE TABLE IF NOT EXISTS guild_settings (\
                         guild_id TEXT PRIMARY KEY, \
                         anti_spam INTEGER NOT NULL DEFAULT 0, \
                         anti_scam INTEGER NOT NULL DEFAULT 0, \
                         welcome_channel_id TEXT, \
                         leave_channel_id TEXT, \
                         autorole_id TEXT, \
                         honeypot_channel_id TEXT)",
                        (),
                    )
                    .await?;
            }
            Self::Postgres(client) => {
                client
                    .batch_execute(
                        "CREATE TABLE IF NOT EXISTS guild_settings (\
                         guild_id TEXT PRIMARY KEY, \
                         anti_spam BOOLEAN NOT NULL DEFAULT FALSE, \
                         anti_scam BOOLEAN NOT NULL DEFAULT FALSE, \
                         welcome_channel_id TEXT, \
                         leave_channel_id TEXT, \
                         autorole_id TEXT, \
                         honeypot_channel_id TEXT);",
                    )
                    .await?;
            }
        }

        Ok(())
    }

    pub async fn get(&mut self, guild_id: u64) -> Result<GuildSettings> {
        match self {
            Self::Memory(settings) => Ok(settings.get(&guild_id).cloned().unwrap_or_default()),
            Self::Turso(connection) => {
                let mut rows = connection
                    .query(
                        "SELECT anti_spam, anti_scam, welcome_channel_id, leave_channel_id, \
                         autorole_id, honeypot_channel_id FROM guild_settings WHERE guild_id = ?1",
                        params![guild_id.to_string()],
                    )
                    .await?;

                let Some(row) = rows.next().await? else {
                    return Ok(GuildSettings::default());
                };

                Ok(GuildSettings {
                    anti_spam: row.get::<i64>(0)? != 0,
                    anti_scam: row.get::<i64>(1)? != 0,
                    welcome_channel_id: parse_id(row.get::<Option<String>>(2)?),
                    leave_channel_id: parse_id(row.get::<Option<String>>(3)?),
                    autorole_id: parse_id(row.get::<Option<String>>(4)?),
                    honeypot_channel_id: parse_id(row.get::<Option<String>>(5)?),
                })
            }
            Self::Postgres(client) => {
                let guild_id = guild_id.to_string();
                let Some(row) = client
                    .query_opt(
                        "SELECT anti_spam, anti_scam, welcome_channel_id, leave_channel_id, \
                         autorole_id, honeypot_channel_id FROM guild_settings WHERE guild_id = $1",
                        &[&guild_id],
                    )
                    .await?
                else {
                    return Ok(GuildSettings::default());
                };

                Ok(GuildSettings {
                    anti_spam: row.get(0),
                    anti_scam: row.get(1),
                    welcome_channel_id: parse_id(row.get(2)),
                    leave_channel_id: parse_id(row.get(3)),
                    autorole_id: parse_id(row.get(4)),
                    honeypot_channel_id: parse_id(row.get(5)),
                })
            }
        }
    }

    pub async fn save(&mut self, guild_id: u64, settings: &GuildSettings) -> Result<()> {
        match self {
            Self::Memory(all) => {
                all.insert(guild_id, settings.clone());
            }
            Self::Turso(connection) => {
                connection
                    .execute(
                        "INSERT INTO guild_settings (guild_id, anti_spam, anti_scam, \
                         welcome_channel_id, leave_channel_id, autorole_id, honeypot_channel_id) \
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7) \
                         ON CONFLICT(guild_id) DO UPDATE SET \
                         anti_spam = excluded.anti_spam, anti_scam = excluded.anti_scam, \
                         welcome_channel_id = excluded.welcome_channel_id, \
                         leave_channel_id = excluded.leave_channel_id, \
                         autorole_id = excluded.autorole_id, \
                         honeypot_channel_id = excluded.honeypot_channel_id",
                        params![
                            guild_id.to_string(),
                            i64::from(settings.anti_spam),
                            i64::from(settings.anti_scam),
                            format_id(settings.welcome_channel_id),
                            format_id(settings.leave_channel_id),
                            format_id(settings.autorole_id),
                            format_id(settings.honeypot_channel_id),
                        ],
                    )
                    .await?;
            }
            Self::Postgres(client) => {
                let guild_id = guild_id.to_string();
                let welcome = format_id(settings.welcome_channel_id);
                let leave = format_id(settings.leave_channel_id);
                let autorole = format_id(settings.autorole_id);
                let honeypot = format_id(settings.honeypot_channel_id);

                client
                    .execute(
                        "INSERT INTO guild_settings (guild_id, anti_spam, anti_scam, \
                         welcome_channel_id, leave_channel_id, autorole_id, honeypot_channel_id) \
                         VALUES ($1, $2, $3, $4, $5, $6, $7) \
                         ON CONFLICT(guild_id) DO UPDATE SET \
                         anti_spam = EXCLUDED.anti_spam, anti_scam = EXCLUDED.anti_scam, \
                         welcome_channel_id = EXCLUDED.welcome_channel_id, \
                         leave_channel_id = EXCLUDED.leave_channel_id, \
                         autorole_id = EXCLUDED.autorole_id, \
                         honeypot_channel_id = EXCLUDED.honeypot_channel_id",
                        &[
                            &guild_id,
                            &settings.anti_spam,
                            &settings.anti_scam,
                            &welcome,
                            &leave,
                            &autorole,
                            &honeypot,
                        ],
                    )
                    .await?;
            }
        }

        Ok(())
    }
}

fn parse_id(value: Option<String>) -> Option<u64> {
    value.and_then(|value| value.parse().ok())
}

fn format_id(value: Option<u64>) -> Option<String> {
    value.map(|value| value.to_string())
}
