use crate::ui::domains::{CheckStatus, DomainStatus, HttpCode, MonitoredDomain};
use chrono::Utc;
use log::{error, info};
use reqwest::{Client, StatusCode};
use std::{
    sync::{Arc, Mutex},
    time::{self},
};
use tokio::time::sleep;

pub fn start_monitoring_task(
    domains: Arc<Mutex<Vec<MonitoredDomain>>>,
    update_domains: impl Fn(&[MonitoredDomain]) -> Result<(), std::io::Error> + Send + Sync + 'static,
) {
    let client = Client::builder()
        .timeout(time::Duration::from_secs(10))
        .build()
        .expect("Failed to create client");

    let domains_guard = domains.lock().unwrap();
    let domains_clone = domains_guard.clone();

    drop(domains_guard);

    info!("Starting monitoring task");

    for domain in domains_clone {
        let client = client.clone();
        let domains_arc_clone = Arc::clone(&domains);

        tokio::spawn(async move {
            let domain_id = domain.id;
            let interval = time::Duration::from_secs(domain.interval_seconds);
            info!(
                "Monitoring task started for URL: {} (ID: {}) with interval: {:?}",
                domain.url, domain_id, interval
            );

            loop {
                let start_time = Utc::now();
                let head_req = domain_head_request(&client, &domain.url).await;
                let response_time = (start_time - Utc::now()).num_milliseconds() as u64;

                let check_status = match head_req {
                    Ok(statuscode) => {
                        let http_code = HttpCode::from_status_code(statuscode);
                        let domain_status = if statuscode.is_success() {
                            DomainStatus::UP
                        } else {
                            DomainStatus::DOWN
                        };
                        CheckStatus {
                            timestamp: Utc::now(),
                            status: domain_status,
                            http_code: Some(http_code),
                            error_message: None,
                            response_time_ms: Some(response_time),
                        }
                    }

                    Err(e) => {
                        let err_msg = e.to_string();
                        error!("Error checking {}: {}", domain.url, err_msg);
                        CheckStatus {
                            timestamp: Utc::now(),
                            status: DomainStatus::Error(err_msg.clone()),
                            http_code: if e.is_timeout() {
                                Some(HttpCode::Timeout)
                            } else {
                                Some(HttpCode::NetworkError)
                            },
                            response_time_ms: None,
                            error_message: Some(err_msg),
                        }
                    }
                };

                {
                    let mut domains_guard = domains_arc_clone.lock().unwrap().clone();
                    if let Some(d) = domains_guard.iter_mut().find(|d| d.id == domain_id) {
                        d.check_history.push(check_status);

                        if d.check_history.len() > 100 {
                            d.check_history.drain(0..1);
                        }

                        // if let Err(e) = update_domains(&domains_guard) {
                        //     error!("Failed to save domains after check: {}", e);
                        // }
                    }
                }
                sleep(interval).await;
            }
        });
    }
}

async fn domain_head_request(client: &Client, url: &str) -> Result<StatusCode, reqwest::Error> {
    let res = client.head(url).send().await.unwrap();
    let status = res.status();
    Ok(status)
}

#[cfg(test)]
mod test {
    use std::time::Duration;

    use super::*;

    #[tokio::test]
    async fn is_success() {
        let url = "https://google.com";
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("Failed to create client");

        let status = domain_head_request(&client, url).await.unwrap();

        assert!(StatusCode::is_success(&status))
    }
}
