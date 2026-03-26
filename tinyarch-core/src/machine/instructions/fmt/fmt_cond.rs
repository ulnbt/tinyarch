use super::{Condition, fmt};

impl fmt::Display for Condition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Condition::Eq    => "==",
            Condition::NEq   => "!=",
            Condition::Gt    => ">",
            Condition::NGt   => "<=",
            Condition::GtU   => ">u",
            Condition::NGtU  => "<=u",
            Condition::Mask  => "&",
            Condition::NMask => "!&",
            Condition::EqF   => "==f",
            Condition::NEqF  => "!=f",
            Condition::GtF   => ">f",
            Condition::NGtF  => "<=f",
            Condition::Par   => "p",
            Condition::NPar  => "!p",
            Condition::Even  => "%2",
            Condition::NEven => "!%2",
        };
        write!(f, "{}", s)
    }
}