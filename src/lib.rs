use spin_sdk::http::{IntoResponse, Request, Response};
use spin_sdk::http_component;

mod wit {
    wit_bindgen::generate!({
        inline: r#"
        package test:test;
        world test {
            import wasi:blobstore/blobstore@0.2.0-draft-2024-09-01;
        }
        "#,
        path: "../../github/spin/wit",
    });
}

/// A simple Spin HTTP component.
#[http_component]
fn handle_blobby_blobby_blobby(req: Request) -> anyhow::Result<impl IntoResponse> {
    let resp_str = handle_blobby_blobby_blobby_impl(req).map_err(|s| anyhow::anyhow!("{s}"))?;
    Ok(Response::builder()
        .status(200)
        .header("content-type", "text/plain")
        .body(resp_str)
        .build())
}

fn handle_blobby_blobby_blobby_impl(_req: Request) -> Result<String, String> {
    let container = wit::wasi::blobstore::blobstore::get_container(&"default".to_string())?;
    let data = wit::wasi::blobstore::types::OutgoingValue::new_outgoing_value();
    container.write_data(&"spork".to_string(), &data)?;
    let stm = data.outgoing_value_write_body().unwrap();
    stm.write("wibbly wobbly".as_bytes()).unwrap();
    wit::wasi::blobstore::types::OutgoingValue::finish(data)?;
    container.name()
}
