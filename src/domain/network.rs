use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub(crate) struct AllowEntry(pub(crate) String);

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum AllowEntryKind {
    Host,
    Cidr,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ParsedAllowEntry {
    pub(crate) canonical: String,
    pub(crate) kind: AllowEntryKind,
    pub(crate) target: String,
    pub(crate) port: Option<u16>,
}

impl AllowEntry {
    pub(crate) fn parse(raw: &str) -> Result<Self, String> {
        parse_allow_entry(raw).map(|entry| AllowEntry(entry.canonical))
    }

    pub(crate) fn parsed(&self) -> Result<ParsedAllowEntry, String> {
        parse_allow_entry(&self.0)
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct Allowlist(pub(crate) Vec<AllowEntry>);

impl Allowlist {
    pub(crate) fn union(&self, other: &Self) -> Self {
        let mut out = self.0.clone();
        for entry in &other.0 {
            let canonical = entry
                .parsed()
                .map(|parsed| parsed.canonical)
                .unwrap_or_else(|_| entry.0.clone());
            if !out.iter().any(|existing| {
                existing
                    .parsed()
                    .map(|parsed| parsed.canonical == canonical)
                    .unwrap_or(existing.0 == canonical)
            }) {
                out.push(AllowEntry(canonical));
            }
        }
        Self(out)
    }

    pub(crate) fn validate(&self) -> Result<Self, String> {
        let mut out = Allowlist::default();
        for entry in &self.0 {
            let parsed = entry.parsed()?;
            out = out.union(&Allowlist(vec![AllowEntry(parsed.canonical)]));
        }
        Ok(out)
    }
}

fn parse_allow_entry(raw: &str) -> Result<ParsedAllowEntry, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() || trimmed.contains(char::is_whitespace) {
        return Err("allow entry must be a host or CIDR with optional :port".to_string());
    }
    let (target, port) = split_port(trimmed)?;
    let kind = if target.contains('/') {
        validate_cidr(target)?;
        AllowEntryKind::Cidr
    } else {
        validate_host(target)?;
        AllowEntryKind::Host
    };
    let target = target.to_ascii_lowercase();
    let canonical = match port {
        Some(port) => format!("{target}:{port}"),
        None => target.clone(),
    };
    Ok(ParsedAllowEntry {
        canonical,
        kind,
        target,
        port,
    })
}

fn split_port(value: &str) -> Result<(&str, Option<u16>), String> {
    let colon_count = value.matches(':').count();
    if colon_count == 0 {
        return Ok((value, None));
    }
    if colon_count > 1 {
        return Err("IPv6 allow entries are not supported in this round".to_string());
    }
    let (target, port) = value
        .rsplit_once(':')
        .ok_or_else(|| "allow entry port is malformed".to_string())?;
    if target.is_empty() || port.is_empty() {
        return Err("allow entry port is malformed".to_string());
    }
    let port = port
        .parse::<u16>()
        .map_err(|_| "allow entry port must be 1-65535".to_string())?;
    if port == 0 {
        return Err("allow entry port must be 1-65535".to_string());
    }
    Ok((target, Some(port)))
}

fn validate_host(host: &str) -> Result<(), String> {
    if host.len() > 253 {
        return Err("host allow entry is too long".to_string());
    }
    for label in host.split('.') {
        if label.is_empty() || label.len() > 63 {
            return Err("host allow entry has an empty or oversized label".to_string());
        }
        let bytes = label.as_bytes();
        if bytes.first() == Some(&b'-') || bytes.last() == Some(&b'-') {
            return Err("host labels must not start or end with '-'".to_string());
        }
        if !bytes
            .iter()
            .all(|b| b.is_ascii_alphanumeric() || *b == b'-')
        {
            return Err("host allow entry contains an invalid character".to_string());
        }
    }
    Ok(())
}

fn validate_cidr(cidr: &str) -> Result<(), String> {
    let (addr, prefix) = cidr
        .split_once('/')
        .ok_or_else(|| "CIDR allow entry is malformed".to_string())?;
    let octets: Vec<&str> = addr.split('.').collect();
    if octets.len() != 4 {
        return Err("only IPv4 CIDR allow entries are supported in this round".to_string());
    }
    for octet in octets {
        octet
            .parse::<u8>()
            .map_err(|_| "CIDR allow entry has an invalid IPv4 octet".to_string())?;
    }
    let prefix = prefix
        .parse::<u8>()
        .map_err(|_| "CIDR allow entry prefix must be 0-32".to_string())?;
    if prefix > 32 {
        return Err("CIDR allow entry prefix must be 0-32".to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn union_is_ordered_and_deduped() {
        let a = Allowlist(vec![
            AllowEntry("one.example".into()),
            AllowEntry("two.example".into()),
        ]);
        let b = Allowlist(vec![
            AllowEntry("two.example".into()),
            AllowEntry("three.example".into()),
        ]);
        assert_eq!(
            a.union(&b).0,
            vec![
                AllowEntry("one.example".into()),
                AllowEntry("two.example".into()),
                AllowEntry("three.example".into())
            ]
        );
    }

    #[test]
    fn entries_are_canonicalized_and_validated() {
        assert_eq!(
            AllowEntry::parse("GitHub.COM:443").unwrap(),
            AllowEntry("github.com:443".to_string())
        );
        assert_eq!(
            AllowEntry::parse("10.0.0.0/8").unwrap(),
            AllowEntry("10.0.0.0/8".to_string())
        );
        assert!(AllowEntry::parse("bad host").is_err());
        assert!(AllowEntry::parse("example.com:0").is_err());
    }
}
