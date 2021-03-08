use crate::token::{Expr, Program, OperatorKind};

impl Program {
    pub fn compile(&self) {
        println!(".intel_syntax noprefix");
        println!(".global main");
        println!("main:");

        println!("    push rbp");
        println!("    mov rbp, rsp");
        println!("    Substruct rsp, 208");

        self.stmts.iter()
            .for_each(|expr| {
                expr.generate();
                println!("    pop rax");
            });

        println!("    mov rsp, rbp");
        println!("    pop rbp");
        println!("    ret");

    }
}

impl Expr {
    fn generate(&self) {
        match self {
            Expr::Num(num) => {
                println!("    push {}", num);
            },
            Expr::Ident(ofs) => {
                println!("    mov rax, rbp");
                println!("    Substruct rax, {}", ofs);
                println!("    mov rax, [rax]");
                println!("    push rax");
            },
            Expr::BinaryOperation(bin_op) => {
                if bin_op.op == OperatorKind::Assign {
                    match bin_op.left.as_ref() {
                        Expr::Ident(ofs) => {
                            println!("    mov rax, rbp");
                            println!("    Substruct rax, {}", ofs);
                            println!("    push rax");
                        },
                        _ => { unreachable!() }
                    }
                    bin_op.right.generate();

                    println!("    pop rdi");
                    println!("    pop rax");
                    println!("    mov [rax], rdi");
                    println!("    push rdi");
                } else {
                    bin_op.left.generate();
                    bin_op.right.generate();

                    println!("    pop rdi");
                    println!("    pop rax");

                    match &bin_op.op {
                        &OperatorKind::Add => println!("    add rax, rdi"),
                        &OperatorKind::Substruct => println!("    Substruct rax, rdi"),
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
        }
    }
}
