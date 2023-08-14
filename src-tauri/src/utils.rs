pub mod http;

#[macro_export]
macro_rules! log_if_err {
    ($run:expr) => {
        crate::log_if_err!($run, stringify!($run))
    };

    ($run:expr, $msg:expr $(,)?) => {
        if let Err(err) = $run {
            ::tracing::error!(?err, concat!("FAILED: ", $msg))
        }
    };
}
