pub struct EmailService {
    smtp_host: String,
    smtp_port: u16,
    from: String,
}

impl EmailService {
    pub fn new(smtp_host: &str, smtp_port: u16, from: &str) -> Self {
        EmailService {
            smtp_host: smtp_host.to_string(),
            smtp_port,
            from: from.to_string(),
        }
    }
    
    pub fn send(&self, to: &str, subject: &str, body: &str) -> Result<(), String> {
        println!("Sending email from {} to {}", self.from, to);
        println!("Subject: {}", subject);
        println!("Body: {}", body);
        Ok(())
    }
    
    pub fn send_template(&self, to: &str, template: &str, vars: &[(String, String)]) -> Result<(), String> {
        let mut body = template.to_string();
        for (key, value) in vars {
            body = body.replace(&format!("{{{}}}", key), value);
        }
        self.send(to, "Template Email", &body)
    }
    
    pub fn send_with_attachments(&self, to: &str, subject: &str, body: &str, _attachments: &[&str]) -> Result<(), String> {
        self.send(to, subject, body)
    }
}

pub struct TemplateEngine {
    templates: std::collections::HashMap<String, String>,
}

impl TemplateEngine {
    pub fn new() -> Self {
        TemplateEngine {
            templates: std::collections::HashMap::new(),
        }
    }
    
    pub fn register(&mut self, name: &str, template: &str) {
        self.templates.insert(name.to_string(), template.to_string());
    }
    
    pub fn render(&self, name: &str, vars: &[(String, String)]) -> String {
        let template = self.templates.get(name).cloned().unwrap_or_default();
        let mut result = template;
        for (key, value) in vars {
            result = result.replace(&format!("{{{}}}", key), value);
        }
        result
    }
}
