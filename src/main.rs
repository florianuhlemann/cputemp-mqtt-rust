// FLORIAN UHLEMANN (C) 2023
// ToDo:
// - get hostname and use as hostname for publishing messages and specifying topic
// - mqtt to "cluster/computeblade/1/cpu_temp"


use tokio::{time, task};
use std::fs;
use rumqttc::{MqttOptions, AsyncClient, QoS};


async fn read_float_from_file(input: &str) -> Result<f32, std::num::ParseFloatError> {
    let parsed = input.parse::<f32>()?;
    let result = parsed / 1000.0;
    Ok(result)
}


async fn get_cpu_temperature() -> f32 {
    let filename = "/sys/class/thermal/thermal_zone0/temp";
    let input = fs::read_to_string(filename).expect("Error reading file");
    let input = input.trim_end();
    read_float_from_file(&input).await.unwrap()
}


async fn send_temp_to_mqtt(my_mqtt_client: &rumqttc::AsyncClient) {
    let result = get_cpu_temperature().await;
    println!("Result: {:.2}°C", result);
    send_mqtt_message(&my_mqtt_client, result.to_string()).await;
}


async fn setup_mqtt_connection() -> Result<rumqttc::AsyncClient, std::num::ParseFloatError> {
    let mqttoptions = MqttOptions::new("computeblade_1", "192.168.1.15", 1883);
    let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);
    task::spawn(async move {
        while let Ok(_notification) = eventloop.poll().await {
            //println!("Received = {:?}", notification);
        }
    });
    
    Ok(client)
}

async fn send_mqtt_message(my_mqtt_client: &rumqttc::AsyncClient, input: String) {
    my_mqtt_client.publish("cluster/computeblade/1/cpu_temp", QoS::AtLeastOnce, false, input).await.unwrap();
}


// Execute function every 5 seconds, but immediately after starting once.
#[tokio::main]
async fn main() {

    // setup MQTT client and hand over pointer to client for publish
    let my_mqtt_client = &setup_mqtt_connection().await.unwrap();

    // start interval timer to grab CPU temp and send via mqtt
    let mut interval = time::interval(time::Duration::from_secs(5));
    loop {
        interval.tick().await;
        send_temp_to_mqtt(my_mqtt_client).await;
    }
}

