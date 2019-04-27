use crate::{
    error::BlisprError,
    eval::lval_eval,
    lval::{lval_add, lval_num, lval_qexpr, lval_sexpr, lval_sym, Lval},
};
use pest::{iterators::Pair, Parser};
use rustyline::{error::ReadlineError, Editor};

#[cfg(debug_assertions)]
const _GRAMMAR: &str = include_str!("blispr.pest");

#[derive(Parser)]
#[grammar = "blispr.pest"]
pub struct BlisprParser;

fn is_bracket_or_eoi(parsed: &Pair<Rule>) -> bool {
    if parsed.as_rule() == Rule::EOI {
        return true;
    }
    let c = parsed.as_str();
    c == "(" || c == ")" || c == "{" || c == "}"
}

fn lval_read(parsed: Pair<Rule>) -> Result<Box<Lval>, BlisprError> {
    match parsed.as_rule() {
        Rule::blispr | Rule::sexpr => {
            let mut ret = lval_sexpr();
            for child in parsed.into_inner() {
                // here is where you skip stuff
                if is_bracket_or_eoi(&child) {
                    continue;
                }
                lval_add(&mut ret, lval_read(child)?)?;
            }
            Ok(ret)
        }
        Rule::expr => lval_read(parsed.into_inner().next().unwrap()),
        Rule::qexpr => {
            let mut ret = lval_qexpr();
            for child in parsed.into_inner() {
                if is_bracket_or_eoi(&child) {
                    continue;
                }
                lval_add(&mut ret, lval_read(child)?)?;
            }
            Ok(ret)
        }
        Rule::num => Ok(lval_num(parsed.as_str().parse::<i64>().unwrap())),
        Rule::symbol => Ok(lval_sym(parsed.as_str())),
        _ => unreachable!(), // COMMENT/WHITESPACE etc
    }
}

pub fn repl(print_parsed: bool) -> Result<(), BlisprError> {
    println!("Blispr v0.0.1");
    println!("Press Ctrl-C or Ctrl-D to exit");
    if print_parsed {
        println!("Debug mode enabled");
    }

    let mut rl = Editor::<()>::new();
    if rl.load_history("./.blisp-history.txt").is_err() {
        println!("No history found.");
    }

    loop {
        let input = rl.readline("blispr> ");
        match input {
            Ok(line) => {
                rl.add_history_entry(line.as_ref());
                let parsed = match BlisprParser::parse(Rule::blispr, &line) {
                    Ok(mut iter) => iter.next().unwrap(),
                    Err(e) => {
                        println!("Syntax error:\n{}", e);
                        continue;
                    }
                };
                debug!("{}", parsed);
                let lval_ptr = lval_read(parsed)?;
                if print_parsed {
                    println!("{}", *lval_ptr);
                }
                match lval_eval(lval_ptr) {
                    Ok(r) => {
                        debug!("Result: {:?}", r);
                        println!("{}", r);
                    }
                    Err(e) => eprintln!("Error: {}", e),
                };
            }
            Err(ReadlineError::Interrupted) => {
                info!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                info!("CTRL-D");
                break;
            }
            Err(err) => {
                warn!("Error: {:?}", err);
                break;
            }
        }
    }
    rl.save_history("./.blisp-history.txt").unwrap();
    Ok(())
}