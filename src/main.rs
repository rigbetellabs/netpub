use serde_json::{json, Value};
use std::error::Error;
use std::process::Stdio;
use std::process::{Command, Output};
use regex::Regex;

fn center_ip_address(ip_address: &str, total_width: usize) -> String {
    let ip_length = ip_address.len();
    if total_width <= ip_length {
        return ip_address.to_string();
    }

    let padding = (total_width - ip_length) / 2;
    let formatted_string = format!("{:width$}{ip_address}{:width$}", "", ip_address = ip_address, width = padding);
    formatted_string
}

fn grep_ssid() -> Result<String, Box<dyn Error>> {
    // Call the command `iw dev`
    let output = Command::new("iw")
        .arg("dev")
        .output()?;
    
    // Convert the output to a string
    let output_str = String::from_utf8_lossy(&output.stdout);

    // Regular expression pattern to match the line containing the SSID
    let ssid_pattern = r"\bssid\s(.+)\b";

    // Compile the regex pattern
    let re = Regex::new(ssid_pattern)?;

    // Search for the SSID pattern in the output
    if let Some(cap) = re.captures(&output_str) {
        // Extract and return the SSID
        if let Some(ssid) = cap.get(1) {
            return Ok(ssid.as_str().trim().to_string());
        }
    }
    return Ok(String::from("FETCHING-IP"));

    Err("SSID not found".into())
}

#[allow(dead_code)]
fn interface_name() -> Option<String> {
    match Command::new("sudo")
        .args(&["nmcli", "device", "status"])
        .output()
    {
        Ok(output) => {
            if let Ok(result) = String::from_utf8(output.stdout) {
                if let Some(wifi_line) = result.lines().find(|line| line.contains("wifi ")) {
                    if let Some(device_name) = wifi_line.split_whitespace().next() {
                        return Some(device_name.to_string());
                    }
                }
            }
            None
        }
        Err(e) => {
            eprintln!("Error executing the command: {}", e);
            None
        }
    }
}

fn which_type_on() -> Result<String, Box<dyn Error>> {
    match Command::new("sudo")
        .args(&["nmcli", "connection", "show", "--active"])
        .stdout(Stdio::piped())
        .output()
    {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let lines: Vec<&str> = stdout.trim().split('\n').collect();

            if lines.is_empty() {
                return Ok(String::from("no_connection"));
            }

            if let Some(first_line) = lines.get(1) {
                if let Some(second_line) = lines.get(2) {
                    if !second_line.trim().is_empty() {
                        return Ok(String::from("wifi"));
                    }
                }

                let connection_name = first_line.split_whitespace().next().unwrap_or("");

                if connection_name == "Hotspot" {
                    return Ok(String::from("hotspot"));
                } else {
                    return Ok(String::from("wifi"));
                }
            }
            Ok(String::from("no_connection"))
        }
        Err(e) => {
            eprintln!("Error executing the command: {}", e);
            Ok(String::from("no_connection"))
        }
        Err(_) => Ok(String::from("no_connection")),
    }
}

fn get_json_str() -> Result<String, Box<dyn Error>> {
    let some_ip_address_output: Output = Command::new("robonet-getip")
        .stderr(Stdio::null())
        .output()?;

    let some_ip_address_str = String::from("192.168.0.178");
    let some_ip_address = some_ip_address_str.trim();

    let mut network_status = 0; // Default network status: no connection
    let mut type_of_network = String::new();

    let mut fetch_ip_address = String::new();
    let mut ip_address = String::new();
    let mut internal_ip_address = String::new();

    let mut json_obj = json!({});

    let some_ip_address_output = Command::new("robonet-getip")
        .stderr(Stdio::null())
        .output()?;
    let some_ip_address_str = String::from_utf8_lossy(&some_ip_address_output.stdout);
    let some_ip_address = some_ip_address_str.trim();
    //println!("{}", some_ip_address);

    match which_type_on() {
        Ok(network_type) => {
            type_of_network = network_type;
            network_status = match type_of_network.as_str() {
                "hotspot" => 2,
                "wifi" => 1,
                "no_connection" => 0,
                _ => 4,
            };
        }
        Err(_) => network_status = 4,
    }

    if let Some(split_index) = some_ip_address.find(' ') {
        ip_address = some_ip_address[..split_index].to_string();
        internal_ip_address = some_ip_address[split_index + 1..].to_string();
    }

    match network_status {
        1 => {
            json_obj = json!({
                "mode": network_status,
                "status": type_of_network,
                "info": center_ip_address(&grep_ssid()?,15),
                "ip": center_ip_address(&ip_address,15),
            });
        }
        0 | 4 => {
            json_obj = json!({
                "mode": network_status,
                "status": type_of_network,
                "info": "Connecting....",
                "ip": "Waiting for IP",
            });
        }
        2 => {
            json_obj = json!({
                "mode": network_status,
                "status": type_of_network,
                "info": center_ip_address("ubuntu",15),
                "ip": center_ip_address(&ip_address,15),
            });
        }
        _ => (),
    }

    let json_str = serde_json::to_string(&json_obj)?;
    Ok(json_str)
}

fn main() {
    // Initialize node
    rosrust::init("network_status");

    // Create publisher
    let chatter_pub = rosrust::publish("network_status", 10).unwrap();

    let mut count = 0;

    // Create object that maintains 10Hz between sleep requests
    let rate = rosrust::rate(1.0);

    // Breaks when a shutdown signal is sent
    let json_msg = get_json_str().unwrap();
    while rosrust::is_ok() {
        // Create string message
        let mut msg = rosrust_msg::std_msgs::String::default();

        msg.data = format!("{}", json_msg);
        println!("{}",json_msg);

        // Send string message to topic via publisher
        chatter_pub.send(msg).unwrap();

        if count > 10 {
            let json_msg = get_json_str().unwrap();
            count = 0;
        }

        // Sleep to maintain 10Hz rate
        rate.sleep();

        count += 1;
    }
}
