use std::{
    collections::{HashMap, VecDeque},
    time::{Duration, Instant},
};

const BURST_WINDOW: Duration = Duration::from_secs(8);
const REPEAT_WINDOW: Duration = Duration::from_secs(12);
const BURST_LIMIT: usize = 6;
const REPEAT_LIMIT: usize = 3;

#[derive(Default)]
pub struct SpamTracker {
    users: HashMap<(u64, u64), UserWindow>,
}

#[derive(Default)]
struct UserWindow {
    messages: VecDeque<Instant>,
    last_content: String,
    last_repeat_at: Option<Instant>,
    repeats: usize,
}

impl SpamTracker {
    pub fn is_spam(&mut self, guild_id: u64, user_id: u64, content: &str) -> bool {
        let now = Instant::now();
        let window = self.users.entry((guild_id, user_id)).or_default();

        while window
            .messages
            .front()
            .is_some_and(|sent| now.duration_since(*sent) > BURST_WINDOW)
        {
            window.messages.pop_front();
        }
        window.messages.push_back(now);

        let normalized = content.trim().to_ascii_lowercase();
        if !normalized.is_empty()
            && normalized == window.last_content
            && window
                .last_repeat_at
                .is_some_and(|sent| now.duration_since(sent) <= REPEAT_WINDOW)
        {
            window.repeats += 1;
        } else {
            window.repeats = 1;
            window.last_content = normalized;
        }
        window.last_repeat_at = Some(now);

        window.messages.len() >= BURST_LIMIT || window.repeats >= REPEAT_LIMIT
    }
}

pub fn looks_like_scam(content: &str) -> bool {
    let text = content.to_ascii_lowercase();
    let has_link = ["https://", "http://", "discord.gg/", "discord.com/invite/"]
        .iter()
        .any(|needle| text.contains(needle));

    if !has_link {
        return false;
    }

    let bait = [
        "mrbeast",
        "free nitro",
        "steam gift",
        "free robux",
        "airdrop",
        "crypto giveaway",
        "you won",
        "you've won",
    ];
    let action = [
        "claim",
        "verify",
        "click",
        "redeem",
        "limited time",
        "expires",
        "giveaway",
    ];

    bait.iter().any(|needle| text.contains(needle))
        && action.iter().any(|needle| text.contains(needle))
}

#[cfg(test)]
mod tests {
    use super::{SpamTracker, looks_like_scam};

    #[test]
    fn scam_detector_requires_bait_action_and_link() {
        assert!(looks_like_scam(
            "MRBEAST GIVEAWAY: claim your prize at https://example.invalid"
        ));
    }

    #[test]
    fn normal_creator_link_is_not_a_scam() {
        assert!(!looks_like_scam(
            "New MrBeast video: https://youtube.com/watch?v=example"
        ));
    }

    #[test]
    fn repeated_messages_trigger_spam_detection() {
        let mut tracker = SpamTracker::default();
        assert!(!tracker.is_spam(1, 2, "same message"));
        assert!(!tracker.is_spam(1, 2, "same message"));
        assert!(tracker.is_spam(1, 2, "same message"));
    }
}
