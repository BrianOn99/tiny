extern crate notify_rust;

pub use tui::messaging::Timestamp;
use tui::MsgTarget;

use self::notify_rust::Notification;

/// Destktop notification handler
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Notifier { Off, Mentions, Messages }

fn notify(summary: &str, body: &str) {
    Notification::new()
        .summary(summary)
        .body(body)
        .show()
        .unwrap();
}

impl Notifier {
    pub fn notify_privmsg(
        &mut self,
        sender: &str,
        msg: &str,
        target: &MsgTarget,
        our_nick: &str,
        mention: bool,
    ) {
        if our_nick == sender {
            return;
        }
        match *target {
            MsgTarget::Chan { chan_name, .. } => {
                if *self == Notifier::Messages || (*self == Notifier::Mentions && mention) {
                    notify(&format!("{} in {}", sender, chan_name), &format!("{}", msg))
                }
            }
            MsgTarget::User { nick: ref nick_sender, .. } => {
                if *self != Notifier::Off {
                    notify(
                        &format!("{} sent a private message", nick_sender),
                        &format!("{}", msg),
                    )
                }
            }
            _ => {}
        }
    }
}
