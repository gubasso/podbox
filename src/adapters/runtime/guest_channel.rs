use std::fmt;

// Host-to-guest command transport contract:
// - every request carries a per-instance token before argv data;
// - frames are length-delimited and bounded by MAX_FRAME_BYTES;
// - argv is transmitted as bytes, not a rendered shell string;
// - argc and per-argument sizes are bounded independently;
// - protocol version is explicit so incompatible agents fail closed;
// - concurrent exec requests must be handled as independent child processes by
//   the guest agent, with shutdown represented as channel failure if the agent
//   exits before returning a status.
pub(crate) const PROTOCOL_VERSION: u16 = 1;
pub(crate) const MAX_FRAME_BYTES: usize = 1024 * 1024;
pub(crate) const MAX_ARGC: usize = 256;
pub(crate) const MAX_ARG_BYTES: usize = 64 * 1024;

const MAGIC: &[u8; 4] = b"PBXG";

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GuestExecRequest {
    pub(crate) token: Vec<u8>,
    pub(crate) argv: Vec<Vec<u8>>,
}

impl GuestExecRequest {
    pub(crate) fn new(token: Vec<u8>, argv: Vec<Vec<u8>>) -> Result<Self, GuestChannelError> {
        validate_args(&argv)?;
        if token.is_empty() {
            return Err(GuestChannelError::MissingToken);
        }
        if token.len() > MAX_ARG_BYTES {
            return Err(GuestChannelError::ArgumentTooLarge);
        }
        Ok(Self { token, argv })
    }

    pub(crate) fn assert_no_shell_escape(&self) -> Result<(), GuestChannelError> {
        for window in self.argv.windows(2) {
            if window[0] == b"sh" && window[1] == b"-c" {
                return Err(GuestChannelError::ShellEscape);
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum GuestExit {
    Code(u8),
    Signal(u8),
    MissingExecutable,
    NotExecutable,
    ChannelFailure,
}

impl GuestExit {
    pub(crate) fn status_code(&self) -> u8 {
        match self {
            Self::Code(code) => *code,
            Self::Signal(signal) => 128u8.saturating_add(*signal),
            Self::MissingExecutable => 127,
            Self::NotExecutable => 126,
            Self::ChannelFailure => 125,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum GuestChannelError {
    EmptyArgv,
    TooManyArgs,
    ArgumentTooLarge,
    FrameTooLarge,
    Truncated,
    BadMagic,
    UnsupportedVersion(u16),
    MissingToken,
    ShellEscape,
}

impl fmt::Display for GuestChannelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyArgv => write!(f, "guest exec argv is empty"),
            Self::TooManyArgs => write!(f, "guest exec argv has too many arguments"),
            Self::ArgumentTooLarge => write!(f, "guest exec argument exceeds limit"),
            Self::FrameTooLarge => write!(f, "guest channel frame exceeds limit"),
            Self::Truncated => write!(f, "guest channel frame is truncated"),
            Self::BadMagic => write!(f, "guest channel frame has invalid magic"),
            Self::UnsupportedVersion(version) => {
                write!(f, "guest channel protocol version {version} is unsupported")
            }
            Self::MissingToken => write!(f, "guest channel request is missing its instance token"),
            Self::ShellEscape => write!(f, "guest channel request attempted sh -c execution"),
        }
    }
}

pub(crate) fn encode_request(request: &GuestExecRequest) -> Result<Vec<u8>, GuestChannelError> {
    request.assert_no_shell_escape()?;
    validate_args(&request.argv)?;
    let mut payload = Vec::new();
    payload.extend(MAGIC);
    payload.extend(PROTOCOL_VERSION.to_be_bytes());
    write_field(&mut payload, &request.token)?;
    write_u16(&mut payload, request.argv.len())?;
    for arg in &request.argv {
        write_field(&mut payload, arg)?;
    }
    if payload.len() > MAX_FRAME_BYTES {
        return Err(GuestChannelError::FrameTooLarge);
    }
    let mut frame = Vec::with_capacity(payload.len() + 4);
    frame.extend((payload.len() as u32).to_be_bytes());
    frame.extend(payload);
    Ok(frame)
}

pub(crate) fn decode_request(frame: &[u8]) -> Result<GuestExecRequest, GuestChannelError> {
    if frame.len() < 4 {
        return Err(GuestChannelError::Truncated);
    }
    let payload_len = u32::from_be_bytes(frame[0..4].try_into().expect("length slice")) as usize;
    if payload_len > MAX_FRAME_BYTES {
        return Err(GuestChannelError::FrameTooLarge);
    }
    if frame.len() != payload_len + 4 {
        return Err(GuestChannelError::Truncated);
    }
    let payload = &frame[4..];
    let mut cursor = Cursor::new(payload);
    if cursor.take(4)? != MAGIC {
        return Err(GuestChannelError::BadMagic);
    }
    let version = cursor.u16()?;
    if version != PROTOCOL_VERSION {
        return Err(GuestChannelError::UnsupportedVersion(version));
    }
    let token = cursor.field()?;
    let argc = cursor.u16()? as usize;
    if argc == 0 {
        return Err(GuestChannelError::EmptyArgv);
    }
    if argc > MAX_ARGC {
        return Err(GuestChannelError::TooManyArgs);
    }
    let mut argv = Vec::with_capacity(argc);
    for _ in 0..argc {
        argv.push(cursor.field()?);
    }
    if !cursor.is_empty() {
        return Err(GuestChannelError::Truncated);
    }
    GuestExecRequest::new(token, argv)
}

pub(crate) fn live_transport_unavailable() -> crate::adapters::runtime::RuntimeError {
    crate::adapters::runtime::RuntimeError::Unsupported(
        concat!(
            "guest vsock/PTTY/signal transport is a raw Linux syscall integration point ",
            "and is not wired in this build",
        )
        .to_string(),
    )
}

fn validate_args(argv: &[Vec<u8>]) -> Result<(), GuestChannelError> {
    if argv.is_empty() {
        return Err(GuestChannelError::EmptyArgv);
    }
    if argv.len() > MAX_ARGC {
        return Err(GuestChannelError::TooManyArgs);
    }
    if argv.iter().any(|arg| arg.len() > MAX_ARG_BYTES) {
        return Err(GuestChannelError::ArgumentTooLarge);
    }
    Ok(())
}

fn write_u16(out: &mut Vec<u8>, value: usize) -> Result<(), GuestChannelError> {
    if value > u16::MAX as usize {
        return Err(GuestChannelError::TooManyArgs);
    }
    out.extend((value as u16).to_be_bytes());
    Ok(())
}

fn write_field(out: &mut Vec<u8>, bytes: &[u8]) -> Result<(), GuestChannelError> {
    if bytes.len() > MAX_ARG_BYTES {
        return Err(GuestChannelError::ArgumentTooLarge);
    }
    out.extend((bytes.len() as u32).to_be_bytes());
    out.extend(bytes);
    Ok(())
}

struct Cursor<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Cursor<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    fn take(&mut self, len: usize) -> Result<&'a [u8], GuestChannelError> {
        let end = self
            .pos
            .checked_add(len)
            .ok_or(GuestChannelError::FrameTooLarge)?;
        let out = self
            .data
            .get(self.pos..end)
            .ok_or(GuestChannelError::Truncated)?;
        self.pos = end;
        Ok(out)
    }

    fn u16(&mut self) -> Result<u16, GuestChannelError> {
        let bytes = self.take(2)?;
        Ok(u16::from_be_bytes(bytes.try_into().expect("u16 slice")))
    }

    fn u32(&mut self) -> Result<u32, GuestChannelError> {
        let bytes = self.take(4)?;
        Ok(u32::from_be_bytes(bytes.try_into().expect("u32 slice")))
    }

    fn field(&mut self) -> Result<Vec<u8>, GuestChannelError> {
        let len = self.u32()? as usize;
        if len > MAX_ARG_BYTES {
            return Err(GuestChannelError::ArgumentTooLarge);
        }
        Ok(self.take(len)?.to_vec())
    }

    fn is_empty(&self) -> bool {
        self.pos == self.data.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(argv: Vec<&[u8]>) -> GuestExecRequest {
        GuestExecRequest::new(
            b"instance-token".to_vec(),
            argv.into_iter().map(|arg| arg.to_vec()).collect(),
        )
        .expect("request")
    }

    #[test]
    fn argv_vector_round_trips_as_bytes() {
        let original = request(vec![b"/bin/echo", b"hello", &[0xff, b'x']]);
        let encoded = encode_request(&original).expect("encode");
        assert_eq!(decode_request(&encoded).expect("decode"), original);
    }

    #[test]
    fn empty_argv_is_rejected() {
        assert_eq!(
            GuestExecRequest::new(b"token".to_vec(), Vec::new()),
            Err(GuestChannelError::EmptyArgv)
        );
    }

    #[test]
    fn oversized_frame_is_rejected() {
        let mut frame = Vec::new();
        frame.extend(((MAX_FRAME_BYTES + 1) as u32).to_be_bytes());
        assert_eq!(
            decode_request(&frame),
            Err(GuestChannelError::FrameTooLarge)
        );
    }

    #[test]
    fn oversized_argument_is_rejected() {
        assert_eq!(
            GuestExecRequest::new(b"token".to_vec(), vec![vec![b'x'; MAX_ARG_BYTES + 1]]),
            Err(GuestChannelError::ArgumentTooLarge)
        );
    }

    #[test]
    fn exit_status_mapping_matches_shell_conventions_without_using_shell() {
        assert_eq!(GuestExit::Code(42).status_code(), 42);
        assert_eq!(GuestExit::Signal(9).status_code(), 137);
        assert_eq!(GuestExit::MissingExecutable.status_code(), 127);
        assert_eq!(GuestExit::NotExecutable.status_code(), 126);
    }

    #[test]
    fn shell_escape_is_rejected() {
        let req = request(vec![b"sh", b"-c", b"echo no"]);
        assert_eq!(encode_request(&req), Err(GuestChannelError::ShellEscape));
    }
}
