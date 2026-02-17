#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LocDelta {
    pub added: u64,
    pub removed: u64,
}

impl LocDelta {
    pub fn parse_git_numstat(output: &str) -> Self {
        let mut added: u64 = 0;
        let mut removed: u64 = 0;

        for line in output.lines() {
            let mut parts = line.split('\t');
            let a = parts.next().unwrap_or("");
            let r = parts.next().unwrap_or("");
            if a == "-" || r == "-" {
                continue;
            }
            let Ok(a) = a.parse::<u64>() else { continue };
            let Ok(r) = r.parse::<u64>() else { continue };
            added = added.saturating_add(a);
            removed = removed.saturating_add(r);
        }

        Self { added, removed }
    }
}

