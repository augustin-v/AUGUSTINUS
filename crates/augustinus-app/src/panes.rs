#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PaneId {
    Motivation,
    General,
    Agents,
    Stats,
}

impl PaneId {
    pub fn next(self) -> Self {
        match self {
            Self::Motivation => Self::General,
            Self::General => Self::Agents,
            Self::Agents => Self::Stats,
            Self::Stats => Self::Motivation,
        }
    }
}

