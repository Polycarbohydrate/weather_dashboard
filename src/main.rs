use std::io;
use tokio;
use serde_json::Value;
fn get_input() -> String {
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");
    input.trim().to_string()
}

fn coordinates() -> f64 {
    loop {
        let input = get_input();
        let is_coordinate_num: f64 = match input.trim().parse() {
            Ok(num) => num,
            Err(_) => {
                println!("===========================================");
                println!("Invalid input. Please enter a valid number.");
                println!("===========================================");
                continue;
            }
        };
        if is_coordinate_num.abs().to_string().len() < 7 || is_coordinate_num.abs().to_string().len() > 8 {
            println!("==========================================================================");
            println!("Invalid coordinate format. Please enter coordinate with at least 6 digits.");
            println!("6-7 digits for longitude. Negative sign not included. (e.g., -12.3456)");
            println!("==========================================================================");
            continue;
        }
        break is_coordinate_num;
    }
}

fn main() {
    println!("===============================================================");
    println!("This script will get the weather forecast for a given location.");
    println!("Enter the latitude of the location [(-)XX.XXXX]:");
    println!("================================================");
    let fmt_latitude = coordinates();
    if fmt_latitude <= -90.0 || fmt_latitude >= 90.0 {
        println!("Invalid latitude. Please enter a value equal to or between -90 and 90.");
        return;
    }
    println!("=================================================");
    println!("Enter the longitude of the location [(-)XXX.XXXX]:");
    println!("=================================================");
    let fmt_longitude = coordinates();
    if fmt_longitude <= -180.0 || fmt_longitude >= 180.0 {
        println!("Invalid longitude. Please enter a value equal to or between -180 and 180.");
        return;
    }
    again(fmt_latitude, fmt_longitude);
}

fn again(fmt_latitude: f64, fmt_longitude: f64 ) {
    let mut coordinates: Vec<(f64, f64)> = Vec::new();
    coordinates.push((fmt_latitude, fmt_longitude));
    println!("===========================================================");
    println!("Do you want to get weather data for another location? (y/n)");
    println!("===========================================================");
    loop {
        let input = get_input();
        if input.to_lowercase() == "y" {
            main();
        } else if input.to_lowercase() == "n" {
            println!("=====================================");
            println!("Getting weather data for coordinates.");
            println!("=====================================");
            for (latitude, longitude) in coordinates {
                println!("Latitude: {}, Longitude: {}", latitude, longitude);
                tokio::runtime::Runtime::new().unwrap().block_on(get_weather_data(latitude, longitude));
            }
            break
        } else {
            println!("=======================================");
            println!("Invalid input. Please enter 'y' or 'n'.");
            println!("=======================================");
            continue
        }
    }
}

async fn get_weather_data(latitude: f64, longitude: f64) {
    let url = format!("https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&hourly=temperature_2m,precipitation_probability,precipitation,visibility&forecast_days=1", latitude, longitude);

    match reqwest::get(&url).await {
        Ok(response) => {
            match response.text().await {
                Ok(body) => {
                    let v: Value = match serde_json::from_str(&body) {
                        Ok(data) => data,
                        Err(err) => {
                            eprintln!("Failed to parse JSON: {}", err);
                            return;
                        }
                    };
                    let hourly = &v["hourly"];
                    let empty_vec: Vec<Value> = Vec::new();
                    let time = hourly["time"].as_array().unwrap_or(&empty_vec);
                    let temp = hourly["temperature_2m"].as_array().unwrap_or(&empty_vec);
                    let precip_prob = hourly["precipitation_probability"].as_array().unwrap_or(&empty_vec);
                    let precip = hourly["precipitation"].as_array().unwrap_or(&empty_vec);
                    let visibility = hourly["visibility"].as_array().unwrap_or(&empty_vec);
                    println!("\n=============== Weather Forecast ===============");
                    println!("Location: ({}, {})", latitude, longitude);
                    println!("==============================================");
                    println!("{:<8} {:<10} {:<10} {:<10} {:<10}",
                             "Hour", "Temp(°C)", "Precip(%)", "Rain(mm)", "Vis(m)");
                    println!("----------------------------------------------");
                    for i in 0..time.len() {
                        if let Some(time_str) = time[i].as_str() {
                            let hour = time_str.split('T').nth(1).unwrap_or("N/A");
                            println!("{:<8} {:<10.1} {:<10.0} {:<10.1} {:<10.0}",
                                     hour,
                                     temp[i].as_f64().unwrap_or(0.0),
                                     precip_prob[i].as_f64().unwrap_or(0.0),
                                     precip[i].as_f64().unwrap_or(0.0),
                                     visibility[i].as_f64().unwrap_or(0.0));
                        }
                    }
                    let temp_values: Vec<f64> = temp.iter()
                        .filter_map(|v| v.as_f64())
                        .collect();
                    let precip_prob_values: Vec<f64> = precip_prob.iter()
                        .filter_map(|v| v.as_f64())
                        .collect();
                    let precip_values: Vec<f64> = precip.iter()
                        .filter_map(|v| v.as_f64())
                        .collect();
                    if !temp_values.is_empty() && !precip_prob_values.is_empty() && !precip_values.is_empty() {
                        let avg_temp = temp_values.iter().sum::<f64>() / temp_values.len() as f64;
                        let max_precip_prob = precip_prob_values.iter().fold(0.0, |max, &val| f64::max(max, val));
                        let total_rain = precip_values.iter().sum::<f64>();
                        println!("==============================================");
                        println!("SUMMARY:");
                        println!("Avg Temperature: {:.1}°C", avg_temp);
                        println!("Max Precip Chance: {:.0}%", max_precip_prob);
                        println!("Total Precipitation: {:.1} mm", total_rain);
                        println!("==============================================\n");
                    }
                },
                Err(err) => eprintln!("Failed to read response body: {}", err),
            }
        },
        Err(err) => eprintln!("Request failed: {}", err),
    }
}