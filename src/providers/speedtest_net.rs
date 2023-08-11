use std::{path::Path, fs::File, process::{Command, Stdio}, io::{BufReader, BufRead}};

use async_trait::async_trait;
use flate2::read::GzDecoder;
use serde::{Deserialize, Serialize};

use crate::{provider::{Provider, TestInfo, TestDownloadInfo, TestUploadInfo, TestResultInfo}, utils::{download_file, get_app_data_dir}};


#[derive(Deserialize, Serialize)]
struct DownloadInfo {
    timestamp: String,
    download: Download
}

#[derive(Deserialize, Serialize)]
struct Download {
    bandwidth: i32,
    bytes: i32,
    elapsed: i32,
    progress: f32,
}

#[derive(Deserialize, Serialize)]
struct DownloadWithoutProgress {
    bandwidth: i32,
    bytes: i32,
    elapsed: i32,
}

#[derive(Deserialize, Serialize)]
struct UploadInfo {
    timestamp: String,
    upload: Upload
}

#[derive(Deserialize, Serialize)]
struct Upload {
    bandwidth: i32,
    bytes: i32,
    elapsed: i32,
    progress: f32,
}

#[derive(Deserialize, Serialize)]
struct UploadWithoutProgress {
    bandwidth: i32,
    bytes: i32,
    elapsed: i32,
}

#[derive(Deserialize, Serialize)]
struct TestStartInfo {
    timestamp: String,
    isp: String,
    interface: Interface,
    server: ServerInfo,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Interface {
    external_ip: String,
}

#[derive(Deserialize, Serialize)]
struct ServerInfo {
    host: String,
    name: String,
    location: String,
    country: String,
}

#[derive(Deserialize, Serialize)]
struct ResultInfo {
    timestamp: String,
    download: DownloadWithoutProgress,
    upload: UploadWithoutProgress,

    #[serde(rename = "packetLoss")]
    packet_loss: f32,

    isp: String,
    interface: Interface,
    server: ServerInfo,
    result: ResultExpanded,
}

#[derive(Deserialize, Serialize)]
struct ResultExpanded {
    id: String,
    url: String,
    persisted: bool,
}

#[derive(Clone, Copy)]
pub struct SpeedtestNetProvider {
    pid: Option<u32>,
}

#[async_trait]
impl Provider for SpeedtestNetProvider {
    fn new() -> Self {
        Self {
            pid: None,
        }
    }

    fn get_name(&self) -> &str {
        "speedtest.net"
    }

    fn get_website(&self) -> &str {
        "https://speedtest.net/"
    }

    async fn setup(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = format!("{}/speedtest", get_speedtest_cli_path());

        if Path::new(&path).exists() {
            return Ok(());
        }

        let _archive_path = format!("{}/speedtest.tgz", get_app_data_dir());
        let archive_path = Path::new(_archive_path.as_str());

        if !archive_path.exists() {
            println!("Downloading speedtest.net CLI");

            // For now, only Linux x86_64 is supported
            let arch = "x86_64";
            let download_link = format!("https://install.speedtest.net/app/cli/ookla-speedtest-1.2.0-linux-{}.tgz", arch);
            
            download_file(&download_link, archive_path.as_os_str().to_str().unwrap()).await?;
        }

        if archive_path.exists() {
            println!("Extracting speedtest.net CLI");

            let tar = GzDecoder::new(File::open(archive_path)?);
            let mut archive = tar::Archive::new(tar);
            archive.unpack(get_speedtest_cli_path())?;
        }

        Ok(())
    }

    fn stop_test(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.pid.is_some() {
            println!("Stopping test");
            let _ = Command::new("kill")
                .arg(self.pid.unwrap().to_string())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn();

            self.pid = None;
            return Ok(());
        }

        Err("No test running".into())
    }

    fn start_test(
        &mut self, 
        infofn: impl Fn(TestInfo), 
        downloadfn: impl Fn(TestDownloadInfo), 
        uploadfn: impl Fn(TestUploadInfo),
        resultfn: impl Fn(TestResultInfo)
    ) {
        if self.pid.is_some() {
            println!("Test already running");
            return;
        }

        println!("Starting download test");

        let _ = exec_cli(&mut self.pid, 
            |info| {
                infofn(TestInfo {
                    isp: info.isp,
                    external_ip: info.interface.external_ip,
                    host: info.server.host,
                    name: info.server.name,
                    location: info.server.location,
                    country: info.server.country,
                });
            }, 
            |download| {
                downloadfn(TestDownloadInfo {
                    bandwidth: download.download.bandwidth,
                    elapsed: download.download.elapsed,
                    progress: download.download.progress,
                });
            },
            |upload| {
                uploadfn(TestUploadInfo {
                    bandwidth: upload.upload.bandwidth,
                    elapsed: upload.upload.elapsed,
                    progress: upload.upload.progress,
                });
            },
            |result| {
                resultfn(TestResultInfo {
                    download_bandwidth: result.download.bandwidth,
                    upload_bandwidth: result.upload.bandwidth,
                    result_url: Some(result.result.url),
                });
            }
        );
    }
}

pub fn get_speedtest_cli_path() -> String {
    format!("{}/speedtest_net", get_app_data_dir())
}

pub fn speedtest_cli_installed() -> bool {
    Path::new(get_speedtest_cli_path().as_str()).exists()
}

fn exec_cli(
    pid: &mut Option<u32>, 
    info: impl Fn(TestStartInfo), 
    download: impl Fn(DownloadInfo), 
    upload: impl Fn(UploadInfo),
    result: impl Fn(ResultInfo),
) -> Result<(), Box<dyn std::error::Error>> {
    let mut command = Command::new(format!("{}/speedtest", get_speedtest_cli_path()))
        .arg("--accept-license")
        .arg("--accept-gdpr")
        .arg("--format=json")
        .arg("--progress=yes")
        .stdout(Stdio::piped())
        .spawn()?;

    *pid = Some(command.id());

    let stdout = command.stdout.take().unwrap();
    let lines = BufReader::new(stdout).lines();
    for line in lines {
        let line = line.unwrap();
        
        if !line.starts_with("{\"type") {
            continue;
        }

        match true {
            _ if line.starts_with("{\"type\":\"testStart\"") => info(serde_json::from_str::<TestStartInfo>(&line)?),
            _ if line.starts_with("{\"type\":\"download\"") => download(serde_json::from_str::<DownloadInfo>(&line)?),
            _ if line.starts_with("{\"type\":\"upload\"") => upload(serde_json::from_str::<UploadInfo>(&line)?),
            _ if line.starts_with("{\"type\":\"result\"") => result(serde_json::from_str::<ResultInfo>(&line)?),
            _ => {}
        };
    }

    Ok(())
}
