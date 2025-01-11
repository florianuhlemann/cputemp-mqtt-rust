// FLORIAN UHLEMANN (C) 2025
use tokio::{time, task};
use std::fs;
use rumqttc::{MqttOptions, AsyncClient, QoS};
use std::error::Error;
use std::env;

#[derive(Debug)]
struct AppError(String);

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for AppError {}

struct AppConfig {
    silent: bool,
}

impl AppConfig {
    fn new() -> Self {
        let args: Vec<String> = env::args().collect();
        let silent = args.contains(&"--silent".to_string());
        AppConfig { silent }
    }

    fn log(&self, message: &str) {
        if !self.silent {
            println!("{}", message);
        }
    }
}

async fn read_float_from_file(input: &str) -> Result<f32, Box<dyn Error>> {
    let parsed = input.parse::<f32>()
        .map_err(|e| AppError(format!("Failed to parse temperature: {}", e)))?;
    let result = parsed / 1000.0;
    Ok(result)
}

async fn get_hostname() -> Result<String, Box<dyn Error>> {
    let hostname = fs::read_to_string("/etc/hostname")
        .map_err(|e| AppError(format!("Failed to read hostname: {}", e)))?;
    let hostname = hostname.trim_end();
    // Get first segment before dot
    let hostname = hostname.split('.')
        .next()
        .ok_or_else(|| AppError("Invalid hostname format".to_string()))?;
    Ok(hostname.to_string())
}

async fn get_cpu_temperature() -> Result<f32, Box<dyn Error>> {
    let filename = "/sys/class/thermal/thermal_zone0/temp";
    let input = fs::read_to_string(filename)
        .map_err(|e| AppError(format!("Failed to read temperature file: {}", e)))?;
    let input = input.trim_end();
    read_float_from_file(&input).await
}

async fn send_temp_to_mqtt(my_mqtt_client: &rumqttc::AsyncClient, topic: &str, config: &AppConfig) -> Result<(), Box<dyn Error>> {
    let result = get_cpu_temperature().await?;
    config.log(&format!("Temperature: {:.2}°C", result));
    send_mqtt_message(my_mqtt_client, topic, result.to_string()).await?;
    Ok(())
}

async fn setup_mqtt_connection(hostname: &str) -> Result<rumqttc::AsyncClient, Box<dyn Error>> {
    let mqttoptions = MqttOptions::new(hostname, "192.168.1.15", 1883);
    let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);
    
    task::spawn(async move {
        while let Ok(_notification) = eventloop.poll().await {
            //println!("Received = {:?}", notification);
        }
    });
    
    Ok(client)
}

async fn send_mqtt_message(my_mqtt_client: &rumqttc::AsyncClient, topic: &str, input: String) -> Result<(), Box<dyn Error>> {
    my_mqtt_client.publish(topic, QoS::AtLeastOnce, false, input)
        .await
        .map_err(|e| AppError(format!("Failed to publish MQTT message: {}", e)))?;
    Ok(())
}

#[tokio::main]
async fn main() {
    let config = AppConfig::new();

    // Get hostname first
    let hostname = match get_hostname().await {
        Ok(h) => h,
        Err(e) => {
            eprintln!("Failed to get hostname, using default: {}", e);
            "computeblade_default".to_string()
        }
    };
    
    config.log(&format!("Using hostname: {}", hostname));
    let topic = format!("cluster/{}/cpu_temp", hostname);
    
    // Setup MQTT client with retry mechanism
    let my_mqtt_client = loop {
        match setup_mqtt_connection(&hostname).await {
            Ok(client) => break client,
            Err(e) => {
                eprintln!("Failed to connect to MQTT broker: {}. Retrying in 5 seconds...", e);
                time::sleep(time::Duration::from_secs(5)).await;
            }
        }
    };
    
    // Start interval timer to grab CPU temp and send via mqtt
    let mut interval = time::interval(time::Duration::from_secs(5));
    loop {
        interval.tick().await;
        if let Err(e) = send_temp_to_mqtt(&my_mqtt_client, &topic, &config).await {
            eprintln!("Error sending temperature: {}. Will retry on next interval.", e);
        }
    }
}