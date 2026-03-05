use tickle_mugen::{TriggerParser, TriggerExpr};

#[test]
fn test_trigger_parser_simple() {
    // Simple comparison
    let expr = TriggerParser::parse("Time >= 10").unwrap();
    match expr {
        TriggerExpr::Ge(left, right) => {
            assert!(matches!(*left, TriggerExpr::Variable(_)));
            assert!(matches!(*right, TriggerExpr::Int(10)));
        }
        _ => panic!("Expected Ge expression"),
    }
}

#[test]
fn test_trigger_parser_and() {
    // Logical AND
    let expr = TriggerParser::parse("Time >= 10 && Vel Y > 0").unwrap();
    match expr {
        TriggerExpr::And(left, right) => {
            assert!(matches!(*left, TriggerExpr::Ge(_, _)));
            assert!(matches!(*right, TriggerExpr::Gt(_, _)));
        }
        _ => panic!("Expected And expression"),
    }
}

#[test]
fn test_trigger_parser_function() {
    // Function call
    let expr = TriggerParser::parse("ifelse(var(8)=0, 10, 20)").unwrap();
    match expr {
        TriggerExpr::FunctionCall(name, args) => {
            assert_eq!(name, "ifelse");
            assert_eq!(args.len(), 3);
        }
        _ => panic!("Expected FunctionCall expression"),
    }
}

#[test]
fn test_trigger_parser_nested() {
    // Nested expression with parentheses
    let expr = TriggerParser::parse("(Time > 5) && (Vel X < 10)").unwrap();
    match expr {
        TriggerExpr::And(left, right) => {
            assert!(matches!(*left, TriggerExpr::Gt(_, _)));
            assert!(matches!(*right, TriggerExpr::Lt(_, _)));
        }
        _ => panic!("Expected And expression"),
    }
}

#[test]
fn test_trigger_parser_arithmetic() {
    // Arithmetic operations
    let expr = TriggerParser::parse("Time + 5 * 2").unwrap();
    match expr {
        TriggerExpr::Add(left, right) => {
            assert!(matches!(*left, TriggerExpr::Variable(_)));
            assert!(matches!(*right, TriggerExpr::Mul(_, _)));
        }
        _ => panic!("Expected Add expression"),
    }
}

#[test]
fn test_trigger_parser_not() {
    // Logical NOT
    let expr = TriggerParser::parse("!ctrl").unwrap();
    match expr {
        TriggerExpr::Not(inner) => {
            assert!(matches!(*inner, TriggerExpr::Variable(_)));
        }
        _ => panic!("Expected Not expression"),
    }
}

#[test]
fn test_trigger_parser_string() {
    // String comparison
    let expr = TriggerParser::parse("command = \"holdfwd\"").unwrap();
    match expr {
        TriggerExpr::Eq(left, right) => {
            assert!(matches!(*left, TriggerExpr::Variable(_)));
            assert!(matches!(*right, TriggerExpr::String(_)));
        }
        _ => panic!("Expected Eq expression"),
    }
}

#[test]
fn test_trigger_parser_complex() {
    // Complex expression from acceptance criteria
    let expr = TriggerParser::parse("Time >= 10 && Vel Y > 0").unwrap();

    // Should parse as: (Time >= 10) && (Vel Y > 0)
    if let TriggerExpr::And(left, right) = expr {
        // Left side: Time >= 10
        if let TriggerExpr::Ge(time_var, ten) = *left {
            if let TriggerExpr::Variable(name) = *time_var {
                assert_eq!(name, "Time");
            } else {
                panic!("Expected Variable");
            }
            assert!(matches!(*ten, TriggerExpr::Int(10)));
        } else {
            panic!("Expected Ge");
        }

        // Right side: Vel Y > 0
        if let TriggerExpr::Gt(vel_var, zero) = *right {
            if let TriggerExpr::Variable(name) = *vel_var {
                assert_eq!(name, "Vel Y");
            } else {
                panic!("Expected Variable");
            }
            assert!(matches!(*zero, TriggerExpr::Int(0)));
        } else {
            panic!("Expected Gt");
        }
    } else {
        panic!("Expected And");
    }
}

#[test]
fn test_trigger_parser_ifelse() {
    // ifelse function from acceptance criteria
    let expr = TriggerParser::parse("ifelse(var(8)=0, 10, 20)").unwrap();

    if let TriggerExpr::FunctionCall(name, args) = expr {
        assert_eq!(name, "ifelse");
        assert_eq!(args.len(), 3);

        // First arg: var(8)=0
        if let TriggerExpr::Eq(_, _) = &args[0] {
            // OK
        } else {
            panic!("Expected Eq in first argument");
        }

        // Second arg: 10
        assert!(matches!(args[1], TriggerExpr::Int(10)));

        // Third arg: 20
        assert!(matches!(args[2], TriggerExpr::Int(20)));
    } else {
        panic!("Expected FunctionCall");
    }
}
