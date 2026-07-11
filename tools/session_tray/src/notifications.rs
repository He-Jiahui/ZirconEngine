use std::time::{Duration, Instant};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrayNotification {
    pub title: String,
    pub body: String,
}

pub struct NotificationPolicy {
    last_state: Option<String>,
    last_sent: Option<Instant>,
    minimum_interval: Duration,
}

impl Default for NotificationPolicy {
    fn default() -> Self {
        Self {
            last_state: None,
            last_sent: None,
            minimum_interval: Duration::from_secs(30),
        }
    }
}

impl NotificationPolicy {
    pub fn state_change(&mut self, state: &str, detail: &str) -> Option<TrayNotification> {
        if self.last_state.as_deref() == Some(state) {
            return None;
        }
        let now = Instant::now();
        if self
            .last_sent
            .is_some_and(|previous| now.duration_since(previous) < self.minimum_interval)
        {
            self.last_state = Some(state.to_owned());
            return None;
        }
        self.last_state = Some(state.to_owned());
        self.last_sent = Some(now);
        Some(TrayNotification {
            title: format!("Zircon Coordinator：{state}"),
            body: detail.to_owned(),
        })
    }
}

#[cfg(windows)]
pub fn show_native(notification: &TrayNotification) -> Result<(), crate::TrayError> {
    use windows::core::HSTRING;
    use windows::Data::Xml::Dom::XmlDocument;
    use windows::UI::Notifications::{ToastNotification, ToastNotificationManager};

    let document = XmlDocument::new()?;
    let payload = format!(
        "<toast><visual><binding template=\"ToastGeneric\"><text>{}</text><text>{}</text></binding></visual></toast>",
        escape_xml(&notification.title),
        escape_xml(&notification.body),
    );
    document.LoadXml(&HSTRING::from(payload))?;
    let toast = ToastNotification::CreateToastNotification(&document)?;
    let notifier = ToastNotificationManager::CreateToastNotifierWithId(&HSTRING::from(
        "ZirconEngine.SessionCoordinator",
    ))?;
    notifier.Show(&toast)?;
    Ok(())
}

#[cfg(not(windows))]
pub fn show_native(_notification: &TrayNotification) -> Result<(), crate::TrayError> {
    Ok(())
}

fn escape_xml(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unchanged_state_does_not_repeat_notification() {
        let mut policy = NotificationPolicy::default();
        assert!(policy.state_change("healthy", "ready").is_some());
        assert!(policy.state_change("healthy", "still ready").is_none());
    }

    #[test]
    fn notification_xml_is_escaped() {
        assert_eq!("a&amp;b&lt;c&gt;&quot;&apos;", escape_xml("a&b<c>\"'"));
    }
}
