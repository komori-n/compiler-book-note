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

        let mut label_idx = 0;
        let mut map = HashMap::new();
        for expr in &self.stmts {
            expr.generate(&mut map, &mut label_idx);
            println!("    pop rax");
        }

        println!("    mov rsp, rbp");
        println!("    pop rbp");
        println!("    ret");
    }
}

impl Expr {
    fn generate(&self, ident_map :&mut HashMap<String, usize>, label_idx: &mut usize) {
        match self {
            Expr::Num(num) => {
                println!("    push {}", num);
            },
            Expr::Ident(ident) => {
                println!("    mov rax, rbp");
                println!("    sub rax, {}", get_offset(ident_map, ident.to_owned()));
                println!("    mov rax, [rax]");
                println!("    push rax");
            },
            Expr::BinaryOperation(bin_op) => {
                if bin_op.op == OperatorKind::Assign {
                    match bin_op.left.as_ref() {
                        Expr::Ident(ident) => {
                            println!("    mov rax, rbp");
                            println!("    sub rax, {}", get_offset(ident_map, ident.to_owned()));
                            println!("    push rax");
                        },
                        _ => { unreachable!() }
                    }
                    bin_op.right.generate(ident_map, label_idx);

                    println!("    pop rdi");
                    println!("    pop rax");
                    println!("    mov [rax], rdi");
                    println!("    push rdi");
                } else {
                    bin_op.left.generate(ident_map, label_idx);
                    bin_op.right.generate(ident_map, label_idx);

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
                expr.generate(ident_map, label_idx);

                println!("    pop rax");
                println!("    mov rsp, rbp");
                println!("    pop rbp");
                println!("    ret")
            },
            Expr::If(cond, then, else_) => {
                let if_label_idx = *label_idx;
                *label_idx += 1;

                cond.generate(ident_map, label_idx);
                println!("    pop rax");
                println!("    cmp rax, 0");
                println!("    je  .Lelse{}", label_idx);

                then.generate(ident_map, label_idx);
                println!("    jmp .Lend{}", label_idx);

                println!(".Lelse{}:", label_idx);
                if let Some(else_) = else_ {
                    else_.generate(ident_map, label_idx);
                }
                println!(".Lend{}:", label_idx);
            },
            Expr::While(cond, stmt) => {
                let while_label_idx = *label_idx;
                *label_idx += 1;

                println!(".Lbegin{}:", while_label_idx);
                cond.generate(ident_map, label_idx);
                println!("    pop rax");
                println!("    cmp rax, 0");
                println!("    je .Lend{}", while_label_idx);

                stmt.generate(ident_map, label_idx);
                println!("    jmp .Lbegin{}", while_label_idx);
                println!(".Lend{}:", while_label_idx);
            }
        }
    }
}

fn get_offset(ident_map: &mut HashMap<String, usize>, ident: String) -> usize {
    8 * get_ident(ident_map, ident)
}

fn get_ident(ident_map: &mut HashMap<String, usize>, ident: String) -> usize {
    let size = ident_map.len();
    ident_map.entry(ident)
        .or_insert(size)
        .to_owned()
}