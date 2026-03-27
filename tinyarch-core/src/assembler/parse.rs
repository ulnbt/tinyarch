use crate::machine::{BinaryOperator, Condition, Instruction};

// ── Error ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum ParseError {
    InvalidSyntax(String),
    InvalidRegister(String),
    InvalidOperator(String),
    InvalidOperand(String),
    NonStrict(String),
    ImmediateOutOfRange { value: u64, bits: u32 },
    Unimplemented(String),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidSyntax(s)                  => write!(f, "invalid syntax: {s}"),
            Self::InvalidRegister(s)                => write!(f, "invalid register: {s}"),
            Self::InvalidOperator(s)                => write!(f, "unknown operator: {s}"),
            Self::InvalidOperand(s)                 => write!(f, "invalid operand: {s}"),
            Self::NonStrict(s)                      => write!(f, "non-strict instruction: {s}"),
            Self::ImmediateOutOfRange{value, bits}  => write!(f, "immediate {value} does not fit in {bits} bits"),
            Self::Unimplemented(s)                  => write!(f, "unimplemented: {s}"),
        }
    }
}

impl std::error::Error for ParseError {}

// ── Operand IR ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
enum Operand {
    Zero,       // 0
    Reg(u8),    // rx
    MemReg(u8), // @rx
    RegIdx { base: u8, idx: u8, mul: u32 },
    MemRegMul { idx: u8, mul: u32 },
    Imm(u32),
    MemImm(u32),
}

impl Operand {
    /// Convert to an r_rix byte + multiplier product.
    fn to_r_rix(&self) -> Result<(u8, u32), ParseError> {
        match self {
            Operand::Zero                        => Ok((0x00, 1)),
            Operand::Reg(r)                      => Ok((*r << 4, 1)),
            Operand::MemReg(rix)                 => Ok((*rix, 1)),
            Operand::RegIdx { base, idx, mul }   => Ok(((*base << 4) | *idx, *mul)),
            Operand::MemRegMul { idx, mul }      => Ok((*idx, *mul)),
            other => Err(ParseError::NonStrict(
                format!("{other:?} is not a valid r_rix operand"),
            )),
        }
    }

    /// Like `to_r_rix` but rejects mul != 1 (OpRg / BrCoRg do not allow multipliers).
    fn to_r_rix_no_mul(&self) -> Result<u8, ParseError> {
        let (rrix, mul) = self.to_r_rix()?;
        if mul != 1 {
            return Err(ParseError::NonStrict(
                "multipliers are not permitted on operands in this instruction form".into(),
            ));
        }
        Ok(rrix)
    }

    /// Require a plain 4-bit register (OpIm / BrCoIm).
    fn to_reg4(&self) -> Result<u8, ParseError> {
        match self {
            Operand::Zero   => Ok(0),
            Operand::Reg(r) => Ok(*r),
            other => Err(ParseError::NonStrict(format!(
                "{other:?} is not a plain register, but immediate mode requires one",
            ))),
        }
    }
}

// ── Lexical helpers ───────────────────────────────────────────────────────────

/// Greedily consume one operand token from the start of `s`.
/// Returns `(token, remainder)` with leading whitespace stripped from remainder.
///
/// Handles: `0`, `rx`, `@rx`, `@N`, `rx[...]`, `@rx[...]`, numeric immediates.
fn consume_operand(s: &str) -> Option<(&str, &str)> {
    let s = s.trim_start();
    let b = s.as_bytes();
    if b.is_empty() { return None; }
    let mut i = 0;

    if b[i] == b'@' {
        i += 1;
        if i < b.len() && b[i] == b'r' {
            // @rx  or  @rx[...]
            i += 1; // skip 'r'
            if i < b.len() && (b[i] as char).is_ascii_hexdigit() { i += 1; }
            if i < b.len() && b[i] == b'[' {
                i += 1;
                while i < b.len() && b[i] != b']' { i += 1; }
                if i < b.len() { i += 1; } // consume ']'
            }
        } else {
            // @N — consume until whitespace
            while i < b.len() && !(b[i] as char).is_ascii_whitespace() { i += 1; }
        }
        return Some((&s[..i], s[i..].trim_start()));
    }

    if b[i] == b'r' {
        // rx  or  rx[...]
        i += 1;
        if i < b.len() && (b[i] as char).is_ascii_hexdigit() { i += 1; }
        if i < b.len() && b[i] == b'[' {
            i += 1;
            while i < b.len() && b[i] != b']' { i += 1; }
            if i < b.len() { i += 1; }
        }
        return Some((&s[..i], s[i..].trim_start()));
    }

    // Numeric immediate (or bare `0`) — consume until whitespace
    while i < b.len() && !(b[i] as char).is_ascii_whitespace() { i += 1; }
    if i > 0 { Some((&s[..i], s[i..].trim_start())) } else { None }
}

fn parse_int(s: &str) -> Option<u64> {
    let s = s.trim();
    if let Some(h) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        u64::from_str_radix(h, 16).ok()
    } else if let Some(b) = s.strip_prefix("0b").or_else(|| s.strip_prefix("0B")) {
        u64::from_str_radix(b, 2).ok()
    } else {
        s.parse::<u64>().ok()
    }
}

/// Parse a register name `r1`–`rF`, returning the 4-bit index.
pub fn parse_reg_name(s: &str) -> Result<u8, ParseError> {
    let s = s.trim();
    let hex = s.strip_prefix('r')
        .ok_or_else(|| ParseError::InvalidRegister(s.to_string()))?;
    let n = u8::from_str_radix(hex, 16)
        .map_err(|_| ParseError::InvalidRegister(s.to_string()))?;
    if n == 0 || n > 0xF {
        return Err(ParseError::InvalidRegister(format!(
            "register r{n:X} out of range r1..=rF"
        )));
    }
    Ok(n)
}

fn parse_operand(s: &str) -> Result<Operand, ParseError> {
    let s = s.trim();

    if s == "0" { return Ok(Operand::Zero); }

    // Float detection — leave to the caller to handle
    if s.contains('.') || s.contains('e') && !s.starts_with("0x") {
        return Err(ParseError::Unimplemented(
            "float literal parsing is not yet implemented".into(),
        ));
    }

    // `@...`
    if let Some(rest) = s.strip_prefix('@') {
        if rest.starts_with('r') {
            if let Some(bi) = rest.find('[') {
                // @rx[*m]
                let rix = parse_reg_name(&rest[..bi])?;
                let inner = rest[bi + 1..].strip_suffix(']')
                    .ok_or_else(|| ParseError::InvalidSyntax(format!("unclosed '[' in '{s}'")))?;
                let mul = inner
                    .strip_prefix('*')
                    .and_then(|m| m.trim().parse::<u32>().ok())
                    .ok_or_else(|| ParseError::InvalidSyntax(
                        format!("expected '*N' inside brackets in '{s}'"),
                    ))?;
                return Ok(Operand::MemRegMul { idx: rix, mul });
            }
            return Ok(Operand::MemReg(parse_reg_name(rest)?));
        }
        // @N
        let val = parse_int(rest)
            .ok_or_else(|| ParseError::InvalidOperand(s.to_string()))?;
        return Ok(Operand::MemImm(val as u32));
    }

    // `rx` or `rx[ry*k*...]`
    if s.starts_with('r') {
        if let Some(bi) = s.find('[') {
            let base = parse_reg_name(&s[..bi])?;
            let inner = s[bi + 1..].strip_suffix(']')
                .ok_or_else(|| ParseError::InvalidSyntax(format!("unclosed '[' in '{s}'")))?;
            // inner = "ry"  |  "ry*k1*k2*..."
            let parts: Vec<&str> = inner.split('*').collect();
            let idx = parse_reg_name(parts[0].trim())?;
            let mul = parts[1..].iter().try_fold(1u32, |acc, p| {
                p.trim().parse::<u32>().map(|v| acc * v).map_err(|_| {
                    ParseError::InvalidSyntax(format!("invalid multiplier '{p}' in '{s}'"))
                })
            })?;
            return Ok(Operand::RegIdx { base, idx, mul });
        }
        return Ok(Operand::Reg(parse_reg_name(s)?));
    }

    // Bare integer
    if let Some(val) = parse_int(s) {
        return Ok(Operand::Imm(val as u32));
    }

    Err(ParseError::InvalidOperand(s.to_string()))
}

// ── Operator tables (longest match first — order is critical) ────────────────

// Binop operators. Multi-char prefixes of shorter ones must appear first:
//   `>>>` before `>>`,  `-rf` before `-r` before `-`,  `<<f` before `<<`, etc.
const BINOP_STRS: &[&str] = &[
    ">>>", "<<f", "-rf", ">>", "<<",
    "!^", "-r", "+f", "-f", "*f", "/f", "%f", "/u", "%u",
    "minu", "maxu", "minf", "maxf", "min", "max",
    "rol", "ror",
    "+", "-", "*", "/", "%", "&", "|", "^",
];

// Condition operators.  `<=u` before `<=`,  `==f` before `==`, etc.
const COND_STRS: &[&str] = &[
    "==f", "!=f", "<=f", ">f",
    "==", "!=", "<=u", ">u", "<=", ">",
    "!&", "!%2", "!p",
    "&", "%2", "p", 
];

fn parse_binop(s: &str) -> Result<BinaryOperator, ParseError> {
    match s {
        "+"    => Ok(BinaryOperator::Add),
        "-"    => Ok(BinaryOperator::Sub),
        "-r"   => Ok(BinaryOperator::SubR),
        "*"    => Ok(BinaryOperator::Mul),
        "/"    => Ok(BinaryOperator::Div),
        "%"    => Ok(BinaryOperator::Mod),
        "min"  => Ok(BinaryOperator::Min),
        "max"  => Ok(BinaryOperator::Max),
        "+f"   => Ok(BinaryOperator::AddF),
        "-f"   => Ok(BinaryOperator::SubF),
        "-rf"  => Ok(BinaryOperator::SubFR),
        "*f"   => Ok(BinaryOperator::MulF),
        "/f"   => Ok(BinaryOperator::DivF),
        "%f"   => Ok(BinaryOperator::ModF),
        "minf" => Ok(BinaryOperator::MinF),
        "maxf" => Ok(BinaryOperator::MaxF),
        "&"    => Ok(BinaryOperator::And),
        "|"    => Ok(BinaryOperator::Or),
        "^"    => Ok(BinaryOperator::Xor),
        "!^"   => Ok(BinaryOperator::Xnor),
        "minu" => Ok(BinaryOperator::MinU),
        "maxu" => Ok(BinaryOperator::MaxU),
        "/u"   => Ok(BinaryOperator::DivU),
        "%u"   => Ok(BinaryOperator::ModU),
        "rol"  => Ok(BinaryOperator::Rol),
        "ror"  => Ok(BinaryOperator::Ror),
        "<<"   => Ok(BinaryOperator::Lsl),
        ">>"   => Ok(BinaryOperator::Lsr),
        ">>>"  => Ok(BinaryOperator::Asr),
        "<<f"  => Ok(BinaryOperator::Fpintexp),
        other  => Err(ParseError::InvalidOperator(other.to_string())),
    }
}

fn parse_condition(s: &str) -> Result<Condition, ParseError> {
    match s {
        "=="   => Ok(Condition::Eq),
        "!="   => Ok(Condition::NEq),
        ">"    => Ok(Condition::Gt),
        "<="   => Ok(Condition::NGt),
        ">u"   => Ok(Condition::GtU),
        "<=u"  => Ok(Condition::NGtU),
        "&"    => Ok(Condition::Mask),
        "!&"   => Ok(Condition::NMask),
        "==f"  => Ok(Condition::EqF),
        "!=f"  => Ok(Condition::NEqF),
        ">f"   => Ok(Condition::GtF),
        "<=f"  => Ok(Condition::NGtF),
        "p"    => Ok(Condition::Par),
        "!p"   => Ok(Condition::NPar),
        "%2"   => Ok(Condition::Even),
        "!%2"  => Ok(Condition::NEven),
        other  => Err(ParseError::InvalidOperator(other.to_string())),
    }
}

fn negate_condition(co: Condition) -> Condition {
    match co {
        Condition::Eq    => Condition::NEq,   Condition::NEq   => Condition::Eq,
        Condition::Gt    => Condition::NGt,   Condition::NGt   => Condition::Gt,
        Condition::GtU   => Condition::NGtU,  Condition::NGtU  => Condition::GtU,
        Condition::Mask  => Condition::NMask, Condition::NMask => Condition::Mask,
        Condition::EqF   => Condition::NEqF,  Condition::NEqF  => Condition::EqF,
        Condition::GtF   => Condition::NGtF,  Condition::NGtF  => Condition::GtF,
        Condition::Par   => Condition::NPar,  Condition::NPar  => Condition::Par,
        Condition::Even  => Condition::NEven, Condition::NEven => Condition::Even,
    }
}

// ── Lhs splitters ─────────────────────────────────────────────────────────────

/// Split `"A op B"` → `(a, op, b)` by consuming the first operand greedily,
/// then matching an operator prefix.  Returns `None` if no operator is found
/// (i.e., `s` is just a plain operand — it's a move, not a binop).
fn split_binop_lhs(s: &str) -> Option<(&str, &str, &str)> {
    let (a, rest) = consume_operand(s)?;
    let rest = rest.trim();
    for &op in BINOP_STRS {
        if let Some(after) = rest.strip_prefix(op) {
            let b = after.trim();
            if !b.is_empty() {
                return Some((a, op, b));
            }
        }
    }
    None
}

fn split_condition_lhs(s: &str) -> Option<(&str, &str, &str)> {
    let (a, rest) = consume_operand(s)?;
    let rest = rest.trim();
    for &op in COND_STRS {
        if let Some(after) = rest.strip_prefix(op) {
            let b = after.trim();
            if !b.is_empty() {
                return Some((a, op, b));
            }
        }
    }
    None
}

// ── Per-group parsers ─────────────────────────────────────────────────────────

fn parse_swap(s: &str) -> Result<Instruction, ParseError> {
    let (lhs, rhs) = s.split_once("<->")
        .ok_or_else(|| ParseError::InvalidSyntax(format!("invalid swap: '{s}'")))?;

    let rx = match parse_operand(lhs.trim())? {
        Operand::Reg(r) => r,
        _ => return Err(ParseError::NonStrict(
            format!("swap LHS must be a plain register, got '{}'", lhs.trim()),
        )),
    };
    let ry = match parse_operand(rhs.trim())? {
        Operand::Reg(r) => r,
        _ => return Err(ParseError::NonStrict(
            format!("swap RHS must be a plain register, got '{}'", rhs.trim()),
        )),
    };

    // Encoded as `rx[ry] -> 0`
    Ok(Instruction::MvRg { src: (rx << 4) | ry, dst: 0, src_m: 1, dst_m: 1 })
}

fn parse_move(src_str: &str, dst_str: &str) -> Result<Instruction, ParseError> {
    let src = parse_operand(src_str)?;
    let dst = parse_operand(dst_str)?;

    // Guard: destination can never be a bare immediate
    if matches!(dst, Operand::Imm(_) | Operand::MemImm(_)) {
        return Err(ParseError::NonStrict(format!(
            "move destination cannot be an immediate value: '{dst_str}'"
        )));
    }

    match &src {
        Operand::Imm(val) => {
            // N -> r_rix  →  MvLd
            check_imm(*val as u64, 16)?;
            Ok(Instruction::MvLd { dst: dst.to_r_rix()?.0, val: *val })
        }
        Operand::MemImm(val) => {
            // @N -> r_rix  →  MvRd
            check_imm(*val as u64, 16)?;
            Ok(Instruction::MvRd { dst: dst.to_r_rix()?.0, val: *val })
        }
        _ => {
            // register form -> register form  →  MvRg
            let (src_rrix, src_m) = src.to_r_rix()?;
            let (dst_rrix, dst_m) = dst.to_r_rix()?;
            Ok(Instruction::MvRg {
                src: src_rrix,
                dst: dst_rrix,
                src_m: src_m as u8,
                dst_m: dst_m as u8,
            })
        }
    }
}

fn parse_binop_instr(lhs_str: &str, dst_str: &str) -> Result<Instruction, ParseError> {
    let (a_str, op_str, b_str) = split_binop_lhs(lhs_str)
        .ok_or_else(|| ParseError::InvalidSyntax(
            format!("cannot find operator in '{lhs_str}'"),
        ))?;

    let op  = parse_binop(op_str)?;
    let a   = parse_operand(a_str)?;
    let b   = parse_operand(b_str)?;
    let dst = parse_operand(dst_str)?;

    match &b {
        Operand::Imm(val) => {
            // Immediate mode — A and C must be plain registers
            check_imm(*val as u64, 16)?;
            Ok(Instruction::OpIm {
                op,
                lhs: a.to_reg4()?,
                rhs: *val,
                dst: dst.to_reg4()?,
            })
        }
        _ => {
            // Register mode — no multipliers permitted on any operand
            Ok(Instruction::OpRg {
                op,
                lhs: a.to_r_rix_no_mul()?,
                rhs: b.to_r_rix_no_mul()?,
                dst: dst.to_r_rix_no_mul()?,
            })
        }
    }
}

fn parse_data(s: &str) -> Result<Instruction, ParseError> {
    let rest = s["DAT ".len()..].trim();
    if let Some(val) = parse_int(rest) {
        Ok(Instruction::RawData(val as u32))
    } else { Err(ParseError::InvalidSyntax("invalid data syntax, expecting integer".to_string())) }
}

fn parse_branch_unconditional(s: &str) -> Result<Instruction, ParseError> {
    let rest = s["jmp ".len()..].trim();
    let target = parse_operand(rest.trim())?;
    
    match target {
        Operand::Imm(dst) => Ok(Instruction::BrUnconditional { dst }),
        _ => Ok(Instruction::BrCoRg { co: Condition::Eq, lhs: 0, rhs: 0, dst: target.to_r_rix_no_mul()? }),
    }
}

fn parse_branch(s: &str) -> Result<Instruction, ParseError> {
    // s begins with "if "
    let rest = s["if ".len()..].trim();

    // Optional `not` keyword — assembler negates the condition
    let (negate, rest) = match rest.strip_prefix("not ") {
        Some(r) => (true,  r.trim()),
        None    => (false, rest),
    };

    // Split on " jmp " to separate condition from target
    let (cond_part, target_str) = rest.split_once(" jmp ")
        .ok_or_else(|| ParseError::InvalidSyntax(
            format!("missing 'jmp' in branch: '{s}'"),
        ))?;

    let target = parse_operand(target_str.trim())?;

    let (a_str, op_str, b_str) = split_condition_lhs(cond_part.trim())
        .ok_or_else(|| ParseError::InvalidSyntax(
            format!("cannot parse condition '{}'", cond_part.trim()),
        ))?;

    let co = {
        let c = parse_condition(op_str)?;
        if negate { negate_condition(c) } else { c }
    };

    let a = parse_operand(a_str)?;
    let b = parse_operand(b_str)?;

    match &target {
        Operand::Imm(val) => {
            // Immediate mode — A and B must be plain registers
            check_imm(*val as u64, 16)?;
            Ok(Instruction::BrCoIm {
                co,
                lhs: a.to_reg4()?,
                rhs: b.to_reg4()?,
                val: *val,
            })
        }
        _ => {
            // Register mode — no multipliers
            Ok(Instruction::BrCoRg {
                co,
                lhs: a.to_r_rix_no_mul()?,
                rhs: b.to_r_rix_no_mul()?,
                dst: target.to_r_rix_no_mul()?,
            })
        }
    }
}

// ── Utility ───────────────────────────────────────────────────────────────────

fn check_imm(value: u64, bits: u32) -> Result<(), ParseError> {
    if value >= (1u64 << bits) {
        Err(ParseError::ImmediateOutOfRange { value, bits })
    } else {
        Ok(())
    }
}

// ── Top-level entry point ─────────────────────────────────────────────────────

pub fn parse_instruction(s: &str) -> Result<Instruction, ParseError> {
    let s = s.trim();

    if s.starts_with("DAT ") {
        return parse_data(s);
    }

    // ── Special keyword instructions ──────────────────────────────────────────
    match s {
        "HALT"  => return Ok(Instruction::SpHalt),
        "NOP"   => return Ok(Instruction::SpNop),
        "INT"   => return Ok(Instruction::SpInt),
        "VEC"   => return Ok(Instruction::SpVec),
        "WRITH" => return Ok(Instruction::SpWrith),
        "REATH" => return Ok(Instruction::SpReath),
        "INITH" => return Ok(Instruction::SpInith),
        "UMEM"  => return Ok(Instruction::SpUmem),
        "USER"  => return Ok(Instruction::SpUser),
        "DEBUG" => return Ok(Instruction::SpDebug),
        _ => {}
    }



    // ── Branch ────────────────────────────────────────────────────────────────
    if s.starts_with("jmp ") {
        return parse_branch_unconditional(s);
    }
    if s.starts_with("if ") {
        return parse_branch(s);
    }

    // ── Swap  (rx <-> ry) ─────────────────────────────────────────────────────
    if s.contains("<->") {
        return parse_swap(s);
    }

    // ── Move or BinOp  (both use `->`) ────────────────────────────────────────
    // rfind so the destination is always unambiguously after the *last* `->`.
    if let Some(arrow) = s.rfind("->") {
        let lhs = s[..arrow].trim();
        let dst = s[arrow + 2..].trim();

        return if split_binop_lhs(lhs).is_some() {
            parse_binop_instr(lhs, dst)
        } else {
            parse_move(lhs, dst)
        };
    }

    Err(ParseError::InvalidSyntax(format!("unrecognized instruction: '{s}'")))
}