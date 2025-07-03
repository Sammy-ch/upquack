use crate::ui::domains::{CheckStatus, DomainStatus, HttpCode, MonitoredDomain};
use chrono::Utc;
use reqwest::{Client, StatusCode};
use std::{
    ops::Deref,
    sync::{Arc, Mutex},
    time,
};
use tokio::time::sleep;

type DomainCallbackType =
    dyn Fn(&MonitoredDomain, &[CheckStatus]) -> Result<(), std::io::Error> + Send + Sync + 'static;

pub async fn start_monitoring_task(
    domains: Arc<Mutex<Vec<MonitoredDomain>>>,
    update_domains_callback: Arc<DomainCallbackType>,
) {
    let client = Client::builder()
        .timeout(time::Duration::from_secs(10))
        .build()
        .expect("Failed to create client");

    let domains_to_monitor = {
        let domains_guard = domains.lock().unwrap();
        domains_guard.clone()
    };

    log::debug!(
        "Starting monitoring task for {} domains",
        domains_to_monitor.len()
    );

    for domain in domains_to_monitor {
        let client = client.clone();
        let domains_arc_clone = Arc::clone(&domains);
        let update_domains_callback_clone = Arc::clone(&update_domains_callback);

        tokio::spawn(async move {
            let domain_id = domain.id;
            let interval = time::Duration::from_secs(domain.interval_seconds);
            log::debug!(
                "Monitoring task started for URL: {} (ID: {}) with interval: {:?}",
                domain.url,
                domain_id,
                interval
            );

            loop {
                let start_time = Utc::now();
                let head_req_result = domain_head_request(&client, &domain.url).await;
                let end_time = Utc::now();
                let response_time = (end_time - start_time).num_milliseconds() as u64;

                let head_status = match head_req_result {
                    Ok(status_code) => {
                        let http_code = HttpCode::from_status_code(status_code);
                        let domain_status = if status_code.is_success() {
                            DomainStatus::Up
                        } else {
                            DomainStatus::Down
                        };
                        CheckStatus {
                            timestamp: end_time,
                            status: domain_status,
                            http_code: Some(http_code),
                            error_message: None,
                            response_time_ms: Some(response_time),
                        }
                    }
                    Err(e) => {
                        let err_msg = e.to_string();
                        log::error!("Error checking {}: {}", domain.url, err_msg);
                        CheckStatus {
                            timestamp: end_time,
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

                let mut domains_clone = {
                    let domain_guard = domains_arc_clone.lock().unwrap();
                    domain_guard.clone()
                };

                if let Some(d) = domains_clone.iter_mut().find(|d| d.id == domain_id) {
                    d.check_history.push(head_status);

                    if d.check_history.len() > 100 {
                        d.check_history.drain(0..d.check_history.len() - 100); // Only keep the last 100
                    }

                    let update_callback_deref = update_domains_callback_clone.deref();
                    if let Err(e) = update_callback_deref(d, &d.check_history) {
                        log::error!("Failed to save domain {} after check: {}", d.url, e);
                    }
                }

                sleep(interval).await;
            }
        });
    }
}

async fn domain_head_request(client: &Client, url: &str) -> Result<StatusCode, reqwest::Error> {
    let res = client.head(url).send().await?;
    Ok(res.status())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::domains::CheckStatus;
    use std::{
        fs, io,
        path::Path,
        sync::{Arc, Mutex},
        time::Duration,
    };
    use tokio::time::sleep;
    use uuid::Uuid;

    fn test_update_domains_callback(
        domain: &MonitoredDomain,
        check_history: &[CheckStatus],
        file_path: &Path,
    ) -> io::Result<()> {
        let mut temp_domain = domain.clone();
        temp_domain.check_history = check_history.to_vec();

        let domain_data = serde_json::to_string_pretty(&temp_domain)?;
        let specific_domain_file = file_path.join(format!("domain_{}.json", domain.id));

        fs::write(specific_domain_file, domain_data)?;
        Ok(())
    }

    #[tokio::test]
    async fn monitoring_task_processes_domains() {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let temp_dir_path = temp_dir.path().to_path_buf();

        let test_domains = vec![MonitoredDomain {
            id: Uuid::new_v4(),
            url: "http://google.com".to_string(),
            interval_seconds: 1,
            check_history: Vec::new(),
        }];

        let test_domains_arc = Arc::new(Mutex::new(test_domains.clone()));

        //callback for updating domains
        let update_callback_path = temp_dir_path.clone();
        let update_domains_closure = Arc::new(
            move |domain: &MonitoredDomain, check_history: &[CheckStatus]| {
                test_update_domains_callback(domain, check_history, &update_callback_path)
            },
        );

        // Start the monitoring task
        start_monitoring_task(test_domains_arc.clone(), update_domains_closure).await;

        sleep(Duration::from_secs(60)).await;

        // Verify that check history has been updated and saved
        let domains_guard = test_domains_arc.lock().unwrap();
        for domain in domains_guard.iter() {
            assert!(
                !domain.check_history.is_empty(),
                "Check history should not be empty for domain {}",
                domain.id
            );

            // Verify the file was created and contains data
            let expected_file = temp_dir_path.join(format!("domain_{}.json", domain.id));
            assert!(
                expected_file.exists(),
                "File for domain {} should exist",
                domain.id
            );
            let file_content = fs::read_to_string(&expected_file).expect("Failed to read file");
            let loaded_domain: MonitoredDomain =
                serde_json::from_str(&file_content).expect("Failed to deserialize domain");
            assert_eq!(loaded_domain.id, domain.id);
            assert!(
                !loaded_domain.check_history.is_empty(),
                "Loaded domain history should not be empty"
            );
        }
    }
}
