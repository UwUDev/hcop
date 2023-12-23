use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

use reqwest::header::HeaderMap;
use serde_json::json;

#[derive(Clone)]
struct Hcop {
    api_key: String,
    clent: reqwest::blocking::Client,
}

struct HCaptchaTask {
    hcop: Hcop,
    task_id: String,
}

struct UserData {
    balance: f32,
    max_threads: u32,
    rank: String,
    running_threads: u32,
    username: String,
}

struct TaskResult {
    captcha_key: Option<String>,
    refunded: bool,
    status: TaskStatus
}

#[derive(PartialEq)]
enum TaskStatus {
    Processing,
    Completed,
    Error,
}

impl Hcop {
    fn new(api_key: String) -> Hcop {
        Hcop {
            api_key,
            clent: reqwest::blocking::Client::new(),
        }
    }

    fn create_hcaptcha_task(&self, site_key: String, site_url: String, proxy: String, rqdata: Option<String>) -> Result<HCaptchaTask, HCopError> {
        let mut payload = json!({
            "task_type": "hcaptchaEnterprise",
            "api_key": self.api_key,
            "data": {
                "sitekey": site_key,
                "url": site_url,
                "proxy": proxy,
            }
        });

        if rqdata.is_some() {
            payload["data"]["rqdata"] = json!(rqdata.unwrap());
        }

        let mut headers = HeaderMap::new();
        headers.insert("Content-Type", "application/json".parse().unwrap());
        headers.insert("User-Agent", "HCop/rust".parse().unwrap());

        match self.clent.post("https://api.hcoptcha.online/api/createTask").json(&payload).headers(headers).send() {
            Ok(response) => {
                let json = response.json::<serde_json::Value>().unwrap();

                let error = json["error"].as_bool().unwrap();
                if error {
                    return Err(HCopError {
                        message: json["message"].as_str().unwrap().to_string(),
                    });
                }

                let task_id = json["task_id"].as_str().unwrap().to_string();
                Ok(HCaptchaTask {
                    hcop: self.clone(),
                    task_id,
                })
            }
            Err(e) => Err(HCopError {
                message: e.to_string(),
            }),
        }
    }

    fn get_user_data(&self) -> Result<UserData, HCopError> {
        let payload = json!({
            "api_key": self.api_key
        });
        let mut headers = HeaderMap::new();
        headers.insert("Content-Type", "application/json".parse().unwrap());
        headers.insert("User-Agent", "HCop/rust".parse().unwrap());

        match self.clent.post("https://api.hcoptcha.online/api/getUserData").json(&payload).headers(headers).send() {
            Ok(response) => {
                if response.status().as_u16() == 401 {
                    return Err(HCopError {
                        message: "Invalid API key".to_string(),
                    });
                }

                let json = response.json::<serde_json::Value>().unwrap();

                let error = json["error"].as_bool().unwrap();
                if error {
                    return Err(HCopError {
                        message: json["message"].as_str().unwrap().to_string(),
                    });
                }

                let balance = json["data"]["balance"].as_f64().unwrap() as f32;
                let max_threads = json["data"]["max_threads"].as_u64().unwrap() as u32;
                let rank = json["data"]["rank"].as_str().unwrap().to_string();
                let running_threads = json["data"]["running_threads"].as_u64().unwrap() as u32;
                let username = json["data"]["username"].as_str().unwrap().to_string();
                Ok(UserData {
                    balance,
                    max_threads,
                    rank,
                    running_threads,
                    username,
                })
            }
            Err(e) => Err(HCopError {
                message: e.to_string(),
            }),
        }
    }
}

impl HCaptchaTask {
    pub fn get_result(&self) -> Result<TaskResult, HCopError> {
        let payload = json!({
            "api_key": self.hcop.api_key,
            "task_id": self.task_id
        });
        let mut headers = HeaderMap::new();
        headers.insert("Content-Type", "application/json".parse().unwrap());
        headers.insert("User-Agent", "HCop/rust".parse().unwrap());

        match self.hcop.clent.post("https://api.hcoptcha.online/api/getTaskData").json(&payload).headers(headers).send() {
            Ok(response) => {
                let json = response.json::<serde_json::Value>().unwrap();

                let error = json["error"].as_bool().unwrap();
                if error {
                    // print json
                    println!("{}", json.to_string());
                    let mut message = "Unknown error".to_string();
                    if json.get("message").is_some() {
                        message = json["message"].as_str().unwrap().to_string();
                    }
                    return Err(HCopError {
                        message,
                    });
                }

                let captcha_key = json["task"]["captcha_key"].as_str().map(|s| s.to_string());
                let refunded = json["task"]["refunded"].as_bool().unwrap();
                let s = json["task"]["state"].as_str().unwrap().to_string();
                let status = match s.as_str() {
                    "processing" => TaskStatus::Processing,
                    "completed" => TaskStatus::Completed,
                    "error" => TaskStatus::Error,
                    _ => panic!("Unknown status: {}", s),
                };

                Ok(TaskResult {
                    captcha_key,
                    refunded,
                    status,
                })
            }
            Err(e) => Err(HCopError {
                message: e.to_string(),
            }),
        }
    }
}

struct HCopError {
    message: String
}

impl Error for HCopError {}

impl Debug for HCopError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut debug_struct = f.debug_struct("HCopError");
        debug_struct.field("message", &self.message);
        debug_struct.finish()
    }
}

impl Display for HCopError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn feur() {
        let hcop = Hcop::new("YOUR_API_KEY".to_string());
        match hcop.get_user_data() {
            Ok(user_data) => {
                println!("{} has {}$", user_data.username, user_data.balance);
            }
            Err(e) => {
                println!("{}", e.message);
                return;
            }
        }
        
        let task = hcop.create_hcaptcha_task(
            "a5f74b19-9e45-40e0-b45d-47ff91b7a6c2".to_string(),
            "accounts.hcaptcha.com".to_string(),
            "IP:PORT OR USER:PASS@IP:PORT".to_string(),
            None
        );

        let start = std::time::Instant::now();

        match task {
            Ok(task) => {
                println!("Task ID: {}", task.task_id);

                loop {
                    let result = task.get_result();
                    match result {
                        Ok(result) => {
                            if result.status == TaskStatus::Completed {
                                break;
                            }
                        }
                        Err(e) => {
                            println!("{}", e.message);
                            return;
                        }
                    }
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
            }
            Err(e) => {
                println!("{}", e.message);
                return;
            }
        }
        let duration = start.elapsed();
        println!("Solved in {:?}", duration);
    }
}
