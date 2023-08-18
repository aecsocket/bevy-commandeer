/// Respond to a command sender in a context with an Ok outcome, using [`format!`] syntax.
#[macro_export]
macro_rules! respond_ok {
    ($responder: ident, $fmt: literal$(, $($arg:expr),* $(,)?)?) => {
        {
            let msg = format!($fmt$(, $($arg),*)?);
            $responder.ok(msg);
        }
    };
}

/// Respond to a command sender in a context with an Err outcome, using [`format!`] syntax.
#[macro_export]
macro_rules! respond_err {
    ($responder: ident, $fmt: literal$(, $($arg:expr),* $(,)?)?) => {
        {
            let msg = format!($fmt$(, $($arg),*)?);
            $responder.err(msg);
        }
    };
}
