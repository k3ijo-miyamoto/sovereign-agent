use common::ChatMessage;

pub struct History {
    system: Option<String>,
    messages: Vec<ChatMessage>,
}

impl History {
    pub fn new(system: impl Into<String>) -> Self {
        Self { system: Some(system.into()), messages: vec![] }
    }

    pub fn push(&mut self, msg: ChatMessage) {
        self.messages.push(msg);
    }

    pub fn build_messages(&self) -> Vec<ChatMessage> {
        self.build_messages_with_suffix(None)
    }

    /// XML モード用: system prompt にツール定義サフィックスを付加して返す
    pub fn build_messages_with_suffix(&self, suffix: Option<&str>) -> Vec<ChatMessage> {
        let mut out = Vec::with_capacity(self.messages.len() + 1);
        if let Some(sys) = &self.system {
            let content = match suffix {
                Some(s) => format!("{sys}{s}"),
                None => sys.clone(),
            };
            out.push(ChatMessage::system(content));
        }
        out.extend(self.messages.iter().cloned());
        out
    }
}
