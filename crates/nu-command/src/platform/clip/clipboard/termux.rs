use std::{
    io::Write,
    process::{Command, Stdio},
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
            .map_err(|err| ShellError::GenericError {
                error: "Failed to run `termux-clipboard-set`".into(),
                msg: err.to_string(),
                help: Some("Make sure you have termux-api package installed.".into()),
                span: None,
                inner: vec![],
            })?;

        let stdin = child.stdin.as_mut().expect("Stdio::piped() is used");
        stdin
            .write_all(text.as_bytes())
            .map_err(|err| ShellError::GenericError {
                error: err.to_string(),
                msg: "msg test write_all".into(),
                span: None,
                help: None,
                inner: vec![],
            })?;

        let success = child
            .wait_or_timeout(Duration::from_secs(5), None)
            .map_err(|err| ShellError::GenericError {
                error: err.to_string(),
                msg: "idfl".into(),
                span: None,
                help: None,
                inner: vec![],
            })?
            .is_some_and(|status| status.success());

        if !success {
            return Err(ShellError::GenericError {
                error: "Failed to copy text.".into(),
                msg: "".into(),
                help: Some("Make sure you have Termux:API add-on installed.".into()),
                span: None,
                inner: vec![],
            });
        }

        Ok(())
    }

    fn get_text(&self) -> Result<String, ShellError> {
        let output = Command::new("termux-clipboard-get").output().map_err(|_| {
            ShellError::GenericError {
                error: "Failed to run `termux-clipboard-get`".into(),
                msg: "".into(),
                help: Some("Make sure you have termux-api package installed.".into()),
                span: None,
                inner: vec![],
            }
        })?;

        String::try_from(output.stdout).map_err(|_| ShellError::CantConvert {
            to_type: "string".into(),
            from_type: "binary".into(),
            span: Span::unknown(),
            help: None,
        })
    }
}
