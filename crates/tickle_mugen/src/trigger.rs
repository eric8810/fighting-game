use crate::error::SffError;

/// AST node for trigger expressions
#[derive(Debug, Clone, PartialEq)]
pub enum TriggerExpr {
    // Literals
    Int(i32),
    Float(f32),
    String(String),

    // Variables
    Variable(String),

    // Binary operations
    And(Box<TriggerExpr>, Box<TriggerExpr>),
    Or(Box<TriggerExpr>, Box<TriggerExpr>),
    Eq(Box<TriggerExpr>, Box<TriggerExpr>),
    Ne(Box<TriggerExpr>, Box<TriggerExpr>),
    Lt(Box<TriggerExpr>, Box<TriggerExpr>),
    Gt(Box<TriggerExpr>, Box<TriggerExpr>),
    Le(Box<TriggerExpr>, Box<TriggerExpr>),
    Ge(Box<TriggerExpr>, Box<TriggerExpr>),
    Add(Box<TriggerExpr>, Box<TriggerExpr>),
    Sub(Box<TriggerExpr>, Box<TriggerExpr>),
    Mul(Box<TriggerExpr>, Box<TriggerExpr>),
    Div(Box<TriggerExpr>, Box<TriggerExpr>),
    Mod(Box<TriggerExpr>, Box<TriggerExpr>),

    // Unary operations
    Not(Box<TriggerExpr>),
    Neg(Box<TriggerExpr>),

    // Function calls
    FunctionCall(String, Vec<TriggerExpr>),
}

/// A runtime value produced by evaluating a TriggerExpr.
#[derive(Debug, Clone, PartialEq)]
pub enum TriggerValue {
    Int(i32),
    Float(f32),
    Bool(bool),
}

/// All variables that trigger expressions can read at evaluation time.
#[derive(Debug, Clone)]
pub struct TriggerContext {
    /// Ticks elapsed in the current state (state_frame)
    pub time: i32,
    /// Current state number
    pub stateno: i32,
    /// Previous state number
    pub prev_stateno: i32,
    /// Current state type: 0=Standing(S), 1=Crouching(C), 2=Aerial(A), 3=Lying(L)
    pub statetype: i32,
    /// Velocity X in logic units (divide by 100 for pixels)
    pub vel_x: i32,
    /// Velocity Y in logic units
    pub vel_y: i32,
    /// Y position (0 = ground level)
    pub pos_y: i32,
    /// Control flag – whether the character can accept new commands
    pub ctrl: bool,
    /// Integer variables var(0)..var(59)
    pub vars: [i32; 60],
    /// Current animation number
    pub anim_num: i32,
    /// Current animation element (1-based)
    pub anim_elem: i32,
    /// Ticks into the current animation element
    pub anim_time: i32,
    /// Whether the last attack connected
    pub move_hit: bool,
    /// Whether the last attack made contact (hit or guarded)
    pub move_contact: bool,
    /// Whether the last attack was guarded
    pub move_guarded: bool,
    /// Command names that are active this frame
    pub active_commands: Vec<String>,
    /// Current life (HP)
    pub life: i32,
    /// Current power gauge value
    pub power: i32,

    // ── P2 (opponent) state ─────────────────────────────────────────────
    /// Opponent's current state number
    pub p2_stateno: i32,
    /// Opponent's current life (HP)
    pub p2_life: i32,
    /// Horizontal distance to opponent (positive = opponent is in front)
    pub p2_bodydist_x: i32,
    /// Vertical distance to opponent
    pub p2_bodydist_y: i32,
    /// Opponent's state type: 0=Standing, 1=Crouching, 2=Aerial, 3=Lying
    pub p2_statetype: i32,
    /// Opponent's move type: 0=Idle, 1=Attack, 2=BeingHit
    pub p2_movetype: i32,
    /// Opponent's control flag
    pub p2_ctrl: bool,
    /// Opponent's velocity X
    pub p2_vel_x: i32,
    /// Opponent's velocity Y
    pub p2_vel_y: i32,
}

impl Default for TriggerContext {
    fn default() -> Self {
        Self {
            time: 0,
            stateno: 0,
            prev_stateno: 0,
            statetype: 0,
            vel_x: 0,
            vel_y: 0,
            pos_y: 0,
            ctrl: false,
            vars: [0i32; 60],
            anim_num: 0,
            anim_elem: 0,
            anim_time: 0,
            move_hit: false,
            move_contact: false,
            move_guarded: false,
            active_commands: Vec::new(),
            life: 0,
            power: 0,
            p2_stateno: 0,
            p2_life: 0,
            p2_bodydist_x: 0,
            p2_bodydist_y: 0,
            p2_statetype: 0,
            p2_movetype: 0,
            p2_ctrl: false,
            p2_vel_x: 0,
            p2_vel_y: 0,
        }
    }
}

// ─── helpers ─────────────────────────────────────────────────────────────────

fn to_float(v: &TriggerValue) -> f32 {
    match v {
        TriggerValue::Int(n) => *n as f32,
        TriggerValue::Float(f) => *f,
        TriggerValue::Bool(b) => if *b { 1.0 } else { 0.0 },
    }
}

fn to_int(v: &TriggerValue) -> i32 {
    match v {
        TriggerValue::Int(n) => *n,
        TriggerValue::Float(f) => *f as i32,
        TriggerValue::Bool(b) => if *b { 1 } else { 0 },
    }
}

fn is_truthy(v: &TriggerValue) -> bool {
    match v {
        TriggerValue::Bool(b) => *b,
        TriggerValue::Int(n) => *n != 0,
        TriggerValue::Float(f) => *f != 0.0,
    }
}

/// Promote two values to a common numeric type (Float wins).
fn coerce_numeric(a: &TriggerValue, b: &TriggerValue) -> (f32, f32, bool) {
    let a_is_float = matches!(a, TriggerValue::Float(_));
    let b_is_float = matches!(b, TriggerValue::Float(_));
    (to_float(a), to_float(b), a_is_float || b_is_float)
}

// ─── evaluate ────────────────────────────────────────────────────────────────

impl TriggerExpr {
    /// Evaluate this expression to a TriggerValue given the current game context.
    pub fn evaluate(&self, ctx: &TriggerContext) -> TriggerValue {
        match self {
            // ── Literals ──────────────────────────────────────────────────
            TriggerExpr::Int(n) => TriggerValue::Int(*n),
            TriggerExpr::Float(f) => TriggerValue::Float(*f),
            TriggerExpr::String(_) => TriggerValue::Int(0), // strings are resolved by callers

            // ── Variables ─────────────────────────────────────────────────
            TriggerExpr::Variable(name) => {
                let lower = name.to_lowercase();
                match lower.as_str() {
                    "time" | "animtime" if lower == "animtime" => TriggerValue::Int(ctx.anim_time),
                    "time" => TriggerValue::Int(ctx.time),
                    "stateno" => TriggerValue::Int(ctx.stateno),
                    "prevstateno" => TriggerValue::Int(ctx.prev_stateno),
                    "statetype" => TriggerValue::Int(ctx.statetype),
                    // State type constants: S=0, C=1, A=2, L=3 (used in "statetype = S" comparisons)
                    "s" => TriggerValue::Int(0),
                    "c" => TriggerValue::Int(1),
                    "a" => TriggerValue::Int(2),
                    "l" => TriggerValue::Int(3),
                    "vel x" | "velx" => TriggerValue::Int(ctx.vel_x),
                    "vel y" | "vely" => TriggerValue::Int(ctx.vel_y),
                    "pos y" | "posy" => TriggerValue::Int(ctx.pos_y),
                    "ctrl" => TriggerValue::Int(if ctx.ctrl { 1 } else { 0 }),
                    "movehit" => TriggerValue::Int(if ctx.move_hit { 1 } else { 0 }),
                    "movecontact" => TriggerValue::Int(if ctx.move_contact { 1 } else { 0 }),
                    "moveguarded" => TriggerValue::Int(if ctx.move_guarded { 1 } else { 0 }),
                    "anim" => TriggerValue::Int(ctx.anim_num),
                    "animelem" => TriggerValue::Int(ctx.anim_elem),
                    "life" => TriggerValue::Int(ctx.life),
                    "power" => TriggerValue::Int(ctx.power),
                    "random" => TriggerValue::Int(0), // placeholder – real impl needs seeded RNG
                    // P2 (opponent) variables
                    "p2stateno" => TriggerValue::Int(ctx.p2_stateno),
                    "p2life" => TriggerValue::Int(ctx.p2_life),
                    "p2bodydist x" | "p2bodydistx" => TriggerValue::Int(ctx.p2_bodydist_x),
                    "p2bodydist y" | "p2bodydisty" => TriggerValue::Int(ctx.p2_bodydist_y),
                    "p2statetype" => TriggerValue::Int(ctx.p2_statetype),
                    "p2movetype" => TriggerValue::Int(ctx.p2_movetype),
                    "p2ctrl" => TriggerValue::Int(if ctx.p2_ctrl { 1 } else { 0 }),
                    "p2vel x" | "p2velx" => TriggerValue::Int(ctx.p2_vel_x),
                    "p2vel y" | "p2vely" => TriggerValue::Int(ctx.p2_vel_y),
                    _ => TriggerValue::Int(0),        // unknown variable
                }
            }

            // ── Unary ──────────────────────────────────────────────────────
            TriggerExpr::Not(inner) => {
                let val = inner.evaluate(ctx);
                TriggerValue::Bool(!is_truthy(&val))
            }
            TriggerExpr::Neg(inner) => {
                let val = inner.evaluate(ctx);
                match val {
                    TriggerValue::Int(n) => TriggerValue::Int(-n),
                    TriggerValue::Float(f) => TriggerValue::Float(-f),
                    TriggerValue::Bool(b) => TriggerValue::Int(if b { -1 } else { 0 }),
                }
            }

            // ── Logical AND / OR ──────────────────────────────────────────
            TriggerExpr::And(left, right) => {
                let lv = left.evaluate(ctx);
                TriggerValue::Bool(is_truthy(&lv) && is_truthy(&right.evaluate(ctx)))
            }
            TriggerExpr::Or(left, right) => {
                let lv = left.evaluate(ctx);
                TriggerValue::Bool(is_truthy(&lv) || is_truthy(&right.evaluate(ctx)))
            }

            // ── Comparisons ────────────────────────────────────────────────
            TriggerExpr::Eq(left, right) => {
                let lv = left.evaluate(ctx);
                let rv = right.evaluate(ctx);
                // String equality check (e.g. command = "holdfwd")
                if let (TriggerExpr::Variable(var_name), TriggerExpr::String(cmd)) =
                    (left.as_ref(), right.as_ref())
                {
                    let lower = var_name.to_lowercase();
                    if lower == "command" {
                        return TriggerValue::Bool(ctx.active_commands.iter().any(|c| c == cmd));
                    }
                }
                let (la, ra, is_float) = coerce_numeric(&lv, &rv);
                if is_float {
                    TriggerValue::Bool((la - ra).abs() < f32::EPSILON)
                } else {
                    TriggerValue::Bool(to_int(&lv) == to_int(&rv))
                }
            }
            TriggerExpr::Ne(left, right) => {
                let lv = left.evaluate(ctx);
                let rv = right.evaluate(ctx);
                // String inequality check (e.g. command != "holdup")
                if let (TriggerExpr::Variable(var_name), TriggerExpr::String(cmd)) =
                    (left.as_ref(), right.as_ref())
                {
                    let lower = var_name.to_lowercase();
                    if lower == "command" {
                        return TriggerValue::Bool(!ctx.active_commands.iter().any(|c| c == cmd));
                    }
                }
                let (la, ra, is_float) = coerce_numeric(&lv, &rv);
                if is_float {
                    TriggerValue::Bool((la - ra).abs() >= f32::EPSILON)
                } else {
                    TriggerValue::Bool(to_int(&lv) != to_int(&rv))
                }
            }
            TriggerExpr::Lt(left, right) => {
                let (la, ra, _) = coerce_numeric(&left.evaluate(ctx), &right.evaluate(ctx));
                TriggerValue::Bool(la < ra)
            }
            TriggerExpr::Gt(left, right) => {
                let (la, ra, _) = coerce_numeric(&left.evaluate(ctx), &right.evaluate(ctx));
                TriggerValue::Bool(la > ra)
            }
            TriggerExpr::Le(left, right) => {
                let (la, ra, _) = coerce_numeric(&left.evaluate(ctx), &right.evaluate(ctx));
                TriggerValue::Bool(la <= ra)
            }
            TriggerExpr::Ge(left, right) => {
                let (la, ra, _) = coerce_numeric(&left.evaluate(ctx), &right.evaluate(ctx));
                TriggerValue::Bool(la >= ra)
            }

            // ── Arithmetic ─────────────────────────────────────────────────
            TriggerExpr::Add(left, right) => {
                let lv = left.evaluate(ctx);
                let rv = right.evaluate(ctx);
                let (la, ra, is_float) = coerce_numeric(&lv, &rv);
                if is_float {
                    TriggerValue::Float(la + ra)
                } else {
                    TriggerValue::Int(to_int(&lv) + to_int(&rv))
                }
            }
            TriggerExpr::Sub(left, right) => {
                let lv = left.evaluate(ctx);
                let rv = right.evaluate(ctx);
                let (la, ra, is_float) = coerce_numeric(&lv, &rv);
                if is_float {
                    TriggerValue::Float(la - ra)
                } else {
                    TriggerValue::Int(to_int(&lv) - to_int(&rv))
                }
            }
            TriggerExpr::Mul(left, right) => {
                let lv = left.evaluate(ctx);
                let rv = right.evaluate(ctx);
                let (la, ra, is_float) = coerce_numeric(&lv, &rv);
                if is_float {
                    TriggerValue::Float(la * ra)
                } else {
                    TriggerValue::Int(to_int(&lv) * to_int(&rv))
                }
            }
            TriggerExpr::Div(left, right) => {
                let lv = left.evaluate(ctx);
                let rv = right.evaluate(ctx);
                let (la, ra, is_float) = coerce_numeric(&lv, &rv);
                if is_float {
                    if ra == 0.0 { TriggerValue::Float(0.0) } else { TriggerValue::Float(la / ra) }
                } else {
                    let ri = to_int(&rv);
                    if ri == 0 { TriggerValue::Int(0) } else { TriggerValue::Int(to_int(&lv) / ri) }
                }
            }
            TriggerExpr::Mod(left, right) => {
                let lv = left.evaluate(ctx);
                let rv = right.evaluate(ctx);
                let (la, ra, is_float) = coerce_numeric(&lv, &rv);
                if is_float {
                    if ra == 0.0 { TriggerValue::Float(0.0) } else { TriggerValue::Float(la % ra) }
                } else {
                    let ri = to_int(&rv);
                    if ri == 0 { TriggerValue::Int(0) } else { TriggerValue::Int(to_int(&lv) % ri) }
                }
            }

            // ── Function calls ─────────────────────────────────────────────
            TriggerExpr::FunctionCall(name, args) => {
                let lower = name.to_lowercase();
                match lower.as_str() {
                    "var" => {
                        if let Some(first) = args.first() {
                            let idx = to_int(&first.evaluate(ctx)) as usize;
                            if idx < ctx.vars.len() {
                                return TriggerValue::Int(ctx.vars[idx]);
                            }
                        }
                        TriggerValue::Int(0)
                    }
                    "ifelse" => {
                        if args.len() >= 3 {
                            let cond = args[0].evaluate(ctx);
                            if is_truthy(&cond) {
                                args[1].evaluate(ctx)
                            } else {
                                args[2].evaluate(ctx)
                            }
                        } else {
                            TriggerValue::Int(0)
                        }
                    }
                    "floor" => {
                        if let Some(first) = args.first() {
                            let v = first.evaluate(ctx);
                            TriggerValue::Int(to_float(&v).floor() as i32)
                        } else {
                            TriggerValue::Int(0)
                        }
                    }
                    "ceil" => {
                        if let Some(first) = args.first() {
                            let v = first.evaluate(ctx);
                            TriggerValue::Int(to_float(&v).ceil() as i32)
                        } else {
                            TriggerValue::Int(0)
                        }
                    }
                    "abs" => {
                        if let Some(first) = args.first() {
                            let v = first.evaluate(ctx);
                            match v {
                                TriggerValue::Int(n) => TriggerValue::Int(n.abs()),
                                TriggerValue::Float(f) => TriggerValue::Float(f.abs()),
                                TriggerValue::Bool(b) => TriggerValue::Int(if b { 1 } else { 0 }),
                            }
                        } else {
                            TriggerValue::Int(0)
                        }
                    }
                    "command" => {
                        // command("name") form
                        if let Some(first) = args.first() {
                            if let TriggerExpr::String(cmd) = first {
                                return TriggerValue::Bool(ctx.active_commands.iter().any(|c| c == cmd));
                            }
                            // Not a string literal; fall back to false.
                            TriggerValue::Bool(false)
                        } else {
                            TriggerValue::Bool(false)
                        }
                    }
                    // const() is used for character constants – return 0 as placeholder
                    "const" => TriggerValue::Int(0),
                    // Unknown functions return 0
                    _ => TriggerValue::Int(0),
                }
            }
        }
    }

    /// Convenience: evaluate and convert to bool.
    /// Bool(b) → b, Int(n) → n != 0, Float(f) → f != 0.0
    pub fn evaluate_bool(&self, ctx: &TriggerContext) -> bool {
        is_truthy(&self.evaluate(ctx))
    }

    /// Convenience: evaluate and convert to i32.
    pub fn evaluate_int(&self, ctx: &TriggerContext) -> i32 {
        to_int(&self.evaluate(ctx))
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Token / parser (unchanged)
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
enum Token {
    // Literals
    Number(f32),
    String(String),
    Identifier(String),

    // Operators
    And,      // &&
    Or,       // ||
    Not,      // !
    Eq,       // =
    Ne,       // !=
    Lt,       // <
    Gt,       // >
    Le,       // <=
    Ge,       // >=
    Plus,     // +
    Minus,    // -
    Star,     // *
    Slash,    // /
    Percent,  // %

    // Delimiters
    LParen,   // (
    RParen,   // )
    Comma,    // ,
}

pub struct TriggerParser;

impl TriggerParser {
    pub fn parse(input: &str) -> Result<TriggerExpr, SffError> {
        let tokens = Self::tokenize(input)?;
        Self::parse_expr(&tokens, 0).map(|(expr, _)| expr)
    }

    fn tokenize(input: &str) -> Result<Vec<Token>, SffError> {
        let mut tokens = Vec::new();
        let mut chars = input.chars().peekable();

        while let Some(&ch) = chars.peek() {
            match ch {
                ' ' | '\t' | '\r' | '\n' => {
                    chars.next();
                }
                '&' => {
                    chars.next();
                    if chars.peek() == Some(&'&') {
                        chars.next();
                        tokens.push(Token::And);
                    } else {
                        return Err(SffError::DefParse("Expected '&&'".to_string()));
                    }
                }
                '|' => {
                    chars.next();
                    if chars.peek() == Some(&'|') {
                        chars.next();
                        tokens.push(Token::Or);
                    } else {
                        return Err(SffError::DefParse("Expected '||'".to_string()));
                    }
                }
                '!' => {
                    chars.next();
                    if chars.peek() == Some(&'=') {
                        chars.next();
                        tokens.push(Token::Ne);
                    } else {
                        tokens.push(Token::Not);
                    }
                }
                '=' => {
                    chars.next();
                    tokens.push(Token::Eq);
                }
                '<' => {
                    chars.next();
                    if chars.peek() == Some(&'=') {
                        chars.next();
                        tokens.push(Token::Le);
                    } else {
                        tokens.push(Token::Lt);
                    }
                }
                '>' => {
                    chars.next();
                    if chars.peek() == Some(&'=') {
                        chars.next();
                        tokens.push(Token::Ge);
                    } else {
                        tokens.push(Token::Gt);
                    }
                }
                '+' => {
                    chars.next();
                    tokens.push(Token::Plus);
                }
                '-' => {
                    chars.next();
                    // Check if it's a negative number
                    if chars.peek().map(|c| c.is_ascii_digit()).unwrap_or(false) {
                        let num = Self::read_number(&mut chars)?;
                        tokens.push(Token::Number(-num));
                    } else {
                        tokens.push(Token::Minus);
                    }
                }
                '*' => {
                    chars.next();
                    tokens.push(Token::Star);
                }
                '/' => {
                    chars.next();
                    tokens.push(Token::Slash);
                }
                '%' => {
                    chars.next();
                    tokens.push(Token::Percent);
                }
                '(' => {
                    chars.next();
                    tokens.push(Token::LParen);
                }
                ')' => {
                    chars.next();
                    tokens.push(Token::RParen);
                }
                ',' => {
                    chars.next();
                    tokens.push(Token::Comma);
                }
                '"' => {
                    chars.next();
                    let s = Self::read_string(&mut chars)?;
                    tokens.push(Token::String(s));
                }
                _ if ch.is_ascii_digit() => {
                    let num = Self::read_number(&mut chars)?;
                    tokens.push(Token::Number(num));
                }
                _ if ch.is_alphabetic() || ch == '_' => {
                    let ident = Self::read_identifier(&mut chars);
                    tokens.push(Token::Identifier(ident));
                }
                _ => {
                    return Err(SffError::DefParse(format!("Unexpected character: {}", ch)));
                }
            }
        }

        Ok(tokens)
    }

    fn read_number(chars: &mut std::iter::Peekable<std::str::Chars>) -> Result<f32, SffError> {
        let mut num_str = String::new();
        let mut has_dot = false;

        while let Some(&ch) = chars.peek() {
            if ch.is_ascii_digit() {
                num_str.push(ch);
                chars.next();
            } else if ch == '.' && !has_dot {
                has_dot = true;
                num_str.push(ch);
                chars.next();
            } else {
                break;
            }
        }

        num_str.parse::<f32>()
            .map_err(|_| SffError::DefParse(format!("Invalid number: {}", num_str)))
    }

    fn read_string(chars: &mut std::iter::Peekable<std::str::Chars>) -> Result<String, SffError> {
        let mut s = String::new();

        while let Some(&ch) = chars.peek() {
            if ch == '"' {
                chars.next();
                return Ok(s);
            } else if ch == '\\' {
                chars.next();
                if let Some(&escaped) = chars.peek() {
                    chars.next();
                    match escaped {
                        'n' => s.push('\n'),
                        't' => s.push('\t'),
                        'r' => s.push('\r'),
                        '"' => s.push('"'),
                        '\\' => s.push('\\'),
                        _ => {
                            s.push('\\');
                            s.push(escaped);
                        }
                    }
                }
            } else {
                s.push(ch);
                chars.next();
            }
        }

        Err(SffError::DefParse("Unterminated string".to_string()))
    }

    fn read_identifier(chars: &mut std::iter::Peekable<std::str::Chars>) -> String {
        let mut ident = String::new();

        while let Some(&ch) = chars.peek() {
            if ch.is_alphanumeric() || ch == '_' {
                ident.push(ch);
                chars.next();
            } else {
                break;
            }
        }

        ident
    }

    // Parse expression with operator precedence
    // Returns (expression, next_position)
    fn parse_expr(tokens: &[Token], pos: usize) -> Result<(TriggerExpr, usize), SffError> {
        Self::parse_or(tokens, pos)
    }

    // Logical OR (lowest precedence)
    fn parse_or(tokens: &[Token], mut pos: usize) -> Result<(TriggerExpr, usize), SffError> {
        let (mut left, new_pos) = Self::parse_and(tokens, pos)?;
        pos = new_pos;

        while pos < tokens.len() {
            if matches!(tokens[pos], Token::Or) {
                pos += 1;
                let (right, new_pos) = Self::parse_and(tokens, pos)?;
                left = TriggerExpr::Or(Box::new(left), Box::new(right));
                pos = new_pos;
            } else {
                break;
            }
        }

        Ok((left, pos))
    }

    // Logical AND
    fn parse_and(tokens: &[Token], mut pos: usize) -> Result<(TriggerExpr, usize), SffError> {
        let (mut left, new_pos) = Self::parse_comparison(tokens, pos)?;
        pos = new_pos;

        while pos < tokens.len() {
            if matches!(tokens[pos], Token::And) {
                pos += 1;
                let (right, new_pos) = Self::parse_comparison(tokens, pos)?;
                left = TriggerExpr::And(Box::new(left), Box::new(right));
                pos = new_pos;
            } else {
                break;
            }
        }

        Ok((left, pos))
    }

    // Comparison operators
    fn parse_comparison(tokens: &[Token], mut pos: usize) -> Result<(TriggerExpr, usize), SffError> {
        let (mut left, new_pos) = Self::parse_additive(tokens, pos)?;
        pos = new_pos;

        while pos < tokens.len() {
            let op = &tokens[pos];
            match op {
                Token::Eq => {
                    pos += 1;
                    let (right, new_pos) = Self::parse_additive(tokens, pos)?;
                    left = TriggerExpr::Eq(Box::new(left), Box::new(right));
                    pos = new_pos;
                }
                Token::Ne => {
                    pos += 1;
                    let (right, new_pos) = Self::parse_additive(tokens, pos)?;
                    left = TriggerExpr::Ne(Box::new(left), Box::new(right));
                    pos = new_pos;
                }
                Token::Lt => {
                    pos += 1;
                    let (right, new_pos) = Self::parse_additive(tokens, pos)?;
                    left = TriggerExpr::Lt(Box::new(left), Box::new(right));
                    pos = new_pos;
                }
                Token::Gt => {
                    pos += 1;
                    let (right, new_pos) = Self::parse_additive(tokens, pos)?;
                    left = TriggerExpr::Gt(Box::new(left), Box::new(right));
                    pos = new_pos;
                }
                Token::Le => {
                    pos += 1;
                    let (right, new_pos) = Self::parse_additive(tokens, pos)?;
                    left = TriggerExpr::Le(Box::new(left), Box::new(right));
                    pos = new_pos;
                }
                Token::Ge => {
                    pos += 1;
                    let (right, new_pos) = Self::parse_additive(tokens, pos)?;
                    left = TriggerExpr::Ge(Box::new(left), Box::new(right));
                    pos = new_pos;
                }
                _ => break,
            }
        }

        Ok((left, pos))
    }

    // Addition and subtraction
    fn parse_additive(tokens: &[Token], mut pos: usize) -> Result<(TriggerExpr, usize), SffError> {
        let (mut left, new_pos) = Self::parse_multiplicative(tokens, pos)?;
        pos = new_pos;

        while pos < tokens.len() {
            match &tokens[pos] {
                Token::Plus => {
                    pos += 1;
                    let (right, new_pos) = Self::parse_multiplicative(tokens, pos)?;
                    left = TriggerExpr::Add(Box::new(left), Box::new(right));
                    pos = new_pos;
                }
                Token::Minus => {
                    pos += 1;
                    let (right, new_pos) = Self::parse_multiplicative(tokens, pos)?;
                    left = TriggerExpr::Sub(Box::new(left), Box::new(right));
                    pos = new_pos;
                }
                _ => break,
            }
        }

        Ok((left, pos))
    }

    // Multiplication, division, modulo
    fn parse_multiplicative(tokens: &[Token], mut pos: usize) -> Result<(TriggerExpr, usize), SffError> {
        let (mut left, new_pos) = Self::parse_unary(tokens, pos)?;
        pos = new_pos;

        while pos < tokens.len() {
            match &tokens[pos] {
                Token::Star => {
                    pos += 1;
                    let (right, new_pos) = Self::parse_unary(tokens, pos)?;
                    left = TriggerExpr::Mul(Box::new(left), Box::new(right));
                    pos = new_pos;
                }
                Token::Slash => {
                    pos += 1;
                    let (right, new_pos) = Self::parse_unary(tokens, pos)?;
                    left = TriggerExpr::Div(Box::new(left), Box::new(right));
                    pos = new_pos;
                }
                Token::Percent => {
                    pos += 1;
                    let (right, new_pos) = Self::parse_unary(tokens, pos)?;
                    left = TriggerExpr::Mod(Box::new(left), Box::new(right));
                    pos = new_pos;
                }
                _ => break,
            }
        }

        Ok((left, pos))
    }

    // Unary operators
    fn parse_unary(tokens: &[Token], pos: usize) -> Result<(TriggerExpr, usize), SffError> {
        if pos >= tokens.len() {
            return Err(SffError::DefParse("Unexpected end of expression".to_string()));
        }

        match &tokens[pos] {
            Token::Not => {
                let (expr, new_pos) = Self::parse_unary(tokens, pos + 1)?;
                Ok((TriggerExpr::Not(Box::new(expr)), new_pos))
            }
            Token::Minus => {
                let (expr, new_pos) = Self::parse_unary(tokens, pos + 1)?;
                Ok((TriggerExpr::Neg(Box::new(expr)), new_pos))
            }
            _ => Self::parse_primary(tokens, pos),
        }
    }

    // Primary expressions (literals, variables, function calls, parenthesized expressions)
    fn parse_primary(tokens: &[Token], pos: usize) -> Result<(TriggerExpr, usize), SffError> {
        if pos >= tokens.len() {
            return Err(SffError::DefParse("Unexpected end of expression".to_string()));
        }

        match &tokens[pos] {
            Token::Number(n) => {
                // Check if it's an integer or float
                if n.fract() == 0.0 {
                    Ok((TriggerExpr::Int(*n as i32), pos + 1))
                } else {
                    Ok((TriggerExpr::Float(*n), pos + 1))
                }
            }
            Token::String(s) => {
                Ok((TriggerExpr::String(s.clone()), pos + 1))
            }
            Token::Identifier(name) => {
                // Check if it's a function call
                if pos + 1 < tokens.len() && matches!(tokens[pos + 1], Token::LParen) {
                    Self::parse_function_call(name.clone(), tokens, pos + 2)
                } else {
                    // Check if it's a multi-word variable (e.g., "Vel X", "Pos Y")
                    let mut var_name = name.clone();
                    let mut new_pos = pos + 1;

                    // Combine consecutive identifiers into a single variable name
                    while new_pos < tokens.len() {
                        if let Token::Identifier(next_name) = &tokens[new_pos] {
                            var_name.push(' ');
                            var_name.push_str(next_name);
                            new_pos += 1;
                        } else {
                            break;
                        }
                    }

                    Ok((TriggerExpr::Variable(var_name), new_pos))
                }
            }
            Token::LParen => {
                // Parenthesized expression
                let (expr, new_pos) = Self::parse_expr(tokens, pos + 1)?;
                if new_pos >= tokens.len() || !matches!(tokens[new_pos], Token::RParen) {
                    return Err(SffError::DefParse("Expected ')'".to_string()));
                }
                Ok((expr, new_pos + 1))
            }
            _ => Err(SffError::DefParse(format!("Unexpected token: {:?}", tokens[pos]))),
        }
    }

    // Parse function call arguments
    fn parse_function_call(name: String, tokens: &[Token], mut pos: usize) -> Result<(TriggerExpr, usize), SffError> {
        let mut args = Vec::new();

        // Check for empty argument list
        if pos < tokens.len() && matches!(tokens[pos], Token::RParen) {
            return Ok((TriggerExpr::FunctionCall(name, args), pos + 1));
        }

        // Parse arguments
        loop {
            let (arg, new_pos) = Self::parse_expr(tokens, pos)?;
            args.push(arg);
            pos = new_pos;

            if pos >= tokens.len() {
                return Err(SffError::DefParse("Expected ')' or ','".to_string()));
            }

            match &tokens[pos] {
                Token::Comma => {
                    pos += 1;
                }
                Token::RParen => {
                    return Ok((TriggerExpr::FunctionCall(name, args), pos + 1));
                }
                _ => {
                    return Err(SffError::DefParse(format!("Expected ')' or ',', got {:?}", tokens[pos])));
                }
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_ctx() -> TriggerContext {
        TriggerContext::default()
    }

    #[test]
    fn test_evaluate_time_ge_10_true() {
        // Time >= 10 evaluates to true when time = 15
        let expr = TriggerParser::parse("Time >= 10").unwrap();
        let mut ctx = make_ctx();
        ctx.time = 15;
        assert!(expr.evaluate_bool(&ctx));
    }

    #[test]
    fn test_evaluate_time_ge_10_false() {
        let expr = TriggerParser::parse("Time >= 10").unwrap();
        let mut ctx = make_ctx();
        ctx.time = 5;
        assert!(!expr.evaluate_bool(&ctx));
    }

    #[test]
    fn test_evaluate_var_eq() {
        // var(0) = 1 evaluates to true when vars[0] = 1
        let expr = TriggerParser::parse("var(0) = 1").unwrap();
        let mut ctx = make_ctx();
        ctx.vars[0] = 1;
        assert!(expr.evaluate_bool(&ctx));
    }

    #[test]
    fn test_evaluate_var_eq_false() {
        let expr = TriggerParser::parse("var(0) = 1").unwrap();
        let mut ctx = make_ctx();
        ctx.vars[0] = 0;
        assert!(!expr.evaluate_bool(&ctx));
    }

    #[test]
    fn test_evaluate_ifelse_ctrl_true() {
        // ifelse(ctrl, 20, 21) returns 20 when ctrl = true
        let expr = TriggerParser::parse("ifelse(ctrl, 20, 21)").unwrap();
        let mut ctx = make_ctx();
        ctx.ctrl = true;
        assert_eq!(expr.evaluate_int(&ctx), 20);
    }

    #[test]
    fn test_evaluate_ifelse_ctrl_false() {
        let expr = TriggerParser::parse("ifelse(ctrl, 20, 21)").unwrap();
        let mut ctx = make_ctx();
        ctx.ctrl = false;
        assert_eq!(expr.evaluate_int(&ctx), 21);
    }

    #[test]
    fn test_evaluate_command_active() {
        // command("QCF_a") returns true when "QCF_a" is in active_commands
        // The parser produces: Eq(Variable("command"), String("QCF_a"))
        let expr = TriggerParser::parse("command = \"QCF_a\"").unwrap();
        let mut ctx = make_ctx();
        ctx.active_commands = vec!["QCF_a".to_string()];
        assert!(expr.evaluate_bool(&ctx));
    }

    #[test]
    fn test_evaluate_command_inactive() {
        let expr = TriggerParser::parse("command = \"QCF_a\"").unwrap();
        let ctx = make_ctx();
        assert!(!expr.evaluate_bool(&ctx));
    }

    #[test]
    fn test_evaluate_command_ne() {
        // command != "holdup" is true when "holdup" is NOT active
        let expr = TriggerParser::parse("command != \"holdup\"").unwrap();
        let mut ctx = make_ctx();
        ctx.active_commands = vec!["holdfwd".to_string()];
        assert!(expr.evaluate_bool(&ctx));
    }

    #[test]
    fn test_evaluate_not() {
        let expr = TriggerParser::parse("!ctrl").unwrap();
        let mut ctx = make_ctx();
        ctx.ctrl = false;
        assert!(expr.evaluate_bool(&ctx));
        ctx.ctrl = true;
        assert!(!expr.evaluate_bool(&ctx));
    }

    #[test]
    fn test_evaluate_arithmetic() {
        // 3 + 4 * 2 = 11  (multiply first)
        let expr = TriggerParser::parse("3 + 4 * 2").unwrap();
        let ctx = make_ctx();
        assert_eq!(expr.evaluate_int(&ctx), 11);
    }

    #[test]
    fn test_evaluate_and() {
        let expr = TriggerParser::parse("1 && 0").unwrap();
        let ctx = make_ctx();
        assert!(!expr.evaluate_bool(&ctx));

        let expr2 = TriggerParser::parse("1 && 1").unwrap();
        assert!(expr2.evaluate_bool(&ctx));
    }

    #[test]
    fn test_evaluate_or() {
        let expr = TriggerParser::parse("0 || 1").unwrap();
        let ctx = make_ctx();
        assert!(expr.evaluate_bool(&ctx));

        let expr2 = TriggerParser::parse("0 || 0").unwrap();
        assert!(!expr2.evaluate_bool(&ctx));
    }

    #[test]
    fn test_evaluate_stateno() {
        let expr = TriggerParser::parse("stateno = 200").unwrap();
        let mut ctx = make_ctx();
        ctx.stateno = 200;
        assert!(expr.evaluate_bool(&ctx));
    }

    #[test]
    fn test_evaluate_velx() {
        let expr = TriggerParser::parse("Vel X > 0").unwrap();
        let mut ctx = make_ctx();
        ctx.vel_x = 100;
        assert!(expr.evaluate_bool(&ctx));
    }

    #[test]
    fn test_evaluate_abs() {
        let expr = TriggerParser::parse("abs(-5)").unwrap();
        let ctx = make_ctx();
        assert_eq!(expr.evaluate_int(&ctx), 5);
    }

    #[test]
    fn test_evaluate_floor() {
        // floor(3.7) = 3
        let expr = TriggerParser::parse("floor(3.7)").unwrap();
        let ctx = make_ctx();
        assert_eq!(expr.evaluate_int(&ctx), 3);
    }

    #[test]
    fn test_evaluate_ceil() {
        // ceil(3.2) = 4
        let expr = TriggerParser::parse("ceil(3.2)").unwrap();
        let ctx = make_ctx();
        assert_eq!(expr.evaluate_int(&ctx), 4);
    }

    #[test]
    fn test_literal_one_is_truthy() {
        // trigger1 = 1 means always true
        let expr = TriggerParser::parse("1").unwrap();
        let ctx = make_ctx();
        assert!(expr.evaluate_bool(&ctx));
    }

    #[test]
    fn test_p2_stateno() {
        let expr = TriggerParser::parse("P2StateNo = 5000").unwrap();
        let mut ctx = make_ctx();
        ctx.p2_stateno = 5000;
        assert!(expr.evaluate_bool(&ctx));

        ctx.p2_stateno = 0;
        assert!(!expr.evaluate_bool(&ctx));
    }

    #[test]
    fn test_p2_life() {
        let expr = TriggerParser::parse("P2Life < 900").unwrap();
        let mut ctx = make_ctx();
        ctx.p2_life = 800;
        assert!(expr.evaluate_bool(&ctx));

        ctx.p2_life = 1000;
        assert!(!expr.evaluate_bool(&ctx));
    }

    #[test]
    fn test_p2_bodydist_x() {
        let expr = TriggerParser::parse("P2BodyDist X < 50").unwrap();
        let mut ctx = make_ctx();
        ctx.p2_bodydist_x = 30;
        assert!(expr.evaluate_bool(&ctx));
    }

    #[test]
    fn test_p2_ctrl() {
        let expr = TriggerParser::parse("P2Ctrl = 1").unwrap();
        let mut ctx = make_ctx();
        ctx.p2_ctrl = true;
        assert!(expr.evaluate_bool(&ctx));

        ctx.p2_ctrl = false;
        assert!(!expr.evaluate_bool(&ctx));
    }

    #[test]
    fn test_p2_combined_expression() {
        let expr = TriggerParser::parse("P2StateNo = 5000 && P2Life < 900").unwrap();
        let mut ctx = make_ctx();
        ctx.p2_stateno = 5000;
        ctx.p2_life = 800;
        assert!(expr.evaluate_bool(&ctx));

        ctx.p2_life = 1000;
        assert!(!expr.evaluate_bool(&ctx));
    }

    #[test]
    fn test_p2_statetype() {
        let expr = TriggerParser::parse("P2StateType = 2").unwrap();
        let mut ctx = make_ctx();
        ctx.p2_statetype = 2; // Aerial
        assert!(expr.evaluate_bool(&ctx));
    }

    #[test]
    fn test_p2_movetype() {
        let expr = TriggerParser::parse("P2MoveType = 1").unwrap();
        let mut ctx = make_ctx();
        ctx.p2_movetype = 1; // Attack
        assert!(expr.evaluate_bool(&ctx));
    }

    #[test]
    fn test_p2_vel() {
        let expr = TriggerParser::parse("P2Vel X > 0").unwrap();
        let mut ctx = make_ctx();
        ctx.p2_vel_x = 100;
        assert!(expr.evaluate_bool(&ctx));
    }
}
