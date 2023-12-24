# HCop
A rust wrapper for the [HCoptcha](https://hcoptcha.online) API.

## Usage
```toml
[dependencies]
hcop = "0.1.0"
```

```rust
use hcop::HCop;

fn main() {
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
```

## Contributing
Pull requests are welcome. For major changes, please open an issue first to discuss what you would like to change.
