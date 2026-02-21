use std::{
    io::{Read, Write},
    process::{Command, Stdio},
    sync::mpsc,
    thread,
    time::Duration,
};

use nu_protocol::{ShellError, Span};
use uucore::process::ChildExt;

use super::provider::Clipboard;

pub(crate) struct ClipBoardTermux;

impl ClipBoardTermux {
    pub fn new() -> Self {
        Self
    }
}

impl Clipboard for ClipBoardTermux {
    fn copy_text(&self, text: &str) -> Result<(), ShellError> {
        let mut child = Command::new("termux-clipboard-set")
            .stdin(Stdio::piped())
            .spawn()
            .map_err(|_| ShellError::GenericError {
                error: "Failed to run `termux-clipboard-set`".into(),
                msg: String::new(),
                help: Some("Make sure you have termux-api package installed.".into()),
                span: None,
                inner: vec![],
            })?;

        let stdin = child.stdin.as_mut().expect("stdin is present");
        stdin
            .write_all(text.as_bytes())
            .map_err(|_| ShellError::GenericError {
                error: "Failed to run `termux-clipboard-set`".into(),
                msg: String::new(),
                help: Some("Make sure you have termux-api package installed.".into()),
                span: None,
                inner: vec![],
            })?;

        let success = child
            .wait_or_timeout(Duration::from_secs(5), None)
            .map_err(|_| {
                let _ = child.kill();
                let _ = child.wait();

                ShellError::GenericError {
                    error: "Failed to copy text.".into(),
                    msg: String::new(),
                    help: Some("Make sure you have Termux:API add-on installed.".into()),
                    span: None,
                    inner: vec![],
                }
            })?
            .is_some_and(|status| status.success());

        if !success {
            return Err(ShellError::GenericError {
                error: "Failed to copy text.".into(),
                msg: String::new(),
                help: Some("Make sure you have Termux:API add-on installed.".into()),
                span: None,
                inner: vec![],
            });
        }

        Ok(())
    }

    fn get_text(&self) -> Result<String, ShellError> {
        let mut child = Command::new("termux-clipboard-get")
            .stdout(Stdio::piped())
            .spawn()
            .map_err(|_| ShellError::GenericError {
                error: "Failed to run `termux-clipboard-get`".into(),
                msg: String::new(),
                help: Some("Make sure you have termux-api package installed.".into()),
                span: None,
                inner: vec![],
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
            Err(_) => Err(ShellError::GenericError {
                error: "Failed to get clipboard text (timeout).".into(),
                msg: String::new(),
                span: None,
                help: Some("Make sure you have Termux:API add-on installed.".into()),
                inner: vec![],
            }),
        }
    }
}
