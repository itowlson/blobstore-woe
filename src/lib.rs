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
        with: {
            "wasi:io/streams": wasi::io::streams,
        },
        generate_all
    });
}

/// A simple Spin HTTP component.
#[http_component]
fn handle_blobby_blobby_blobby(req: Request) -> anyhow::Result<impl IntoResponse> {
    let resp_str = handle_blobby_blobby_blobby_impl(req).map_err(|s| anyhow::anyhow!("{s}"))?;
    Ok(Response::builder()
        .status(200)
        .header("content-type", "text/plain")
        .body(format!("{resp_str}\n"))
        .build())
}

fn handle_blobby_blobby_blobby_impl(_req: Request) -> Result<String, String> {
    use std::io::{Read, Write};

    const WHICH_CONTAINER: &str = "aws";
    let container = wit::wasi::blobstore::blobstore::get_container(&WHICH_CONTAINER.to_string())?;

    // LIST ////////////////////////////////////////////////

    let hobs = container.list_objects()?;
    let mut all_names = Vec::with_capacity(100);
    loop {
        let (mut names, at_end) = hobs.read_stream_object_names(2)?;
        all_names.append(&mut names);
        if at_end {
            break;
        }
    }

    println!("{}", all_names.join("\n"));
    // Ok(all_names.join("\n"))

    const WHICH_BLOB: &str = "my-biggerer-blob";

    // WRITE ///////////////////////////////////////////////////////////////////

    let data = wit::wasi::blobstore::types::OutgoingValue::new_outgoing_value();
    container.write_data(&WHICH_BLOB.to_string(), &data)?;

    let mut stm = data.outgoing_value_write_body().unwrap();

    // The raw way
    // for index in 0..1000 {
    //     loop {
    //         let pb = stm.check_write().unwrap();
    //         if pb > 30 {
    //             break;
    //         }
    //         println!("waiting for flush to complete");
    //         std::thread::sleep(std::time::Duration::from_millis(50));
    //     }

    //     let text = format!("wibblier and wobblier {index}\n");
    //     // println!("writing {index}");
    //     stm.write(text.as_bytes()).unwrap();
    //     if index > 0 && index % 200 == 0 {
    //         stm.flush().unwrap();
    //     }
    // }

    // The easy way
    for index in 0..1000 {
        let text = format!("even wibblier and yet moar wobblier {index}\n");
        stm.write_all(text.as_bytes()).unwrap();
    }

    stm.flush().unwrap();

    wit::wasi::blobstore::types::OutgoingValue::finish(data)?;

    // READ ///////////////////////////////////////////////////////////////////

    let iv = container.get_data(&WHICH_BLOB.to_owned(), 0, u64::MAX)?;

    // Sync
    // let vals = wit::wasi::blobstore::types::IncomingValue::incoming_value_consume_sync(iv)?;
    // Ok(String::from_utf8_lossy(&vals).to_string())

    // Async
    let stm = wit::wasi::blobstore::types::IncomingValue::incoming_value_consume_async(iv);

    let mut stm = stm?;

    let mut vals = Vec::with_capacity(1000);

    // The raw way
    // loop {
    //     match stm.blocking_read(100) {
    //         Ok(mut v) => vals.append(&mut v),
    //         Err(wit::wasi::io0_2_0::streams::StreamError::Closed) => break,
    //         Err(wit::wasi::io0_2_0::streams::StreamError::LastOperationFailed(e)) => Err(e.to_debug_string())?,
    //     };
    // }

    // The easy way
    let read_count = stm.read_to_end(&mut vals).unwrap();

    Ok(String::from_utf8_lossy(&vals[..read_count]).to_string())

}

////////////////////// from wasi crate /////////////////////////////
// use std::io;
// use std::num::NonZeroU64;
// use wit::wasi::io0_2_0::streams::StreamError;

// impl io::Read for wit::wasi::io0_2_0::streams::InputStream {
//     fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
//         let n = buf
//             .len()
//             .try_into()
//             .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
//         match self.blocking_read(n) {
//             Ok(chunk) => {
//                 let n = chunk.len();
//                 if n > buf.len() {
//                     return Err(io::Error::new(
//                         io::ErrorKind::Other,
//                         "more bytes read than requested",
//                     ));
//                 }
//                 buf[..n].copy_from_slice(&chunk);
//                 Ok(n)
//             }
//             Err(StreamError::Closed) => Ok(0),
//             Err(StreamError::LastOperationFailed(e)) => {
//                 Err(io::Error::new(io::ErrorKind::Other, e.to_debug_string()))
//             }
//         }
//     }
// }

// impl io::Write for wit::wasi::io0_2_0::streams::OutputStream {
//     fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
//         let n = loop {
//             match self.check_write().map(NonZeroU64::new) {
//                 Ok(Some(n)) => {
//                     break n;
//                 }
//                 Ok(None) => {
//                     self.subscribe().block();
//                 }
//                 Err(StreamError::Closed) => return Ok(0),
//                 Err(StreamError::LastOperationFailed(e)) => {
//                     return Err(io::Error::new(io::ErrorKind::Other, e.to_debug_string()))
//                 }
//             };
//         };
//         let n = n
//             .get()
//             .try_into()
//             .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
//         let n = buf.len().min(n);
//         wit::wasi::io0_2_0::streams::OutputStream::write(self, &buf[..n]).map_err(|e| match e {
//             StreamError::Closed => io::ErrorKind::UnexpectedEof.into(),
//             StreamError::LastOperationFailed(e) => {
//                 io::Error::new(io::ErrorKind::Other, e.to_debug_string())
//             }
//         })?;
//         Ok(n)
//     }

//     fn flush(&mut self) -> io::Result<()> {
//         self.blocking_flush()
//             .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
//     }
// }
