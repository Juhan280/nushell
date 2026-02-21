use std::{
    io::{Read, Write},
    process::{Command, Stdio},
    sync::mpsc,
    thread,
    time::Duration,
};

use nu_protocol::{ShellError, Span, shell_error::generic::GenericError};
use uucore::process::ChildExt;

use super::provider::Clipboard;

pub(crate) struct ClipBoardTermux;

impl ClipBoardTermux {
    pub fn new() -> Self {
        Self
    }
}

const INSTALL_TERMUX_API_PACKAGE_TEXT: &str = "Make sure you have termux-api package installed.";
const INSTALL_TERMUX_API_ADDON_TEXT: &str = "Make sure you have Termux:API add-on installed.";

impl Clipboard for ClipBoardTermux {
    fn copy_text(&self, text: &str) -> Result<(), ShellError> {
        let mut child = Command::new("termux-clipboard-set")
            .stdin(Stdio::piped())
            .spawn()
            .map_err(|_| {
                GenericError::new_internal(
                    "Failed to run `termux-clipboard-set`",
                    "command not found",
                )
                .with_help(INSTALL_TERMUX_API_PACKAGE_TEXT)
            })?;

        let stdin = child.stdin.as_mut().expect("stdin is present");
        stdin.write_all(text.as_bytes()).map_err(|err| {
            GenericError::new_internal("Failed to run `termux-clipboard-set`", err.to_string())
                .with_help(INSTALL_TERMUX_API_PACKAGE_TEXT)
        })?;

        let success = child
            .wait_or_timeout(Duration::from_secs(5), None)
            .map_err(|err| {
                let _ = child.kill();
                let _ = child.wait();

                GenericError::new_internal("Failed to copy text", err.to_string())
                    .with_help(INSTALL_TERMUX_API_ADDON_TEXT)
            })?
            .is_some_and(|status| status.success());

        if !success {
            return Err(GenericError::new_internal(
                "Failed to copy text",
                "termux-clipboard-set did not respond",
            )
            .with_help(INSTALL_TERMUX_API_ADDON_TEXT)
            .into());
        }

        Ok(())
    }

    fn get_text(&self) -> Result<String, ShellError> {
        let mut child = Command::new("termux-clipboard-get")
            .stdout(Stdio::piped())
            .spawn()
            .map_err(|_| {
                GenericError::new_internal(
                    "Failed to run `termux-clipboard-get`",
                    "command not found",
                )
                .with_help(INSTALL_TERMUX_API_PACKAGE_TEXT)
            })?;

        let mut stdout = child.stdout.take().expect("stdout is present");

        let (tx, rx) = mpsc::channel();

        thread::spawn(move || {
            let mut buf = String::new();
            let res = stdout.read_to_string(&mut buf);
            let _ = tx.send(res.map(|_| buf));
        });

        let output = rx.recv_timeout(Duration::from_secs(5));

        let _ = child.kill();
        let _ = child.wait();

        match output {
            Ok(Ok(str)) => Ok(str),
            Ok(Err(_)) => Err(ShellError::CantConvert {
                to_type: "string".into(),
                from_type: "binary".into(),
                span: Span::unknown(),
                help: None,
            }),
            Err(err) => Err(GenericError::new_internal(
                "Failed to get clipboard text",
                err.to_string(),
            )
            .with_help(INSTALL_TERMUX_API_ADDON_TEXT)
            .into()),
        }
    }
}
