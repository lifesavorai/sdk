# Weather Alerts — Usage Guide

## Overview

Weather Alerts delivers real-time severe weather notifications for your
configured locations. Once set up, the skill monitors weather services and
pushes alerts through the Life Savor agent so Savo can proactively warn you
about dangerous conditions.

## Getting Started

1. **Install the skill** from the Life Savor marketplace.
2. **Complete setup** — the wizard walks you through two steps:
   - **API Credentials** — enter your OpenWeatherMap API key.
   - **Alert Preferences** — choose your location, units, alert types, and
     polling interval.
3. **Start receiving alerts** — Savo will notify you when severe weather is
   detected in your area.

## Obtaining an API Key

1. Create a free account at [openweathermap.org](https://openweathermap.org).
2. Navigate to **API Keys** in your account dashboard.
3. Copy the default key or generate a new one.
4. Paste the key into the **API Credentials** setup step.

> Free-tier keys support up to 60 requests per minute, which is sufficient
> for polling intervals of 5 minutes or longer.

## Configuration Reference

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `api_key` | string (secret) | Yes | Your OpenWeatherMap API key |
| `location` | string | Yes | City name or `lat,lon` coordinates |
| `units` | string | No | `metric` (default) or `imperial` |
| `alert_types` | array of strings | No | Alert categories to monitor (default: `["severe", "warning"]`) |
| `polling_interval` | integer | No | Minutes between checks (default: `15`) |
| `notifications_enabled` | boolean | No | Enable push notifications (default: `true`) |

## Talking to Savo

Once configured, you can ask Savo questions like:

- *"What's the weather forecast for today?"*
- *"Are there any severe weather alerts in my area?"*
- *"Change my weather location to New York."*
- *"Turn off weather notifications."*

## Troubleshooting

| Symptom | Likely Cause | Fix |
|---------|-------------|-----|
| "Invalid API key" during setup | Key is expired or mistyped | Regenerate the key on openweathermap.org |
| No alerts received | Polling interval too long or no active alerts | Lower the interval or check the weather service status |
| Wrong temperature units | Units field not set | Reconfigure the skill and select `metric` or `imperial` |

## Further Reading

- [OpenWeatherMap API docs](https://openweathermap.org/api)
- [Life Savor Skill Developer Guide](https://developer.lifesavor.ai/docs/skills)
