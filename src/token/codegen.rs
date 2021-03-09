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
        let curr_label_idx = *label_idx;
        *label_idx += 1;

        let begin_label = format!(".Lbegin{}", curr_label_idx);
        let else_label = format!(".Lelse{}", curr_label_idx);
        let end_label = format!(".Lend{}", curr_label_idx);


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
                cond.generate(ident_map, label_idx);
                println!("    pop rax");
                println!("    cmp rax, 0");
                println!("    je  {}", else_label);

                then.generate(ident_map, label_idx);
                println!("    jmp {}", end_label);

                println!("{}:", else_label);
                if let Some(else_) = else_ {
                    else_.generate(ident_map, label_idx);
                }
                println!("{}:", end_label);
            },
            Expr::While(cond, stmt) => {
                println!("{}:", begin_label);
                cond.generate(ident_map, label_idx);
                println!("    pop rax");
                println!("    cmp rax, 0");
                println!("    je {}", end_label);

                stmt.generate(ident_map, label_idx);
                println!("    jmp {}", begin_label);
                println!("{}:", end_label);
            },
            Expr::For(init, cond, end, stmt) => {
                if let Some(init) = init {
                    init.generate(ident_map, label_idx);
                }
                println!("{}:", begin_label);
                if let Some(cond) = cond {
                    cond.generate(ident_map, label_idx);
                    println!("    pop rax");
                    println!("    cmp rax, 0");
                    println!("    je  {}", end_label);
                }
                stmt.generate(ident_map, label_idx);
                if let Some(end) = end {
                    end.generate(ident_map, label_idx);
                }
                println!("jmp {}", begin_label);
                println!("{}:", end_label);
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