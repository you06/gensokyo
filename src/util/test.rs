use std::future::Future;
use tokio::runtime::Runtime;

pub fn run_in_tokio<F>(f: F)
where
    F: Future + Send + 'static,
{
    let rt = Runtime::new().unwrap();
    rt.block_on(async move {
        f.await;
    });
    rt.shutdown_background();
}
