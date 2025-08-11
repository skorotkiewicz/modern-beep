use anyhow::Result;
use clap::Parser;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Sample, SampleFormat};
use dasp_sample::{FromSample};
use dirs::home_dir;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;
use tokio::time::sleep;

#[derive(Parser)]
#[command(name = "beep")]
#[command(about = "Modern beep alternative with notifications")]
struct Args {
    /// Frequency in Hz
    #[arg(short, long, default_value = "1000")]
    frequency: f32,

    /// Length in milliseconds
    #[arg(short, long, default_value = "200")]
    length: u64,

    /// Number of repetitions
    #[arg(short, long, default_value = "1")]
    repeats: u32,

    /// Delay between repetitions in ms
    #[arg(short, long, default_value = "100")]
    delay: u64,

    /// Message to send
    #[arg(short = 'D', long)]
    data: Option<String>,

    /// Notification title
    #[arg(short, long)]
    title: Option<String>,

    /// Priority (Pushover only: -2, -1, 0, 1, 2)
    #[arg(short, long)]
    priority: Option<i8>,

    /// Don't play sound locally
    #[arg(long)]
    no_sound: bool,

    /// Path to configuration file
    #[arg(short, long)]
    config: Option<PathBuf>,

    /// Show sample configuration
    #[arg(long)]
    sample_config: bool,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Debug, Deserialize, Serialize)]
struct Config {
    #[serde(skip_serializing_if = "Option::is_none")]
    pushover: Option<PushoverConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    webhook: Option<WebhookConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    sound: Option<SoundConfig>,
}

#[derive(Debug, Deserialize, Serialize)]
struct PushoverConfig {
    api_token: String,
    user_key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    device: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct WebhookConfig {
    url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    method: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    headers: Option<HashMap<String, String>>,
}

#[derive(Debug, Deserialize, Serialize)]
struct SoundConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    url: Option<String>,
}

fn get_config_path(custom_path: Option<PathBuf>) -> PathBuf {
    if let Some(path) = custom_path {
        return path;
    }
    
    if let Some(home) = home_dir() {
        home.join(".config").join("beep.yaml")
    } else {
        PathBuf::from("beep.yaml")
    }
}

fn load_config(path: &PathBuf) -> Result<Option<Config>> {
    if !path.exists() {
        return Ok(None);
    }
    
    let content = fs::read_to_string(path)?;
    let config: Config = serde_yaml::from_str(&content)?;
    Ok(Some(config))
}

fn print_sample_config() {
    let sample = r#"# Modern Beep Configuration (~/.config/beep.yaml)
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
"#;
    println!("{}", sample);
}

async fn send_pushover_notification(
    config: &PushoverConfig, 
    message: &str, 
    title: Option<&str>,
    priority: Option<i8>,
    verbose: bool
) -> Result<()> {
    let client = Client::new();
    let mut params = HashMap::new();
    
    params.insert("token", config.api_token.clone());
    params.insert("user", config.user_key.clone());
    params.insert("message", message.to_string());
    
    if let Some(title) = title {
        params.insert("title", title.to_string());
    }
    
    if let Some(device) = &config.device {
        params.insert("device", device.clone());
    }
    
    if let Some(priority) = priority {
        params.insert("priority", priority.to_string());
    }
    
    let response = client
        .post("https://api.pushover.net/1/messages.json")
        .form(&params)
        .send()
        .await?;
    
    if response.status().is_success() {
        if verbose {
            println!("✓ Pushover notification sent");
        }
    } else {
        eprintln!("✗ Pushover error: {}", response.status());
    }
    
    Ok(())
}

async fn send_webhook_notification(
    config: &WebhookConfig, 
    data: &str,
    verbose: bool
) -> Result<()> {
    let client = Client::new();
    let method = config.method.as_deref().unwrap_or("POST");
    
    let mut request = match method.to_uppercase().as_str() {
        "GET" => client.get(&config.url),
        "PUT" => client.put(&config.url),
        "PATCH" => client.patch(&config.url),
        _ => client.post(&config.url),
    };
    
    // Próbuj sparsować jako JSON, jeśli się nie uda - wyślij jako tekst
    if let Ok(json_value) = serde_json::from_str::<Value>(data) {
        request = request.json(&json_value);
    } else {
        request = request.body(data.to_string());
    }
    
    // Dodaj niestandardowe nagłówki
    if let Some(headers) = &config.headers {
        for (key, value) in headers {
            request = request.header(key, value);
        }
    }
    
    let response = request.send().await?;
    
    if response.status().is_success() {
        if verbose {
            println!("✓ Webhook sent to {}", config.url);
        }
    } else {
        eprintln!("✗ Webhook error: {}", response.status());
    }
    
    Ok(())
}

fn play_sound_file(path: &str, verbose: bool) -> Result<()> {
    use rodio::{Decoder, OutputStream, Sink};
    use std::fs::File;
    use std::io::BufReader;
    
    let (_stream, stream_handle) = OutputStream::try_default()?;
    let sink = Sink::try_new(&stream_handle)?;
    
    let file = BufReader::new(File::open(path)?);
    let source = Decoder::new(file)?;
    
    sink.append(source);
    sink.sleep_until_end();
    
    if verbose {
        println!("✓ Played sound file: {}", path);
    }
    Ok(())
}

async fn play_sound_url(url: &str, verbose: bool) -> Result<()> {
    use rodio::{Decoder, OutputStream, Sink};
    use std::io::Cursor;
    
    let client = Client::new();
    let response = client.get(url).send().await?;
    
    if !response.status().is_success() {
        return Err(anyhow::anyhow!("Failed to download audio file: {}", response.status()));
    }
    
    let bytes = response.bytes().await?;
    let cursor = Cursor::new(bytes);
    
    let (_stream, stream_handle) = OutputStream::try_default()?;
    let sink = Sink::try_new(&stream_handle)?;
    
    let source = Decoder::new(cursor)?;
    sink.append(source);
    sink.sleep_until_end();
    
    if verbose {
        println!("✓ Played sound from URL: {}", url);
    }
    Ok(())
}

fn generate_beep_tone(frequency: f32, duration_ms: u64) -> Result<()> {
    let host = cpal::default_host();
    let device = host.default_output_device()
        .ok_or_else(|| anyhow::anyhow!("No audio device available"))?;
    
    let config = device.default_output_config()?;
    
    match config.sample_format() {
        SampleFormat::F32 => run_beep::<f32>(&device, &config.into(), frequency, duration_ms),
        SampleFormat::I16 => run_beep::<i16>(&device, &config.into(), frequency, duration_ms),
        SampleFormat::U16 => run_beep::<u16>(&device, &config.into(), frequency, duration_ms),
        _ => Err(anyhow::anyhow!("Unsupported sample format")),
    }
}

fn run_beep<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    frequency: f32,
    duration_ms: u64,
) -> Result<()>
where
    T: Sample + cpal::SizedSample + Send + 'static,
    T: FromSample<f32>,
{
    let sample_rate = config.sample_rate.0 as f32;
    let channels = config.channels as usize;
    
    let mut sample_clock = 0f32;
    let total_samples = (sample_rate * (duration_ms as f32 / 1000.0)) as usize;
    let mut samples_played = 0;
    
    let stream = device.build_output_stream(
        config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            for frame in data.chunks_mut(channels) {
                if samples_played >= total_samples {
                    for sample in frame.iter_mut() {
                        *sample = T::EQUILIBRIUM;
                    }
                    continue;
                }
                
                let value = (sample_clock * frequency * 2.0 * std::f32::consts::PI / sample_rate).sin();
                let sample_f32 = value * 0.3; // Reduce volume
                let sample = T::from_sample(sample_f32);
                
                for sample_out in frame.iter_mut() {
                    *sample_out = sample;
                }
                
                sample_clock = (sample_clock + 1.0) % sample_rate;
                samples_played += 1;
            }
        },
        |err| eprintln!("Audio stream error: {}", err),
        None,
    )?;
    
    stream.play()?;
    std::thread::sleep(Duration::from_millis(duration_ms + 50)); // Add buffer
    
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    
    if args.sample_config {
        print_sample_config();
        return Ok(());
    }
    
    let config_path = get_config_path(args.config);
    let config = load_config(&config_path)?;
    
    // Prepare message
    let message = args.data.as_deref().unwrap_or("Beep!");
    let title = args.title.as_deref();
    
    // Send notifications if configured
    if let Some(config) = &config {
        if let Some(pushover_config) = &config.pushover {
            if let Err(e) = send_pushover_notification(pushover_config, message, title, args.priority, args.verbose).await {
                eprintln!("Pushover error: {}", e);
            }
        }
        
        if let Some(webhook_config) = &config.webhook {
            if let Err(e) = send_webhook_notification(webhook_config, message, args.verbose).await {
                eprintln!("Webhook error: {}", e);
            }
        }
        
        // Play sound file if configured
        if let Some(sound_config) = &config.sound {
            if let Some(url) = &sound_config.url {
                if let Err(e) = play_sound_url(url, args.verbose).await {
                    eprintln!("Error playing sound from URL: {}", e);
                }
            } else if let Some(file_path) = &sound_config.file {
                if let Err(e) = play_sound_file(file_path, args.verbose) {
                    eprintln!("Error playing sound file: {}", e);
                }
            }
        }
    }
    
    // Play local beep if not disabled
    if !args.no_sound {
        for i in 0..args.repeats {
            if i > 0 {
                sleep(Duration::from_millis(args.delay)).await;
            }
            
            if let Err(e) = generate_beep_tone(args.frequency, args.length) {
                eprintln!("Error generating sound: {}", e);
                // Fallback to system beep
                print!("\x07");
            } else if args.verbose {
                println!("🔊 Beep {} Hz for {} ms", args.frequency, args.length);
            }
        }
    }
    
    Ok(())
}