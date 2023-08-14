#[macro_export]
macro_rules! request {
    (method: $method:ident, client: $client:tt, url: $url:expr, $($tts:tt)* ) => {{
        use reqwest::header::CONTENT_TYPE;
        let req = $client.$method($url);
        let req = req.header(CONTENT_TYPE, "application/json");

        crate::request!(@config req, $($tts)*)
    }};

    //////////////////// config req ///
    (@config $req:expr $(,)?) => {{
        crate::request!(@do_request, $req)
    }};

    (@config $req:expr, header: {$($h_name:tt: $h_value:expr),* $(,)?} $($tts:tt)*) => {{
        let req = $req;
        $(let req = req.header($h_name, $h_value);)+
        crate::request!(@config req $($tts)*)
    }};

    (@config $req:expr, query: {$($key:literal: $value:expr),* $(,)?} $($tts:tt)*) => {{
        let req = $req;
        let q = ::serde_json::json!({
            $($key: $value),*
        });
        let req = req.query(&q);
        crate::request!(@config req $($tts)*)
    }};

    (@config $req:expr, query: $body:expr $(,)?) => {{
        let req = $req.query($body);
        crate::request!(@config req)
    }};

    (@config $req:expr, query: $body:expr, $($tts:tt)+) => {{
        let req = $req.query($body);
        crate::request!(@config req, $($tts)+)
    }};

    (@config $req:expr, body: {$($key:literal: $value:expr),* $(,)?} $($tts:tt)*) => {{
        let req = $req;
        let q = ::serde_json::json!({
            $($key: $value),*
        });
        let req = req.query(&q);
        crate::request!(@config req $($tts)*)
    }};

    (@config $req:expr, body: $body:expr $(,)?) => {{
        let req = $req.body($body);
        crate::request!(@do_post, req)
    }};

    (@config $req:expr, body: $body:expr, $($tts:tt)+) => {{
        let req = $req.body($body);
        crate::request!(@config req, $($tts)+)
    }};

    (@config $req:expr, timeout: $timeout:literal ms $($tts:tt)*) => {{
        let req = $req.timeout(::std::time::Duration::from_millis($timeout));
        crate::request!(@config req $($tts)*)
    }};

    (@config $req:expr, timeout: $timeout:literal s $($tts:tt)*) => {{
        let req = $req.timeout(::std::time::Duration::from_secs_f64($timeout));
        crate::request!(@config req $($tts)*)
    }};


    (@config $req:expr, ret: $resp_type:ident $(,)?) => {{
        crate::request!(@do_request, $req, $resp_type)
    }};

    ///////////// response ////
    (@do_request, $req:expr $(,$body_type:ident)?) => {{
        use ::anyhow::Context;
        let resp = match $req.send().await {
            Ok(resp) => resp,
            Err(err) => {
                tracing::error!(?err, line = line!(), "[HTTP] failed to send request at: {}:{}", file!(), line!());
                return Err(err).context("[HTTP] send request failed").map_err(Into::into);
            }
        };

        if !resp.status().is_success() {
            tracing::error!(code=%resp.status(), ?resp, "[HTTP] http status error at: {}:{}", file!(), line!());
            return Err(anyhow::anyhow!("http request: code = {}", resp.status()).into())
        }
        $crate::extract_response_body!(GET, resp, $($body_type)?)
    }};
}

#[macro_export]
macro_rules! match_requst {
    (method: $method:tt, client: $client:expr, url: $url:expr $(,)?) => {{
        crate::request!(method: get, client: $client, url: $url,)
    }};

    (method: $method:tt, client: $client:expr, url: $url:expr, $($tts:tt)+) => {{
        crate::request!(method: get, client: $client, url: $url, $($tts)+)
    }};

    (method: $method:tt, $client:expr, $url:expr $(,)?) => {{
        crate::match_requst!(method: $method, client: $client, url: $url)
    }};

    (method: $method:tt, $client:expr, $url:expr, $($tts:tt)+) => {{
        crate::match_requst!(method: $method, client: $client, url: $url, $($tts)+)
    }};

    (method: $method:tt, $url:expr $(,)?) => {{
        let client_ = ::reqwest::Client::new();
        crate::match_requst!(method: $method, client: client_, url: $url)
    }};

    (method: $method:tt, $url:expr, $($tts:tt)+) => {{
        let client_ = ::reqwest::Client::new();
        crate::match_requst!(method: $method, client: client_, url: $url, $($tts)+)
    }};
}

#[macro_export]
macro_rules! get {
    ($($tts:tt)+) => {{
        crate::match_requst!(method: get, $($tts)+)
    }};
}

#[macro_export]
macro_rules! post {
    ($($tts:tt)+) => {{
       crate::match_requst!(method: post, $($tts)+)
    }};
}

#[macro_export]
macro_rules! extract_response_body {
    ($method:tt, $resp:expr $(,)?) => {
        $crate::extract_response_body!($method, $resp, json)
    };

    ($method:tt, $resp:expr, text $(,)?) => {{
        match $resp.text().await {
            Ok(r) => r,
            Err(err) => {
                use anyhow::Context;
                let url = err.url();
                let is_redirect = err.is_redirect();
                ::tracing::error!(?err);
                ::tracing::error!(
                    ?url,
                    is_redirect,
                    concat!(
                        "[",
                        stringify!($method),
                        "]",
                        " failed to read response body as TEXT at: {}:{}"
                    ),
                    file!(),
                    line!()
                );
                return Err(err).context("http body cannot be read as text").map_err(Into::into);
            }
        }
    }};

    ($method:tt, $resp:expr, json $(,)?) => {{
        let url = $resp.url().clone();
        let text = $crate::extract_response_body!($method, $resp, text);
        match ::serde_json::from_str(&text) {
            Ok(r) => r,
            Err(err) => {
                use anyhow::Context;
                ::tracing::error!(%text, "origin response");
                ::tracing::error!(%url, concat!(
                        "[",
                        stringify!($method),
                        "]",
                        "failed to deserialize response body as json. At: {}:{}"
                    ),
                    file!(),
                    line!()
                );
                return Err(err).context("http body cannot be read as json").map_err(Into::into);
            }
        }
    }};

    ($method:tt, $resp:expr, $body_type:tt $(,)?) => {
        match $resp.$body_type().await {
            Ok(r) => r,
            Err(err) => {
                ::tracing::error!(
                    ?err,
                    concat!(
                        "[",
                        stringify!($method),
                        "]",
                        " failed to read response body at: {}:{}"
                    ),
                    file!(),
                    line!()
                );
                return Err(err.into());
            }
        }
    };
}

#[cfg(test)]
mod test {
    use anyhow::Result;
    use tracing_test::traced_test;

    #[tokio::test]
    #[traced_test]
    async fn t_get() -> Result<()> {
        let url = "https://httpbin.org/ip";
        let resp: serde_json::Value = get!(url, timeout: 3.5 s, ret: json);
        println!("{}", resp);

        let url = "https://httpbin.org/headers";
        let resp: serde_json::Value = get!(url, header: {"aa": "bb", "cc":"dd",},  ret: json);
        println!("{}", resp);

        let client = reqwest::Client::new();
        let url = "https://httpbin.org/get";
        let resp: serde_json::Value = get! {
            client: client,
            url: url,
            query: {"key": "value"},
            header: {
                "aa": "bb",
                "cc":"dd",
            },
            ret: json
        };
        println!("{}", resp);

        Ok(())
    }

    #[tokio::test]
    #[traced_test]
    async fn t_post() -> Result<()> {
        let client = reqwest::Client::new();
        let url = "https://httpbin.org/post";
        let resp: serde_json::Value = post! {
            client: client,
            url: url,
            header: {
                "aa": "bb",
                "cc":"dd",
            },
            timeout: 3 ms,
            body: r#"{"key": "value"}"#,
            ret: json
        };
        println!("{}", resp);

        Ok(())
    }
}
