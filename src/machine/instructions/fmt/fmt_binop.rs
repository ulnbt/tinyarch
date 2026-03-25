use super::{BinaryOperator, fmt};

impl fmt::Display for BinaryOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            // Signed integer
            BinaryOperator::Add     => "+",
            BinaryOperator::Sub     => "-",
            BinaryOperator::SubR    => "-r",
            BinaryOperator::Mul     => "*",
            BinaryOperator::Div     => "/",
            BinaryOperator::Mod     => "%",
            BinaryOperator::Min     => "min",
            BinaryOperator::Max     => "max",
            // Float
            BinaryOperator::AddF    => "+f",
            BinaryOperator::SubF    => "-f",
            BinaryOperator::SubFR   => "-rf",
            BinaryOperator::MulF    => "*f",
            BinaryOperator::DivF    => "/f",
            BinaryOperator::ModF    => "%f",
            BinaryOperator::MinF    => "minf",
            BinaryOperator::MaxF    => "maxf",
            // Bitwise
            BinaryOperator::And     => "&",
            BinaryOperator::Or      => "|",
            BinaryOperator::Xor     => "^",
            BinaryOperator::Xnor    => "!^",
            // Unsigned
            BinaryOperator::MinU    => "minu",
            BinaryOperator::MaxU    => "maxu",
            BinaryOperator::DivU    => "/u",
            BinaryOperator::ModU    => "%u",
            // Shifts / rots
            BinaryOperator::Rol     => "rol",
            BinaryOperator::Ror     => "ror",
            BinaryOperator::Lsl     => "<<",
            BinaryOperator::Lsr     => ">>",
            BinaryOperator::Asr     => ">>>",
            // Special
            BinaryOperator::Fpintexp => "<<f",
            BinaryOperator::Nop     => "nop",
        };
        write!(f, "{}", s)
    }
}