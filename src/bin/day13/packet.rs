use std::fmt::Display;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Packet {
    contents: Vec<PacketElement>,
}

#[derive(Debug, Clone)]
pub enum PacketElement {
    List(Vec<PacketElement>),
    Int(u32),
}

impl Packet {
    pub fn new(contents: Vec<PacketElement>) -> Self {
        Self { contents }
    }
}

impl Display for Packet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[")?;

        if let Some((last, first)) = self.contents.split_last() {
            for element in first {
                write!(f, "{},", element)?;
            }
            last.fmt(f)?;
        }

        write!(f, "]")?;

        Ok(())
    }
}

impl Display for PacketElement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::List(list) => {
                write!(f, "[")?;

                if let Some((last, first)) = list.split_last() {
                    for element in first {
                        write!(f, "{},", element)?;
                    }
                    last.fmt(f)?;
                }

                write!(f, "]")?;

                Ok(())
            }
            Self::Int(int) => write!(f, "{}", int),
        }
    }
}

impl PartialEq for PacketElement {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Int(lhs), Self::Int(rhs)) => lhs.eq(rhs),
            (Self::List(lhs), Self::List(rhs)) => lhs.eq(rhs),

            (Self::List(_), Self::Int(rhs)) => self.eq(&Self::List(vec![Self::Int(*rhs)])),
            (Self::Int(lhs), Self::List(_)) => Self::List(vec![Self::Int(*lhs)]).eq(other),
        }
    }
}

impl Eq for PacketElement {}

impl PartialOrd for PacketElement {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (Self::Int(lhs), Self::Int(rhs)) => lhs.partial_cmp(rhs),
            (Self::List(lhs), Self::List(rhs)) => lhs.partial_cmp(rhs),

            (Self::List(_), Self::Int(rhs)) => self.partial_cmp(&Self::List(vec![Self::Int(*rhs)])),
            (Self::Int(lhs), Self::List(_)) => Self::List(vec![Self::Int(*lhs)]).partial_cmp(other),
        }
    }
}

impl Ord for PacketElement {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}
