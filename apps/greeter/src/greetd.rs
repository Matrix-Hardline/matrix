//! The greetd bridge: the IPC protocol of the login daemon.
//!
//! greetd speaks JSON over a Unix socket ($GREETD_SOCK), every
//! message with a u32 length prefix (native endian). We run the
//! whole login as ONE blocking flow (runs in the task pool):
//! create session → answer prompts → set the target command.
//! When the greeter ends afterwards, greetd switches to the user session.

use serde::Deserialize;
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum Answer {
    Success,
    Error {
        #[serde(rename = "error_type")]
        _error_type: String,
        description: String,
    },
    AuthMessage {
        auth_message_type: String,
        #[serde(rename = "auth_message")]
        _auth_message: String,
    },
}

fn send(strom: &mut UnixStream, message: &serde_json::Value) -> Result<Answer, String> {
    let data = message.to_string().into_bytes();
    strom
        .write_all(&(data.len() as u32).to_ne_bytes())
        .and_then(|_| strom.write_all(&data))
        .map_err(|e| format!("greetd nicht erreichbar: {e}"))?;
    let mut length = [0u8; 4];
    strom
        .read_exact(&mut length)
        .map_err(|e| format!("greetd antwortet nicht: {e}"))?;
    let mut puffer = vec![0u8; u32::from_ne_bytes(length) as usize];
    strom
        .read_exact(&mut puffer)
        .map_err(|e| format!("greetd-Antwort abgerissen: {e}"))?;
    serde_json::from_slice(&puffer).map_err(|e| format!("greetd-Antwort unlesbar: {e}"))
}

/// What a display manager tells a Wayland session (gdm did it on the Duo
/// until round 33): the session type and, for the canvas, its desktop names.
fn session_env(cmd: &str) -> Vec<String> {
    let mut env = vec![String::from("XDG_SESSION_TYPE=wayland")];
    if cmd.contains("niri") {
        env.push(String::from("XDG_CURRENT_DESKTOP=niri"));
        if cmd.contains("canvas") {
            env.push(String::from("XDG_SESSION_DESKTOP=matrix-canvas"));
            env.push(String::from("DESKTOP_SESSION=matrix-canvas"));
        }
    }
    env
}

/// The complete login: user + password against PAM, then `cmd` as the
/// session command. Ok = greetd takes over, the greeter may end.
/// For the recovery account (`wache`, passwordless) `passwort` stays
/// empty — greetd then reports success directly without a secret prompt.
pub fn login(user: &str, password: &str, cmd: &str) -> Result<(), String> {
    let sock = std::env::var("GREETD_SOCK")
        .map_err(|_| String::from("kein GREETD_SOCK — läuft der Greeter unter greetd?"))?;
    let mut strom =
        UnixStream::connect(&sock).map_err(|e| format!("greetd-Socket: {e}"))?;

    let mut reply = send(
        &mut strom,
        &serde_json::json!({"type": "create_session", "username": user}),
    )?;

    // Prompt run_loop: we answer secrets with the password, acknowledge
    // info/errors empty — until success or abort.
    loop {
        match reply {
            Answer::Success => break,
            Answer::Error { description, .. } => {
                let _ = send(&mut strom, &serde_json::json!({"type": "cancel_session"}));
                return Err(if description.is_empty() {
                    String::from("Anmeldung abgelehnt")
                } else {
                    description
                });
            }
            Answer::AuthMessage {
                auth_message_type, ..
            } => {
                let field = match auth_message_type.as_str() {
                    "secret" => serde_json::json!({
                        "type": "post_auth_message_response",
                        "response": password
                    }),
                    // visible/info/error: acknowledged
                    _ => serde_json::json!({
                        "type": "post_auth_message_response",
                        "response": null
                    }),
                };
                reply = send(&mut strom, &field)?;
            }
        }
    }

    // Through the shell: session execs may carry arguments
    // (the recovery command did — and died as one-word argv).
    match send(
        &mut strom,
        &serde_json::json!({"type": "start_session", "cmd": ["sh", "-c", cmd], "env": session_env(cmd)}),
    )? {
        Answer::Success => Ok(()),
        Answer::Error { description, .. } => Err(description),
        _ => Err(String::from("greetd: unerwartete Antwort auf start_session")),
    }
}
