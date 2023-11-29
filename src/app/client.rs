use std::path::PathBuf;

use messages::{message_client::MessageClient, FileRequest, FileSend, MessageRequest, MessageSync};

use self::messages::FileResponse;
pub mod messages {
    tonic::include_proto!("messages");
}

//main is for sending
pub async fn send_msg(
    username: String,
    msg: String,
    passw: String,
    ip: String,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut client: MessageClient<tonic::transport::Channel> =
        MessageClient::connect(format!("http://{}", ip)).await?;

    let request = tonic::Request::new(MessageRequest {
        message: msg.trim().to_string(),
        author: username,
        password: passw,
    });

    let response = client.send_message(request).await?.into_inner().clone();

    let message = response.message;

    Ok(message)
}

pub async fn sync_msg(passw: String, ip: String) -> Result<String, Box<dyn std::error::Error>> {
    let mut client: MessageClient<tonic::transport::Channel> =
        MessageClient::connect(format!("http://{}", ip)).await?;

    let request = tonic::Request::new(MessageSync { password: passw });

    let response = client.sync_message(request).await?.into_inner().clone();

    let message = response.message;

    Ok(message)
}

pub async fn send_file(
    passw: String,
    ip: String,
    file_bytes: Vec<u8>,
    file: PathBuf,
    author: String,
) -> Result<i32, Box<dyn std::error::Error>> {
    let mut client: MessageClient<tonic::transport::Channel> =
        MessageClient::connect(format!("http://{}", ip)).await?;
    let f_name = file.file_name().unwrap().to_string_lossy().to_string();

    dbg!(f_name.clone());

    let request = tonic::Request::new(FileSend {
        file: file_bytes,
        name: f_name,
        passw,
        author,
    });

    let response = client.recive_file(request).await?.into_inner().clone();

    Ok(response.error)
}

pub async fn request_file(
    index: i32,
    ip: String,
) -> Result<FileResponse, Box<dyn std::error::Error>> {
    let mut client: MessageClient<tonic::transport::Channel> =
        MessageClient::connect(format!("http://{}", ip)).await?;

    let request = FileRequest { index };

    let response = client.serve_file(request).await?.into_inner().clone();

    Ok(response)
}
