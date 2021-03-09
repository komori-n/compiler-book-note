use std::collections::HashMap;

use crate::token::{Expr, Program, OperatorKind};

impl Program {
    pub fn compile(&self) {
        println!(".intel_syntax noprefix");
        println!(".global main");
        println!("main:");

        println!("    push rbp");
        println!("    mov rbp, rsp");
        println!("    sub rsp, 208");

        let mut map = HashMap::new();
        for expr in &self.stmts {
            expr.generate(&mut map);
            println!("    pop rax");
        }

        println!("    mov rsp, rbp");
        println!("    pop rbp");
        println!("    ret");
    }
}

impl Expr {
    fn generate(&self, idx_map :&mut HashMap<String, usize>) {
        match self {
            Expr::Num(num) => {
                println!("    push {}", num);
            },
            Expr::Ident(ident) => {
                println!("    mov rax, rbp");
                println!("    sub rax, {}", get_offset(idx_map, ident.to_owned()));
                println!("    mov rax, [rax]");
                println!("    push rax");
            },
            Expr::BinaryOperation(bin_op) => {
                if bin_op.op == OperatorKind::Assign {
                    match bin_op.left.as_ref() {
                        Expr::Ident(ident) => {
                            println!("    mov rax, rbp");
                            println!("    sub rax, {}", get_offset(idx_map, ident.to_owned()));
                            println!("    push rax");
                        },
                        _ => { unreachable!() }
                    }
                    bin_op.right.generate(idx_map);

                    println!("    pop rdi");
                    println!("    pop rax");
                    println!("    mov [rax], rdi");
                    println!("    push rdi");
                } else {
                    bin_op.left.generate(idx_map);
                    bin_op.right.generate(idx_map);

                    println!("    pop rdi");
                    println!("    pop rax");

                    match &bin_op.op {
                        &OperatorKind::Add => println!("    add rax, rdi"),
                        &OperatorKind::Substruct => println!("    sub rax, rdi"),
                        &OperatorKind::Multiply => println!("    imul rax, rdi"),
                        &OperatorKind::Divide => {
                            println!("    cqo");
                            println!("    idiv rax, rdi");
                        },
                        op => {
                            println!("    cmp rax, rdi");
                            match op {
                                &OperatorKind::Equal => println!("    sete al"),
                                &OperatorKind::NotEqual => println!("    setne al"),
                                &OperatorKind::Less => println!("    setl al"),
                                &OperatorKind::LessEqual => println!("    setle al"),
                                &OperatorKind::Greater => println!("    setg al"),
                                &OperatorKind::GreaterEqual => println!("    setge al"),
                                _ => unreachable!()
                            }
                            println!("    movzb rax, al");
                        }
                    }

                    println!("    push rax");
                }
            },
            Expr::Return(expr) => {
                expr.generate(idx_map);

                println!("    pop rax");
                println!("    mov rsp, rbp");
                println!("    pop rbp");
                println!("    ret")
            },
        }
    }
}

fn get_offset(idx_map: &mut HashMap<String, usize>, ident: String) -> usize {
    8 * get_idx(idx_map, ident)
}

fn get_idx(idx_map: &mut HashMap<String, usize>, ident: String) -> usize {
    let size = idx_map.len();
    idx_map.entry(ident)
        .or_insert(size)
        .to_owned()
}