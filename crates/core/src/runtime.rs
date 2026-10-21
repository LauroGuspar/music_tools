use std::future::Future;
use std::sync::LazyLock;
use tokio::runtime::{Handle, Runtime};
use tokio::task::JoinHandle;

static RUNTIME: LazyLock<Runtime> = LazyLock::new(|| {
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .thread_name("music-tools-io")
        .enable_all()
        .build()
        .expect("Failed to initialize multi-thread Tokio runtime")
});

pub fn tokio_runtime() -> &'static Runtime {
    &RUNTIME
}

pub fn tokio_handle() -> Handle {
    RUNTIME.handle().clone()
}

pub fn spawn<F>(future: F) -> JoinHandle<F::Output>
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    RUNTIME.spawn(future)
}

pub fn spawn_blocking<F, R>(func: F) -> JoinHandle<R>
where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
{
    RUNTIME.spawn_blocking(func)
}
