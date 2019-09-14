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

fn build_symbol_table(source: Vec<Vec<Lexeme>>) -> HashMap<String, u32> {
    let mut symbol_table = HashMap::new();
    let mut addr: u32 = 0;
    for line in source {
        if let Lexeme::Label(label) = &line[0] {
            symbol_table.insert(label.clone(), addr);
        } else {
            addr += match line[0] {
                Lexeme::R(_) | Lexeme::I(_) | Lexeme::J(_) => 4,
                _ => 0,
            };
        }
    }
    symbol_table
}

fn lexer(source: String) -> Vec<Vec<Lexeme>> {
    let mut lexemes: Vec<Vec<Lexeme>> = Vec::new();
    for line in source.lines() {
        lexemes.push(
            Regex::new(r"([$a-z0-9]+)|,|\(|\)|:")
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
                            "and" => Lexeme::R(Ins::And),
                            "sub" => Lexeme::R(Ins::Sub),
                            "nor" => Lexeme::R(Ins::Nor),
                            "slt" => Lexeme::R(Ins::Slt),
                            "add" => Lexeme::R(Ins::Add),
                            "or" => Lexeme::R(Ins::Or),
                            "addu" => Lexeme::R(Ins::Addu),
                            "subu" => Lexeme::R(Ins::Subu),
                            "beq" => Lexeme::I(Ins::Beq),
                            "sw" => Lexeme::I(Ins::Sw),
                            "bne" => Lexeme::I(Ins::Bne),
                            "lui" => Lexeme::I(Ins::Lui),
                            "lw" => Lexeme::I(Ins::Lw),
                            "addi" => Lexeme::I(Ins::Addi),
                            "addiu" => Lexeme::I(Ins::Addiu),
                            "break" => Lexeme::J(Ins::Break),
                            "j" => Lexeme::J(Ins::J),
                            "zero" => Lexeme::Register(0),
                            "at" => Lexeme::Register(1),
                            "v0" => Lexeme::Register(2),
                            "v1" => Lexeme::Register(3),
                            "a0" => Lexeme::Register(4),
                            "a1" => Lexeme::Register(5),
                            "a2" => Lexeme::Register(6),
                            "a3" => Lexeme::Register(7),
                            "t0" => Lexeme::Register(8),
                            "t1" => Lexeme::Register(9),
                            "t2" => Lexeme::Register(10),
                            "t3" => Lexeme::Register(11),
                            "t4" => Lexeme::Register(12),
                            "t5" => Lexeme::Register(13),
                            "t6" => Lexeme::Register(14),
                            "t7" => Lexeme::Register(15),
                            "s0" => Lexeme::Register(16),
                            "s1" => Lexeme::Register(17),
                            "s2" => Lexeme::Register(18),
                            "s3" => Lexeme::Register(19),
                            "s4" => Lexeme::Register(20),
                            "s5" => Lexeme::Register(21),
                            "s6" => Lexeme::Register(22),
                            "s7" => Lexeme::Register(23),
                            "t8" => Lexeme::Register(24),
                            "t9" => Lexeme::Register(25),
                            "k0" => Lexeme::Register(26),
                            "k1" => Lexeme::Register(27),
                            "gp" => Lexeme::Register(28),
                            "sp" => Lexeme::Register(29),
                            "fp" => Lexeme::Register(30),
                            "ra" => Lexeme::Register(31),
                            label => Lexeme::Label(label.to_string()),
                        }
                    }
                })
                .collect(),
        );
    }
    lexemes
}

#[derive(Debug)]
enum Token {
    R(u8, u8, u8, u8, u8, u8),
    I(u8, u8, u8, u32),
    J(u8, u32),
}

fn tokenize_line(line: &Vec<Lexeme>, symbol_table: &HashMap<String, u32>) -> Option<Token> {
    if line.len() == 0 {
        None
    } else {
        match &line[0] {
            Lexeme::R(ins) => {
                if line.len() == 6 {
                    let rd = if let Lexeme::Register(val) = &line[1] {
                        *val
                    } else {
                        0
                    };
                    let rs = if let Lexeme::Register(val) = &line[3] {
                        *val
                    } else {
                        0
                    };

                    let rt = if let Lexeme::Register(val) = &line[5] {
                        *val
                    } else {
                        0
                    };

                    Some(Token::R(0, rs, rt, rd, 0, get_funct(ins)))
                } else {
                    println!(
                        "Tried to decode {:?}, but did not find expected number of symbols",
                        line
                    );
                    None
                }
            }
            Lexeme::I(ins) => {
                use Ins::*;
                let (s, t, i) = match ins {
                    Beq | Bne => {
                        if line.len() != 6 {
                            println!("wrong number of lexemes for {:?}", ins);
                            (0, 0, 0)
                        } else {
                            let s = if let Lexeme::Register(reg) = line[1] {
                                reg
                            } else {
                                println!("expected register, got {:?}", line[1]);
                                0
                            };
                            let t = if let Lexeme::Register(reg) = line[3] {
                                reg
                            } else {
                                println!("expected register, got {:?}", line[3]);
                                0
                            };
                            let u = if let Lexeme::Label(label) = &line[5] {
                                if let Some(addr) = symbol_table.get(label) {
                                    addr.clone()
                                } else {
                                    println!("Could not find label {:?} in symbol table", label);
                                    0
                                }
                            } else {
                                println!("expected label, got {:?}", line[5]);
                                0
                            };
                            (s, t, u)
                        }
                    }
                    Addi | Addiu => {
                        if line.len() != 6 {
                            println!("wrong number of lexemes for {:?}", ins);
                            (0, 0, 0)
                        } else {
                            let t = if let Lexeme::Register(reg) = line[1] {
                                reg
                            } else {
                                println!("expected register, got {:?}", line[1]);
                                0
                            };
                            let s = if let Lexeme::Register(reg) = line[3] {
                                reg
                            } else {
                                println!("expected register, got {:?}", line[3]);
                                0
                            };
                            let i = if let Lexeme::Number(number) = line[5] {
                                number
                            } else {
                                println!("expected number, got {:?}", line[5]);
                                0
                            };
                            (s, t, i)
                        }
                    }
                    Lw | Sw => {
                        if line.len() == 7 {
                            let t = if let Lexeme::Register(reg) = line[1] {
                                reg
                            } else {
                                println!("expected register, got {:?}", line[1]);
                                0
                            };
                            let s = if let Lexeme::Register(reg) = line[5] {
                                reg
                            } else {
                                println!("expected register, got {:?}", line[5]);
                                0
                            };
                            let i = if let Lexeme::Number(number) = line[3] {
                                number
                            } else {
                                println!("expected number, got {:?}", line[3]);
                                0
                            };
                            (s, t, i)
                        } else if line.len() == 4 {
                            let t = if let Lexeme::Register(reg) = line[1] {
                                reg
                            } else {
                                println!("expected register, got {:?}", line[1]);
                                0
                            };
                            let s = if let Lexeme::Register(reg) = line[3] {
                                reg
                            } else {
                                println!("expected register, got {:?}", line[3]);
                                0
                            };
                            (s, t, 0)
                        } else {
                            println!("wrong number of lexemes for {:?}", ins);
                            (0, 0, 0)
                        }
                    }
                    Lui => {
                        if line.len() != 4 {
                            println!("wrong number of lexemes for {:?}", ins);
                            (0, 0, 0)
                        } else {
                            let t = if let Lexeme::Register(reg) = line[1] {
                                reg
                            } else {
                                println!("expected register, got {:?}", line[1]);
                                0
                            };
                            let i = if let Lexeme::Number(number) = line[3] {
                                number
                            } else {
                                println!("expected number, got {:?}", line[4]);
                                0
                            };
                            (0, t, i)
                        }
                    }
                    instr => {
                        println!("Unexpected instruction {:?}", instr);
                        (0, 0, 0)
                    }
                };

                Some(Token::I(get_opcode(ins), s, t, i))
            }
            Lexeme::J(Ins::J) => {
                if line.len() != 2 {
                    println!("wrong number of lexemes for jmp");
                    None
                } else {
                    let addr = if let Lexeme::Label(label) = &line[1] {
                        if let Some(addr) = symbol_table.get(label) {
                            addr.clone()
                        } else {
                            println!("Could not find label {:?} in symbol table", label);
                            0
                        }
                    } else {
                        println!("Expected addr/label, got {:?}", line[1]);
                        0
                    };

                    Some(Token::J(get_opcode(&Ins::J), addr))
                }
            }
            Lexeme::J(Ins::Break) => {
                if line.len() != 2 {
                    println!(
                        "wrong number of lexemes for break, expected 1, got {}",
                        line.len()
                    );
                    None
                } else {
                    Some(Token::J(get_opcode(&Ins::Break), 0xd))
                }
            }
            _ => None,
        }
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
        Token::J(opcode, addr) => ((opcode as u32) << 26) | addr,
    }
}

pub fn assemble_file(filename: String) -> Result<(), Box<dyn Error>> {
    let source = fs::read_to_string(filename)?;
    let lexemes = lexer(source);
    let symbol_table = build_symbol_table(lexemes.clone());
    let mut line_nr = 0;
    for line in lexemes {
        let tokens = tokenize_line(&line, &symbol_table);
        if let Some(token) = tokens {
            let instruction: u32 = asseble_token(token);
            println!("0x{:08x}\t0x{:08x}", line_nr, instruction);
            line_nr += 4;
        }
    }

    Ok(())
}
