# discord/tester

Discord utilities for OSL - send messages, embeds, and notifications to Discord webhooks.

## Installation

```bash
oslc install discord/tester
```

## Quick Start

```osl
import discord/tester;

let WEBHOOK_URL = "https://discord.com/api/webhooks/YOUR/WEBHOOK/HERE";

// Send a simple message
send_message(WEBHOOK_URL, "Hello from OSL!");

// Send an embed
send_embed(WEBHOOK_URL, "Title", "Description here", 3066993);

// Send with custom username
send_as(WEBHOOK_URL, "Hello!", "My Bot", "https://example.com/avatar.png");
```

## Template Files

Copy templates from the `templates/` folder into your project:

| Template | Description |
|----------|-------------|
| `webhook.osl` | Basic webhook setup with route examples |
| `embed.osl` | Rich embed notifications with color presets |
| `cicd.osl` | CI/CD deploy notifications |
| `logging.osl` | Log monitoring with log levels |
| `activity.osl` | User activity tracking (signup, login, etc.) |
| `cron.osl` | Cron job monitoring |
| `uptime.osl` | Server uptime monitoring |
| `database.osl` | Database alerts (queries, connections, backups) |
| `metrics.osl` | Server metrics dashboard |
| `github.osl` | GitHub webhook events |

## Functions

### send_message(webhook_url, content)
Send a simple text message to Discord.

### send_embed(webhook_url, title, description, color)
Send a rich embed message.
- `title`: Embed title
- `description`: Main text content  
- `color`: Decimal color value (e.g., 3066993 = green)

### send_as(webhook_url, content, username, avatar_url)
Send a message with custom Discord bot username/avatar.

## Color Presets (Decimal)

```osl
let GREEN = 3066993;
let RED = 15158332;
let ORANGE = 15105570;
let BLUE = 3447003;
let PURPLE = 8338334;
let YELLOW = 16073277;
```

## Getting a Webhook URL

1. Open Discord
2. Go to Server Settings > Integrations > Webhooks
3. Create a new webhook or click an existing one
4. Copy the webhook URL

## Example: Server Health Check

```osl
import discord/tester;

let WEBHOOK_URL = "YOUR_WEBHOOK_URL";

server HealthMonitor {
    port: 8080
}

route GET /health (req) {
    let cpu = get_cpu_usage();
    let memory = get_memory_percent();
    
    if cpu > 90 {
        send_embed(WEBHOOK_URL, "⚠️ High CPU", 
            "CPU usage: " + string(cpu) + "%", 15158332);
    }
    
    return { "status": "ok" };
}
```

## License

MIT
