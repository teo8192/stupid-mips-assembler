extern crate regex;

use regex::Regex;
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::str::FromStr;

#[derive(Debug, Clone)]
enum Ins {
    J,
    Beq,
    Bne,
    Lui,
    Slt,
    Lw,
    Sw,
    Add,
    Addu,
    Addi,
    Addiu,
    Sub,
    Subu,
    And,
    Or,
    Nor,
    Break,
}

fn get_funct(ins: &Ins) -> u8 {
    match ins {
        Ins::Add => 0x20,
        Ins::Addu => 0x21,
        Ins::Sub => 0x22,
        Ins::Subu => 0x23,
        Ins::And => 0x24,
        Ins::Or => 0x25,
        Ins::Nor => 0x27,
        Ins::Slt => 0x2a,
        _ => {
            println!("Tried to get funct of instruction {:?}", ins);
            0
        }
    }
}

fn get_opcode(ins: &Ins) -> u8 {
    match ins {
        Ins::Break => 0,
        Ins::J => 2,

        Ins::Beq => 0x4,
        Ins::Bne => 0x5,
        Ins::Addi => 0x8,
        Ins::Addiu => 0x9,
        Ins::Lui => 0xf,
        Ins::Lw => 0x23,
        Ins::Sw => 0x2b,

        _ => 0, // The ones with funcu have opcode 0
    }
}

#[derive(Debug, Clone)]
enum Lexeme {
    Comma,
    R(Ins),
    I(Ins),
    J(Ins),
    Register(u8),
    Label(String),
    Number(u32),
    OpenParen,
    CloseParen,
    Colon,
}

fn build_symbol_table(source: Vec<Vec<Lexeme>>, start: u32) -> HashMap<String, u32> {
    let mut symbol_table = HashMap::new();
    let mut addr: u32 = start;
    for mut line in source {
        addr += match line.pop() {
            Some(Lexeme::Label(label)) => {
                symbol_table.insert(label.clone(), addr >> 2);
                0
            }
            Some(Lexeme::R(_)) | Some(Lexeme::I(_)) | Some(Lexeme::J(_)) => 4,
            _ => 0,
        };
    }
    symbol_table
}

fn lexer(source: String) -> Vec<Vec<Lexeme>> {
    let mut lexemes: Vec<Vec<Lexeme>> = Vec::new();
    for line in source.lines() {
        lexemes.push(reverse(
            Regex::new(r"(-?[$a-z0-9]+)|,|\(|\)|:")
                .unwrap()
                .find_iter(&line)
                .map(|expr| {
                    if let Ok(num) = FromStr::from_str(expr.as_str()) {
                        Lexeme::Number(num)
                    } else {
                        match expr.as_str() {
                            "," => Lexeme::Comma,
                            "(" => Lexeme::OpenParen,
                            ")" => Lexeme::CloseParen,
                            ":" => Lexeme::Colon,
                            "add" => Lexeme::R(Ins::Add),
                            "addu" => Lexeme::R(Ins::Addu),
                            "sub" => Lexeme::R(Ins::Sub),
                            "subu" => Lexeme::R(Ins::Subu),
                            "nor" => Lexeme::R(Ins::Nor),
                            "or" => Lexeme::R(Ins::Or),
                            "and" => Lexeme::R(Ins::And),
                            "slt" => Lexeme::R(Ins::Slt),
                            "addi" => Lexeme::I(Ins::Addi),
                            "addiu" => Lexeme::I(Ins::Addiu),
                            "beq" => Lexeme::I(Ins::Beq),
                            "bne" => Lexeme::I(Ins::Bne),
                            "sw" => Lexeme::I(Ins::Sw),
                            "lw" => Lexeme::I(Ins::Lw),
                            "lui" => Lexeme::I(Ins::Lui),
                            "break" => Lexeme::J(Ins::Break),
                            "j" => Lexeme::J(Ins::J),
                            "$zero" => Lexeme::Register(0),
                            "$at" => Lexeme::Register(1),
                            "$v0" => Lexeme::Register(2),
                            "$v1" => Lexeme::Register(3),
                            "$a0" => Lexeme::Register(4),
                            "$a1" => Lexeme::Register(5),
                            "$a2" => Lexeme::Register(6),
                            "$a3" => Lexeme::Register(7),
                            "$t0" => Lexeme::Register(8),
                            "$t1" => Lexeme::Register(9),
                            "$t2" => Lexeme::Register(10),
                            "$t3" => Lexeme::Register(11),
                            "$t4" => Lexeme::Register(12),
                            "$t5" => Lexeme::Register(13),
                            "$t6" => Lexeme::Register(14),
                            "$t7" => Lexeme::Register(15),
                            "$s0" => Lexeme::Register(16),
                            "$s1" => Lexeme::Register(17),
                            "$s2" => Lexeme::Register(18),
                            "$s3" => Lexeme::Register(19),
                            "$s4" => Lexeme::Register(20),
                            "$s5" => Lexeme::Register(21),
                            "$s6" => Lexeme::Register(22),
                            "$s7" => Lexeme::Register(23),
                            "$t8" => Lexeme::Register(24),
                            "$t9" => Lexeme::Register(25),
                            "$k0" => Lexeme::Register(26),
                            "$k1" => Lexeme::Register(27),
                            "$gp" => Lexeme::Register(28),
                            "$sp" => Lexeme::Register(29),
                            "$fp" => Lexeme::Register(30),
                            "$ra" => Lexeme::Register(31),
                            label => Lexeme::Label(label.to_string()),
                        }
                    }
                })
                .collect(),
        ));
    }
    lexemes
        .into_iter()
        .filter(|lexemes| lexemes.len() > 0)
        .collect()
}

#[derive(Debug)]
enum Token {
    R(u8, u8, u8, u8, u8, u8),
    I(u8, u8, u8, u32),
    J(u8, u32),
}

fn parse_addr(line: &mut Vec<Lexeme>, symbol_table: &HashMap<String, u32>) -> Option<(u8, u32)> {
    match line.pop() {
        Some(Lexeme::Number(num)) => match line.pop() {
            Some(Lexeme::OpenParen) => {
                if let Some(Lexeme::Register(reg)) = line.pop() {
                    match line.pop() {
                        Some(Lexeme::CloseParen) => Some((reg, num)),
                        _ => {
                            println!("Expected close parenthesis");
                            None
                        }
                    }
                } else {
                    println!("Expected register");
                    None
                }
            }
            None => Some((0, num)),
            _ => {
                println!("expected open parenthesis after number for adress mode");
                None
            }
        },
        Some(Lexeme::OpenParen) => {
            if let Some(Lexeme::Register(reg)) = line.pop() {
                match line.pop() {
                    Some(Lexeme::CloseParen) => Some((reg, 0)),
                    _ => {
                        println!("Expected close parenthesis");
                        None
                    }
                }
            } else {
                println!("Expected register");
                None
            }
        }
        Some(Lexeme::Label(label)) => {
            if let Some(addr) = symbol_table.get(&label) {
                Some((0, addr.clone()))
            } else {
                println!("Did not find label in symbol table");
                None
            }
        }
        lexeme => {
            println!("Got {:?} when an adress was expected", lexeme);
            None
        }
    }
}

fn get_relative_addr(line_nr: u32, addr: u32, bits: u32) -> u32 {
    let line_nr = (line_nr ^ 0xffffffff) + addr;

    line_nr & ((1 << bits) - 1)
}

fn tokenize_line(
    line: &mut Vec<Lexeme>,
    symbol_table: &HashMap<String, u32>,
    line_nr: u32,
) -> Option<Token> {
    match line.pop() {
        Some(Lexeme::R(ins)) => {
            let rd = if let Some(Lexeme::Register(val)) = line.pop() {
                val
            } else {
                return None;
            };
            line.pop();
            let rs = if let Some(Lexeme::Register(val)) = line.pop() {
                val
            } else {
                return None;
            };
            line.pop();
            let rt = if let Some(Lexeme::Register(val)) = line.pop() {
                val
            } else {
                return None;
            };
            line.pop();

            Some(Token::R(0, rs, rt, rd, 0, get_funct(&ins)))
        }
        Some(Lexeme::I(ins)) => {
            use Ins::*;
            match ins {
                Beq | Bne => {
                    let s = if let Some(Lexeme::Register(val)) = line.pop() {
                        val
                    } else {
                        return None;
                    };
                    line.pop();
                    let t = if let Some(Lexeme::Register(val)) = line.pop() {
                        val
                    } else {
                        return None;
                    };
                    line.pop();
                    if let Some((r, o)) = parse_addr(line, symbol_table) {
                        if r == 0 {
                            Some(Token::I(
                                get_opcode(&ins),
                                s,
                                t,
                                get_relative_addr(line_nr, o, 16),
                            ))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
                Addi | Addiu => {
                    let t = if let Some(Lexeme::Register(val)) = line.pop() {
                        val
                    } else {
                        return None;
                    };
                    line.pop();
                    let s = if let Some(Lexeme::Register(val)) = line.pop() {
                        val
                    } else {
                        return None;
                    };
                    line.pop();
                    if let Some((r, o)) = parse_addr(line, symbol_table) {
                        if r == 0 {
                            Some(Token::I(get_opcode(&ins), s, t, o))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
                Lw | Sw => {
                    let t = if let Some(Lexeme::Register(val)) = line.pop() {
                        val
                    } else {
                        return None;
                    };
                    line.pop();
                    if let Some((s, o)) = parse_addr(line, symbol_table) {
                        Some(Token::I(get_opcode(&ins), s, t, o))
                    } else {
                        None
                    }
                }
                Lui => {
                    let t = if let Some(Lexeme::Register(val)) = line.pop() {
                        val
                    } else {
                        return None;
                    };
                    line.pop();
                    if let Some((r, i)) = parse_addr(line, symbol_table) {
                        if r == 0 {
                            Some(Token::I(get_opcode(&ins), 0, t, i))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
                _ => None,
            }
        }
        Some(Lexeme::J(Ins::J)) => {
            if let Some((r, addr)) = parse_addr(line, symbol_table) {
                if r == 0 {
                    Some(Token::J(get_opcode(&Ins::J), addr))
                } else {
                    None
                }
            } else {
                None
            }
        }
        Some(Lexeme::J(Ins::Break)) => {
            if line.len() != 0 {
                println!(
                    "wrong number of lexemes for break, expected 1, got {}",
                    line.len()
                );
                None
            } else {
                Some(Token::J(get_opcode(&Ins::Break), 0xd))
            }
        }
        _lexeme => None,
    }
}

fn asseble_token(instr: Token) -> u32 {
    match instr {
        Token::R(opcode, rs, rt, rd, shamt, funct) => {
            ((opcode as u32) << 26)
                | ((rs as u32) << 21)
                | ((rt as u32) << 16)
                | ((rd as u32) << 11)
                | ((shamt as u32) << 6)
                | funct as u32
        }
        Token::I(opcode, rs, rt, immidiate) => {
            ((opcode as u32) << 26) | ((rs as u32) << 21) | ((rt as u32) << 16) | immidiate
        }
        Token::J(opcode, addr) => ((opcode as u32) << 26) | (addr & ((1 << 26) - 1)),
    }
}

fn reverse<T>(mut input: Vec<T>) -> Vec<T> {
    let mut reversed: Vec<T> = Vec::new();
    let size = input.len();

    for _ in 0..size {
        if let Some(val) = input.pop() {
            reversed.push(val);
        }
    }

    reversed
}

fn tokenize(
    lexemes: &mut Vec<Vec<Lexeme>>,
    symbol_table: &HashMap<String, u32>,
    start: u32,
) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut line_nr = start >> 2;

    for line in lexemes {
        if let Some(token) = tokenize_line(line, symbol_table, line_nr) {
            tokens.push(token);
            line_nr += 1;
        }
    }

    tokens
}

fn asseble(tokens: Vec<Token>) -> Vec<u32> {
    tokens
        .into_iter()
        .map(|token| asseble_token(token))
        .collect()
}

pub fn assemble_file(filename: String) -> Result<(), Box<dyn Error>> {
    let start: u32 = 0xbfc00000;
    let mut lexemes = lexer(fs::read_to_string(filename)?);
    let symbol_table = build_symbol_table(lexemes.clone(), start);
    let tokens = tokenize(&mut lexemes, &symbol_table, start);
    let instructions = asseble(tokens);
    let mut line_nr = start;
    for instruction in instructions.into_iter() {
        println!("0x{:08x}\t0x{:08x}", line_nr, instruction);
        line_nr += 4;
    }

    Ok(())
}
