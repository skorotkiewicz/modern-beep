# Modern Beep 🔊

A modern Rust alternative to the classic `beep` command with support for notifications, webhooks, and remote audio playback.

## Features

- **🎵 Audio Generation**: Generate beep tones with customizable frequency, duration, and repetitions
- **📱 Push Notifications**: Send notifications via Pushover
- **🌐 Webhooks**: HTTP POST/GET requests with JSON support
- **🔊 Audio Playback**: Play local files or remote audio URLs
- **⚙️ YAML Configuration**: Flexible configuration system
- **🔄 Multiple Repeats**: Configure delays between beeps
- **🎛️ Volume Control**: Automatic volume adjustment
- **📝 Verbose Mode**: Optional detailed output with `-v` flag

## Installation

### Prerequisites

Make sure you have Rust installed. If not, install it from [rustup.rs](https://rustup.rs/).

### Build from Source

```bash
# Clone the repository
git clone https://github.com/skorotkiewicz/modern-beep
cd modern-beep

# Build the project
cargo build --release

# Install to your PATH
cp target/release/beep ~/.local/bin/
# or
sudo cp target/release/beep /usr/local/bin/
```

### Dependencies

The following system packages might be required depending on your Linux distribution:

```bash
# Ubuntu/Debian
sudo apt install libasound2-dev pkg-config

# Fedora/CentOS/RHEL
sudo dnf install alsa-lib-devel pkgconf-pkg-config

# Arch Linux
sudo pacman -S alsa-lib pkg-config
```

## Usage

### Basic Examples

```bash
# Simple beep
beep

# Custom frequency and duration
beep -f 800 -l 500

# Multiple beeps with delay
beep -f 1200 -l 200 -r 3 -d 300

# Send notification with beep
beep -D "Process completed!" -t "System Alert"

# Send JSON data to webhook (no local sound)
beep -D '{"status": "success", "timestamp": "2025-01-15T10:30:00Z"}' --no-sound

# High priority notification with verbose output
beep -D "Critical error!" -p 2 -v

# Verbose mode to see all operations
beep -v -f 440 -l 1000 -D "Verbose beep"
```

### Command Line Options

```
Usage: beep [OPTIONS]

Options:
  -f, --frequency <FREQUENCY>  Frequency in Hz [default: 1000]
  -l, --length <LENGTH>        Length in milliseconds [default: 200]
  -r, --repeats <REPEATS>      Number of repetitions [default: 1]
  -d, --delay <DELAY>          Delay between repetitions in ms [default: 100]
  -D, --data <DATA>            Message to send
  -t, --title <TITLE>          Notification title
  -p, --priority <PRIORITY>    Priority (Pushover only: -2, -1, 0, 1, 2)
      --no-sound               Don't play sound locally
  -c, --config <CONFIG>        Path to configuration file
      --sample-config          Show sample configuration
  -v, --verbose                Verbose output
  -h, --help                   Print help
```

## Configuration

Modern Beep uses a YAML configuration file located at `~/.config/beep.yaml`.

### Generate Sample Configuration

```bash
beep --sample-config > ~/.config/beep.yaml
```

### Configuration Options

```yaml
# Modern Beep Configuration

# Pushover notifications
pushover:
  api_token: "your_api_token_here"
  user_key: "your_user_key_here"
  device: "optional_device_name"

# HTTP Webhook
webhook:
  url: "https://example.com/notifications"
  method: "POST"  # optional, defaults to POST
  headers:        # optional headers
    Authorization: "Bearer your_token"
    Content-Type: "application/json"

# Sound file playback
sound:
  file: "/path/to/notification.wav"        # local file
  url: "https://example.com/sound.mp3"     # or remote URL
```

## Notification Services

### Pushover Setup

1. Create an account at [pushover.net](https://pushover.net/)
2. Create a new application to get an API token
3. Find your user key in your dashboard
4. Add both to your configuration file

```yaml
pushover:
  api_token: "azGDORePK8gMaC0QOYAMyEEuzJnyUi"
  user_key: "uQiRzpo4DXghDmr9QzzfQu27cmVRsG"
```

Priority levels:
- `-2`: Lowest priority, no notification
- `-1`: Low priority, quiet
- `0`: Normal priority (default)
- `1`: High priority, bypass quiet hours
- `2`: Emergency priority, requires acknowledgment

### Webhook Setup

Modern Beep can send HTTP requests to any endpoint:

```yaml
webhook:
  url: "https://hooks.slack.com/services/YOUR/SLACK/WEBHOOK"
  method: "POST"
  headers:
    Content-Type: "application/json"
```

The webhook will receive either:
- **JSON data**: If `-D` contains valid JSON
- **Plain text**: If `-D` contains regular text

Example JSON payload:
```bash
beep -D '{"message": "Build completed", "status": "success", "duration": "2m 34s"}'
```

### Audio Playback

#### Local Files
```yaml
sound:
  file: "/usr/share/sounds/alsa/Front_Right.wav"
```

#### Remote URLs
```yaml
sound:
  url: "https://example.com/notification.mp3"
```

Supported formats: MP3, WAV, FLAC, OGG, and more (via `rodio` library).

**Note**: URL takes precedence over local file if both are specified.

## Integration Examples

### Shell Scripts

```bash
#!/bin/bash
# Long running process with verbose output
echo "Starting backup..."
rsync -av /home/user/ /backup/
beep -v -D "Backup completed successfully" -t "System Backup"
```

### Cron Jobs

```bash
# Notify when disk space is low
0 */6 * * * /usr/bin/df -h | /usr/bin/awk '$5 > 90 {print $0}' | /usr/bin/wc -l | /usr/bin/awk '{if($1>0) system("beep -D \"Disk space warning\" -p 1")}'
```

### GitHub Actions

```yaml
- name: Notify on deployment
  run: |
    beep -D '{"repository": "${{ github.repository }}", "status": "deployed", "commit": "${{ github.sha }}"}' \
         -t "GitHub Deployment"
```

## Advanced Usage

### Multiple Notifications

You can configure multiple services simultaneously. Modern Beep will send to all configured services:

```yaml
pushover:
  api_token: "your_token"
  user_key: "your_key"

webhook:
  url: "https://your-webhook.com/notify"

sound:
  url: "https://example.com/alert.mp3"
```

### Custom Configuration Path

```bash
beep -c /path/to/custom-config.yaml -D "Using custom config"
```

### Silent Notifications

```bash
# Only send notifications, no local sound
beep --no-sound -D "Silent notification"

# Only play local beep, no notifications
beep -f 440 -l 1000

# Verbose mode to see what's happening
beep -v -f 800 -r 3 -d 200 -D "Testing with verbose output"
```

## Verbose Mode

Use the `-v` or `--verbose` flag to see detailed information about what Modern Beep is doing:

```bash
# See all operations
beep -v -D "Test message" -t "Test"

# Output example:
# ✓ Pushover notification sent
# ✓ Webhook sent to https://example.com/notifications  
# ✓ Played sound file: /usr/share/sounds/notification.wav
# 🔊 Beep 1000 Hz for 200 ms
```

**Verbose shows:**
- ✅ Successful Pushover notifications
- ✅ Successful webhook deliveries  
- ✅ Audio file playback confirmations
- 🔊 Generated beep tone details

## Troubleshooting

### Audio Issues

```bash
# Test if audio system works
beep -f 440 -l 1000

# Check available audio devices
pactl list short sinks
```

### Network Issues

```bash
# Test webhook connectivity
curl -X POST https://your-webhook.com/test

# Test Pushover API
curl -s \
  --form-string "token=YOUR_API_TOKEN" \
  --form-string "user=YOUR_USER_KEY" \
  --form-string "message=Test message" \
  https://api.pushover.net/1/messages.json
```

### Configuration Issues

```bash
# Validate YAML syntax
beep --sample-config | yq eval '.' -

# Check configuration location
ls -la ~/.config/beep.yaml
```

## Development

### Building

```bash
cargo build
```

### Testing

```bash
cargo test
```

### Dependencies

Major dependencies:
- `clap`: Command line argument parsing
- `serde`: Serialization framework
- `tokio`: Async runtime
- `reqwest`: HTTP client
- `rodio`: Audio playback
- `cpal`: Cross-platform audio library

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## Acknowledgments

- Inspired by the classic Unix `beep` command
- Built with the amazing Rust audio ecosystem
- Thanks to the Pushover team for their excellent API

---

**Modern Beep** - Making notifications modern, one beep at a time! 🚀