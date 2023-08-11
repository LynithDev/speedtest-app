use async_trait::async_trait;

#[derive(Debug, Clone)]
pub struct TestInfo {
    pub isp: String,
    pub external_ip: String,
    pub host: String,
    pub name: String,
    pub location: String,
    pub country: String,
}

#[derive(Debug, Clone)]
pub struct TestDownloadInfo {
    pub bandwidth: i32,
    pub elapsed: i32,
    pub progress: f32,
}

#[derive(Debug, Clone)]
pub struct TestUploadInfo {
    pub bandwidth: i32,
    pub elapsed: i32,
    pub progress: f32,
}

#[derive(Debug, Clone)]
pub struct TestResultInfo {
    pub download_bandwidth: i32,
    pub upload_bandwidth: i32,
    pub result_url: Option<String>,
}

#[async_trait]
pub trait Provider {
    fn new() -> Self;

    fn get_name(&self) -> &str;

    fn get_website(&self) -> &str;

    async fn setup(&self) -> Result<(), Box<dyn std::error::Error>>;

    fn stop_test(&mut self) -> Result<(), Box<dyn std::error::Error>>;

    fn start_test(
        &mut self, 
        info: impl Fn(TestInfo), 
        download: impl Fn(TestDownloadInfo),
        upload: impl Fn(TestUploadInfo),
        result: impl Fn(TestResultInfo)
    );
}

pub async fn start_speedtest(
    provider: &mut impl Provider,
    infofn: impl Fn(TestInfo), 
    downloadfn: impl Fn(TestDownloadInfo),
    uploadfn: impl Fn(TestUploadInfo),
    resultfn: impl Fn(TestResultInfo),
) {
    provider.setup().await.unwrap();
    provider.start_test(infofn, downloadfn, uploadfn, resultfn);
}
